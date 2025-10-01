use serde_json::Value;

#[derive(Clone)]
pub struct AppState {
    pub client: reqwest::Client,
    pub start_time: std::time::Instant,
    pub gh_token: String,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            client: reqwest::Client::new(),
            start_time: std::time::Instant::now(),
            gh_token: std::env::var("GITHUB_TOKEN").unwrap_or_default(),
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
