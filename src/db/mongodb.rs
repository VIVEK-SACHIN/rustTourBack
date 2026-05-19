use bson::doc;
use mongodb::Client;

use crate::config::AppConfig;

fn resolve_database_url(app_config: &AppConfig) -> String {
    if !app_config.database.is_empty() {
        app_config.database.replace("<password>", &app_config.database_password)
    } else {
        app_config.database_local.clone()
    }
}

pub async fn create_mongo_client(app_config: &AppConfig) -> Result<Client, mongodb::error::Error> {
    let db_uri = resolve_database_url(app_config);
    let client = match Client::with_uri_str(&db_uri).await{
        Ok(client) => {
            println!("✅ Successfully connected to MongoDB");
            client
        },
        Err(err) => {
            eprintln!("❌ Failed to connect to MongoDB: {}", err);
            return Err(err);
        }
    };

    let admin_db = client.database("admin");
    admin_db.run_command(doc! { "ping": 1 }).await?;

    Ok(client)
}
