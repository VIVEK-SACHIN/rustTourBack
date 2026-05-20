use std::process::Stdio;

use bson::doc;
use mongodb::Client;
use tokio::process::Command;

use crate::config::AppConfig;

/// Same as Node `server.js`: `DATABASE` with `<password>` replaced.
fn resolve_database_url(app_config: &AppConfig) -> (String, &'static str) {
    if !app_config.database_direct.is_empty() {
        return (
            app_config
                .database_direct
                .replace("<password>", &app_config.database_password),
            "DATABASE_DIRECT",
        );
    }
    if !app_config.database.is_empty() {
        return (
            app_config
                .database
                .replace("<password>", &app_config.database_password),
            "DATABASE",
        );
    }
    (
        app_config.database_local.clone(),
        "DATABASE_LOCAL",
    )
}

fn is_srv_local_dns_bug(err: &mongodb::error::Error) -> bool {
    let msg = err.to_string();
    msg.contains(".mongodb.net.local") || (msg.contains("_mongodb._tcp") && msg.contains(".local"))
}

/// Extract `vivekcluster.s9yxoy6.mongodb.net` from `mongodb+srv://user:pass@HOST/...`
fn srv_hostname(srv_uri: &str) -> Option<String> {
    let rest = srv_uri.strip_prefix("mongodb+srv://")?;
    let after_at = rest.split('@').nth(1)?;
    let host = after_at
        .split(&['/', '?', '#'][..])
        .next()?
        .trim();
    if host.is_empty() {
        None
    } else {
        Some(host.to_string())
    }
}

/// macOS + Rust Hickory DNS often appends `.local` to SRV queries. `dig` works; use it to
/// build a standard `mongodb://` URI to the **same Atlas shard hosts** (still Atlas, not local).
async fn atlas_direct_from_srv_via_dig(srv_uri: &str) -> Option<String> {
    let cluster_host = srv_hostname(srv_uri)?;
    let output = Command::new("dig")
        .args(["+short", "SRV", &format!("_mongodb._tcp.{cluster_host}")])
        .stdout(Stdio::piped())
        .output()
        .await
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut shard_hosts = Vec::new();
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        // `0 0 27017 ac-sutfyaa-shard-00-00.s9yxoy6.mongodb.net.`
        let mut parts = line.split_whitespace();
        let _priority = parts.next()?;
        let _weight = parts.next()?;
        let port = parts.next().unwrap_or("27017");
        let target = parts.next()?.trim_end_matches('.');
        shard_hosts.push(format!("{target}:{port}"));
    }

    if shard_hosts.is_empty() {
        return None;
    }

    let rest = srv_uri.strip_prefix("mongodb+srv://")?;
    let userinfo = rest.split('@').next()?;

    // Preserve query exactly as in `.env` (case matters: retryWrites, not retrywrites).
    let mut query = srv_uri
        .split_once('?')
        .map(|(_, q)| q.to_string())
        .unwrap_or_default();
    if !query.is_empty() && !query.contains("tls=") {
        query.push_str("&tls=true");
    }
    if !query.is_empty() && !query.contains("authSource=") {
        query.push_str("&authSource=admin");
    }
    let q_suffix = if query.is_empty() {
        "?tls=true&authSource=admin".to_string()
    } else {
        format!("?{query}")
    };

    Some(format!(
        "mongodb://{userinfo}@{hosts}{q_suffix}",
        hosts = shard_hosts.join(","),
    ))
}

pub async fn create_mongo_client(app_config: &AppConfig) -> Result<Client, mongodb::error::Error> {
    let (db_uri, source) = resolve_database_url(app_config);
    eprintln!("ℹ️  MongoDB connection from {source}");

    match try_connect(&db_uri).await {
        Ok(client) => return Ok(client),
        Err(err) if db_uri.starts_with("mongodb+srv://") && is_srv_local_dns_bug(&err) => {
            eprintln!(
                "⚠️  SRV DNS failed (.local suffix — common on macOS with the Rust driver). Resolving Atlas via `dig`..."
            );
            if let Some(direct) = atlas_direct_from_srv_via_dig(&db_uri).await {
                eprintln!("ℹ️  Connecting to Atlas shard hosts (standard mongodb://, same cluster)");
                return try_connect(&direct).await;
            }
            eprintln!("❌ Could not build Atlas fallback URI from `dig`.");
            eprintln!("   Fix: Atlas → Connect → copy **standard** `mongodb://` URI into DATABASE_DIRECT in .env");
            Err(err)
        }
        Err(err) => Err(err),
    }
}

async fn try_connect(db_uri: &str) -> Result<Client, mongodb::error::Error> {
    let client = Client::with_uri_str(db_uri).await?;
    println!("✅ Successfully connected to MongoDB");
    let admin_db = client.database("admin");
    admin_db.run_command(doc! { "ping": 1 }).await?;
    Ok(client)
}
