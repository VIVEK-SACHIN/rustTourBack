

mod config;
mod models;

use axum::{
    routing::{get},
    Router,
};
use bson::doc;
use config::AppConfig;
use mongodb::Client;

#[axum::debug_handler]
async fn hello_world(message: String) -> String {
    message
}


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
    let addr = app_config.address();

    let db_uri = app_config.resolved_database_url();
    let client = Client::with_uri_str(&db_uri)
        .await
        .expect("Failed to create MongoDB client");

    let admin_db = client.database("admin");
    admin_db
        .run_command(doc! { "ping": 1 })
        .await
        .expect("Failed to ping MongoDB");

    println!("✅ Connected to MongoDB successfully");
    println!("🚀 Starting server on http://{}", addr);

    let app: Router = Router::new().route(
        "/",
        get(move || async move { hello_world(app_config.hello_message.clone()).await }),
    );
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
