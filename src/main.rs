mod config;
mod db;
mod handlers;
mod jwt_util;
mod middleware;
mod models;
mod routes;
mod services;
mod state;
mod utils;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    http::{header, HeaderValue, Method, StatusCode},
    middleware as axum_middleware,
    response::IntoResponse,
    Router,
};
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    limit::RequestBodyLimitLayer,
};
use config::AppConfig;
use db::mongodb::create_mongo_client;
use state::AppState;
use tower_http::catch_panic::CatchPanicLayer;
use utils::error::{init_error_reporting, panic_response_json, AppError};
use routes::{
    review_routes::review_routes,
    tour_routes::tour_routes,
    user_routes::user_routes,
};
//is an attribute macro.It helps give better compiler errors for Axum handlers.
// Without it, Axum errors can be huge and confusing.
// #[axum::debug_handler]
// async fn hello_world(message: String) -> String {
//     message
// }

// this is a procedural macro from the Tokio crate (an asynchronous runtime for Rust).
//  It serves the following purposes:
//  1.Enables async main: It allows your main function to be declared as async fn main() instead of requiring manual runtime setup.

// 2.Automatic runtime initialization: Behind the scenes, it creates a Tokio runtime, runs your async code within it, and handles the event loop.

// 3.Simplifies async code: Without this macro, you'd need to manually create a runtime like:
// fn main() {
//     let rt = tokio::runtime::Runtime::new().unwrap();
//     rt.block_on(async {
//         // your async code here
//     });
// }
// 4.Error handling: It propagates any errors from the async function as the program's exit code.
#[tokio::main]
async fn main() {
    let app_config = AppConfig::from_env();
    init_error_reporting(app_config.is_production());
    let addr = app_config.address();

    let client = create_mongo_client(&app_config).await.unwrap_or_else(|err| {
        eprintln!("Fatal: could not connect to MongoDB: {err}");
        std::process::exit(1);
    });

    let app_state = AppState {
        client,
        config: Arc::new(app_config),
    };

    let rate_limiter = middleware::rate_limit::build_api_rate_limiter();

    let api_v1 = Router::new()
        .merge(tour_routes(&app_state))
        .merge(user_routes(&app_state))
        .merge(review_routes(&app_state))
        .route_layer(axum_middleware::from_fn_with_state(
            rate_limiter.clone(),
            middleware::rate_limit::rate_limit_middleware,
        ));

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            header::ACCEPT,
            header::COOKIE,
        ])
        .allow_credentials(true);

    let app: Router = Router::new()
        .nest("/api/v1", api_v1)
        .fallback(handle_not_found)
        .with_state(app_state)
        .layer(CompressionLayer::new())
        .layer(RequestBodyLimitLayer::new(10 * 1024))
        .layer(cors)
        .layer(axum_middleware::from_fn(
            middleware::request_logger_middleware,
        ))
        .layer(CatchPanicLayer::custom(|_: Box<dyn std::any::Any + Send>| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                panic_response_json(),
            )
                .into_response()
        }));

    async fn handle_not_found(uri: axum::http::Uri) -> AppError {
        AppError::not_found(format!("Cannot find {} on this server", uri.path()))
    }

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
