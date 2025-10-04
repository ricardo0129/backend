use serde_json::Value;
use std::sync::Arc;
use tokio_postgres::{Client, NoTls};

#[derive(Clone)]
pub struct AppState {
    pub client: reqwest::Client,
    pub start_time: std::time::Instant,
    pub gh_token: String,
    pub db_client: Arc<Client>,
}

pub async fn init_db_client() -> tokio_postgres::Client {
    let host: String = std::env::var("DB_HOST").expect("DB_HOST must be set");
    let user: String = std::env::var("DB_USER").expect("DB_USER must be set");
    let password: String = std::env::var("DB_PASSWORD").expect("DB_PASSWORD must be set");
    let db_url = format!("host={} user={} password={}", host, user, password);

    let (client, connection) = tokio_postgres::connect(&db_url, NoTls).await.unwrap();

    // The connection object performs the actual communication with the database,
    // so spawn it off to run on its own.
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    client
}

impl AppState {
    pub async fn new() -> Self {
        let db_client = init_db_client().await;
        AppState {
            client: reqwest::Client::new(),
            start_time: std::time::Instant::now(),
            gh_token: std::env::var("GITHUB_TOKEN").unwrap_or_default(),
            db_client: Arc::new(db_client),
        }
    }
}

pub async fn api_request(
    client: &reqwest::Client,
    url: &str,
    bearer_auth: &str,
    query_params: Vec<(&str, &str)>,
) -> Value {
    let response = client
        .get(url)
        .header("User-Agent", "rust-cf-client") // Required by GitHub API
        .header("Accept", "application/vnd.github.v3+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .query(&query_params)
        .bearer_auth(bearer_auth)
        .send();
    match response.await {
        Ok(resp) => {
            if resp.status().is_success() {
                let json: Value = resp.json().await.unwrap();
                json
            } else {
                println!("{} request failed with status: {}", url, resp.status());
                Value::Null
            }
        }
        Err(e) => {
            println!("Error making request: {}", e);
            Value::Null
        }
    }
}
