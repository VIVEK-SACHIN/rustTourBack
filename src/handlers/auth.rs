use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use mongodb::bson::doc;
use serde::Deserialize;
use serde_json::json;
use sha2::Digest;
use time::Duration;

use crate::jwt_util::sign_jwt;
use crate::models::user::{hash_password, User, UserRole};
use crate::state::AppState;
use crate::utils::email::Email;
use crate::utils::error::AppError;
use crate::utils::validate::{validate_email, validate_password};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignupBody {
    pub name: String,
    pub email: String,
    pub password: String,
    pub password_confirm: String,
    #[serde(default)]
    pub role: Option<UserRole>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginBody {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ForgotPasswordBody {
    pub email: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResetPasswordBody {
    pub new_password: String,
    pub password_confirm: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePasswordBody {
    pub password_current: String,
    pub password: String,
    pub password_confirm: String,
}

fn jwt_same_site(state: &AppState) -> SameSite {
    if state.config.jwt_cookie_same_site_cross_origin() {
        // tour.vivekdev.fun → tourapi.vivekdev.fun (cross-site XHR with credentials)
        SameSite::None
    } else {
        SameSite::Lax
    }
}

fn jwt_cookie(token: &str, state: &AppState) -> Cookie<'static> {
    let days = state.config.jwt_cookie_expires_in.clamp(1, 365) as i64;
    let mut c = Cookie::build(("jwt", token.to_string()))
        .path("/")
        .http_only(true)
        .max_age(Duration::days(days))
        .same_site(jwt_same_site(state));
    if state.config.is_production() {
        c = c.secure(true);
    }
    c.build()
}

fn logged_out_cookie(state: &AppState) -> Cookie<'static> {
    let mut c = Cookie::build(("jwt", "loggedout"))
        .path("/")
        .http_only(true)
        .max_age(Duration::seconds(10))
        .same_site(jwt_same_site(state));
    if state.config.is_production() {
        c = c.secure(true);
    }
    c.build()
}

fn json_user(user: User) -> serde_json::Value {
    serde_json::to_value(user.strip_secrets_for_response()).unwrap_or(json!({}))
}

pub async fn signup(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(body): Json<SignupBody>,
) -> Result<impl IntoResponse, AppError> {
    validate_email(&body.email)?;
    validate_password(&body.password)?;
    if body.password != body.password_confirm {
        return Err(AppError::bad_request("Passwords are not the same!"));
    }

    let db = state.db();
    let users = db.collection::<User>("users");

    let email_lower = body.email.trim().to_lowercase();
    let exists = users
        .find_one(doc! { "email": &email_lower })
        .await
        .map_err(AppError::from)?;
    if exists.is_some() {
        return Err(AppError::bad_request(
            "There is already a user with this email address.",
        ));
    }

    let hash = hash_password(&body.password).map_err(AppError::from)?;

    // TravelAndTour signup should not allow self-assigning admin; always create a `user`.
    let _ = body.role;
    let role = UserRole::User;

    let new_user = User {
        id: None,
        name: body.name.trim().to_string(),
        email: email_lower,
        photo: "default.jpg".to_string(),
        role,
        password: Some(hash),
        password_confirm: None,
        changed_password_at: None,
        password_reset_token: None,
        password_reset_token_expires: None,
        active: true,
    };

    let insert = users.insert_one(&new_user).await.map_err(AppError::from)?;
    let id = insert.inserted_id.as_object_id().ok_or_else(|| {
        AppError::internal("Could not read inserted user id from database.")
    })?;

    let mut user_out = new_user;
    user_out.id = Some(id);

    let welcome_url = if state.config.publish_url.is_empty() {
        format!("http://{}:{}/api/v1/users/me", state.config.host, state.config.port)
    } else {
        format!("{}/me", state.config.publish_url.trim_end_matches('/'))
    };
    if let Err(e) = Email::new(&user_out.name, &user_out.email, &welcome_url, &state.config)
        .send_welcome()
        .await
    {
        eprintln!("Error sending welcome email: {:?}", e);
    }

    let id_hex = id.to_hex();
    let token = sign_jwt(&id_hex, &state.config)?;
    let cookie = jwt_cookie(&token, &state);
    let jar = jar.add(cookie);

    Ok((
        StatusCode::CREATED,
        jar,
        Json(json!({
            "status": "success",
            "token": token,
            "data": { "user": json_user(user_out) }
        })),
    ))
}

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(body): Json<LoginBody>,
) -> Result<impl IntoResponse, AppError> {
    validate_email(&body.email)?;
    let email = body.email.trim().to_lowercase();
    if body.password.is_empty() {
        return Err(AppError::bad_request("please provide email and password"));
    }

    let db = state.db();
    let users = db.collection::<User>("users");

    let user = users
        .find_one(doc! { "email": &email, "active": { "$ne": false } })
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::unauthorized("username Or password is incorrect"))?;

    if !user.verify_password(&body.password) {
        return Err(AppError::unauthorized("username Or password is incorrect"));
    }

    let id = user
        .id
        .ok_or_else(|| AppError::internal("User document missing _id."))?;
    let id_hex = id.to_hex();
    let token = sign_jwt(&id_hex, &state.config)?;
    let cookie = jwt_cookie(&token, &state);
    let jar = jar.add(cookie);

    Ok((
        jar,
        Json(json!({
            "status": "success",
            "token": token,
            "data": { "user": json_user(user) }
        })),
    ))
}

pub async fn logout(State(state): State<AppState>, jar: CookieJar) -> impl IntoResponse {
    let jar = jar.add(logged_out_cookie(&state));
    (jar, Json(json!({ "status": "success" })))
}

/// Same behavior as Node: 404 if email unknown; on success logs reset link (email not wired yet).
pub async fn forgot_password(
    State(state): State<AppState>,
    Json(body): Json<ForgotPasswordBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let email = body.email.trim().to_lowercase();
    let db = state.db();
    let users = db.collection::<User>("users");

    let mut user = users
        .find_one(doc! { "email": &email })
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::not_found("There is no user with email address."))?;

    let reset_token = user.create_password_reset_token();
    let id = user.id.ok_or_else(|| AppError::internal("User missing _id."))?;

    users
        .replace_one(doc! { "_id": id }, &user)
        .await
        .map_err(AppError::from)?;

    let host = format!("{}:{}", state.config.host, state.config.port);
    let reset_url = format!("http://{host}/api/v1/users/resetPassword/{reset_token}");

    match Email::new(&user.name, &user.email, &reset_url, &state.config)
        .send_password_reset()
        .await
    {
        Ok(()) => Ok(Json(json!({
            "status": "success",
            "message": "Token sent to email!"
        }))),
        Err(err) => {
            user.password_reset_token = None;
            user.password_reset_token_expires = None;
            users
                .replace_one(doc! { "_id": id }, &user)
                .await
                .map_err(AppError::from)?;
            Err(AppError::internal(
                "There was an error sending the email. Try again later!",
            )
            .with_debug_detail(format!("{err:?}")))
        }
    }
}

pub async fn reset_password(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(token): Path<String>,
    Json(body): Json<ResetPasswordBody>,
) -> Result<impl IntoResponse, AppError> {
    validate_password(&body.new_password)?;
    if body.new_password != body.password_confirm {
        return Err(AppError::bad_request("Passwords are not the same!"));
    }

    let hashed = {
        let mut h = sha2::Sha256::new();
        h.update(token.as_bytes());
        hex::encode(h.finalize())
    };

    let db = state.db();
    let users = db.collection::<User>("users");
    let now_ms = chrono::Utc::now().timestamp_millis();

    let mut user = users
        .find_one(doc! {
            "passwordResetToken": &hashed,
            "passwordResetTokenexpires": { "$gt": now_ms }
        })
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::bad_request("Token is invalid or has expired"))?;

    let hash = hash_password(&body.new_password).map_err(AppError::from)?;
    user.password = Some(hash);
    user.password_confirm = None;
    user.password_reset_token = None;
    user.password_reset_token_expires = None;
    user.touch_changed_password_at();

    let id = user.id.ok_or_else(|| AppError::internal("User missing _id."))?;
    users
        .replace_one(doc! { "_id": id }, &user)
        .await
        .map_err(AppError::from)?;

    let id_hex = id.to_hex();
    let jwt = sign_jwt(&id_hex, &state.config)?;
    let cookie = jwt_cookie(&jwt, &state);
    let jar = jar.add(cookie);

    Ok((
        jar,
        Json(json!({
            "status": "success",
            "token": jwt,
            "data": { "user": json_user(user) }
        })),
    ))
}

pub async fn update_password(
    State(state): State<AppState>,
    jar: CookieJar,
    Extension(ctx): Extension<User>,
    Json(body): Json<UpdatePasswordBody>,
) -> Result<impl IntoResponse, AppError> {
    validate_password(&body.password)?;
    if body.password != body.password_confirm {
        return Err(AppError::bad_request("Passwords are not the same!"));
    }

    let id = ctx
        .id
        .ok_or_else(|| AppError::unauthorized("Invalid session."))?;

    let db = state.db();
    let users = db.collection::<User>("users");

    let mut user = users
        .find_one(doc! { "_id": id })
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::unauthorized("The user belonging to this token does no longer exist."))?;

    if !user.verify_password(&body.password_current) {
        return Err(AppError::unauthorized("password galat hai"));
    }

    user.password = Some(hash_password(&body.password).map_err(AppError::from)?);
    user.password_confirm = None;
    user.touch_changed_password_at();

    users
        .replace_one(doc! { "_id": id }, &user)
        .await
        .map_err(AppError::from)?;

    let jwt = sign_jwt(&id.to_hex(), &state.config)?;
    let cookie = jwt_cookie(&jwt, &state);
    let jar = jar.add(cookie);

    Ok((
        jar,
        Json(json!({
            "status": "success",
            "token": jwt,
            "data": { "user": json_user(user) }
        })),
    ))
}
