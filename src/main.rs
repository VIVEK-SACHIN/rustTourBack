mod config;
mod db;
mod jwt_util;
mod state;
mod utils;
mod handlers;
mod middleware;
mod models;
mod routes;

use std::sync::Arc;

use axum::{
    http::StatusCode,
    middleware as axum_middleware,
    response::IntoResponse,
    Router,
};
use config::AppConfig;
use db::mongodb::create_mongo_client;
use state::AppState;
use tower_http::catch_panic::CatchPanicLayer;
use utils::error::{init_error_reporting, panic_response_json, AppError};
use routes::{
    auth_routes::auth_routes,
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

    let client = match create_mongo_client(&app_config).await {
        Ok(client) => client,
        Err(err) => {
            panic!("Failed to create MongoDB client: {}", err);
        }
    };

    let app_state = AppState {
        client,
        config: Arc::new(app_config),
    };

    let app: Router = Router::new()
        .merge(tour_routes())
        .merge(user_routes())
        .merge(auth_routes(&app_state))
        .fallback(handle_not_found)
        .with_state(app_state)
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
    axum::serve(listener, app).await.unwrap();
}
