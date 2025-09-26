use crate::routes::utils::AppState;
use axum::{Json, extract::State};
use serde_json::Value;

use std::sync::Arc;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct GithubResponse {
    name: String,
    html_url: String,
    pushed_at: String,
}
pub async fn github_api_request(
    client: &reqwest::Client,
    url: &str,
    bearer_auth: &str,
    query_params: Vec<(&str, &str)>,
) -> Value {
    let response = client
        .get(url)
        .header("User-Agent", "rust-github-client") // Required by GitHub API
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
                println!("GitHub API request failed with status: {}", resp.status());
                Value::Null
            }
        }
        Err(e) => {
            println!("Error making request: {}", e);
            Value::Null
        }
    }
}

pub async fn github_request(State(state): State<Arc<AppState>>) -> Json<Vec<GithubResponse>> {
    let query_params: Vec<(&str, &str)> = vec![("sort", "updated")];
    let response = github_api_request(
        &state.client,
        "https://api.github.com/user/repos",
        &state.gh_token,
        query_params,
    )
    .await;
    if response.is_null() {
        return Json(vec![]);
    }

    let mut repos: Vec<GithubResponse> = vec![];
    for repo in response.as_array().unwrap() {
        println!("last pushed at: {}", repo["pushed_at"].as_str().unwrap());
        println!("Repo Name: {}", repo["name"].as_str().unwrap());
        println!("Repo URL: {}", repo["html_url"].as_str().unwrap());
        repos.push(GithubResponse {
            name: repo["name"].as_str().unwrap().to_string(),
            html_url: repo["html_url"].as_str().unwrap().to_string(),
            pushed_at: repo["pushed_at"].as_str().unwrap().to_string(),
        });
        if repos.len() >= 3 {
            break;
        }
    }
    Json(repos)
}

pub async fn github_last_commit(State(state): State<Arc<AppState>>) -> Json<Value> {
    let response = github_api_request(
        &state.client,
        "https://api.github.com/user",
        &state.gh_token,
        vec![],
    )
    .await;

    Json(response)
}
