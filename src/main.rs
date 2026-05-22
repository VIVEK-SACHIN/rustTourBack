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
    routing::post,
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
use handlers::bookings::webhook_checkout;
use routes::{
    booking_routes::booking_routes,
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
fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        eprintln!("UNCAUGHT PANIC! Shutting down...");
        eprintln!("{info}");
    }));
}

#[tokio::main]
async fn main() {
    install_panic_hook();

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
        .merge(booking_routes(&app_state))
        .route_layer(axum_middleware::from_fn_with_state(
            rate_limiter.clone(),
            middleware::rate_limit::rate_limit_middleware,
        ));

    // Stripe webhook needs the raw body (Natours mounts this before `express.json`).
    let webhook = Router::new()
        .route("/webhook-checkout", post(webhook_checkout))
        .with_state(app_state.clone());

    let cors = CorsLayer::new()
        .allow_origin([
            "http://localhost:3000".parse::<HeaderValue>().unwrap(),
            "http://localhost:5173".parse::<HeaderValue>().unwrap(),
            "https://localhost:5173".parse::<HeaderValue>().unwrap(),
        ])
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            header::ACCEPT,
            header::COOKIE,
        ])
        .allow_credentials(true);

    let app: Router = Router::new()
        .merge(webhook)
        .nest("/api/v1", api_v1)
        .fallback(handle_not_found)
        .with_state(app_state)
        .layer(CompressionLayer::new())
        .layer(RequestBodyLimitLayer::new(10 * 1024))
        .layer(axum_middleware::from_fn(
            middleware::mongo_sanitize::mongo_sanitize_middleware,
        ))
        .layer(axum_middleware::from_fn(middleware::hpp::hpp_middleware))
        .layer(axum_middleware::from_fn(
            middleware::security_headers::security_headers_middleware,
        ))
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
    println!("App running on {addr}...");

    let server = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal());

    if let Err(err) = server.await {
        eprintln!("Server error: {err}");
    } else {
        println!("Server shut down gracefully.");
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(err) = tokio::signal::ctrl_c().await {
            eprintln!("Failed to listen for ctrl-c: {err}");
        }
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{signal, SignalKind};
        if let Ok(mut sig) = signal(SignalKind::terminate()) {
            let _ = sig.recv().await;
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => eprintln!("SIGINT received. Shutting down gracefully..."),
        _ = terminate => eprintln!("SIGTERM received. Shutting down gracefully..."),
    }
}
