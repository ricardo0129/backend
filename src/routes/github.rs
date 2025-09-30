use crate::routes::utils::{AppState, api_request};
use axum::{Json, extract::State};
use serde_json::Value;

use std::sync::Arc;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct GithubResponse {
    name: String,
    html_url: String,
    pushed_at: String,
}

pub async fn github_request(State(state): State<Arc<AppState>>) -> Json<Vec<GithubResponse>> {
    let query_params: Vec<(&str, &str)> = vec![("sort", "updated")];
    let response = api_request(
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
    let response = api_request(
        &state.client,
        "https://api.github.com/user",
        &state.gh_token,
        vec![],
    )
    .await;

    Json(response)
}
