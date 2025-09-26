use crate::routes::utils::AppState;
use axum::{Json, extract::State};
use chrono::{DateTime, Utc};
use serde_json::Value;

use std::sync::Arc;

#[derive(serde::Serialize, Clone)]
pub struct ContestResult {
    contest_id: i64,
    new_rating: i64,
    rank: i64,
    date_unix: i64,
}

#[derive(serde::Serialize)]
pub struct UserInfo {
    rating: i32,
    date_unix: i64,
}

#[derive(serde::Serialize)]
pub struct CodeforcesResponse {
    current_info: UserInfo,
    best_info: UserInfo,
    last_contest: ContestResult,
}

pub async fn codeforces_api_request(
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

pub async fn get_cf_stats() -> Json<CodeforcesResponse> {
    // Placeholder implementation
    let query_params: Vec<(&str, &str)> = vec![("handle", "ruizr274")];
    let response = codeforces_api_request(
        &reqwest::Client::new(),
        "https://codeforces.com/api/user.rating",
        "",
        query_params,
    )
    .await;
    let mut contests = vec![];
    for contest in response["result"].as_array().unwrap_or(&vec![]) {
        contests.push(ContestResult {
            contest_id: contest["contestId"].as_i64().unwrap_or(0),
            new_rating: contest["newRating"].as_i64().unwrap_or(0),
            rank: contest["rank"].as_i64().unwrap_or(0),
            date_unix: contest["ratingUpdateTimeSeconds"].as_i64().unwrap_or(0),
        });
    }
    let current_info = UserInfo {
        rating: contests.last().map_or(0, |c| c.new_rating as i32),
        date_unix: contests.last().map_or(0, |c| c.date_unix),
    };
    let best_info = contests.iter().max_by_key(|c| c.new_rating).map_or(
        UserInfo {
            rating: 0,
            date_unix: 0,
        },
        |c| UserInfo {
            rating: c.new_rating as i32,
            date_unix: c.date_unix,
        },
    );

    let response = CodeforcesResponse {
        current_info: current_info,
        best_info: best_info,
        last_contest: contests.last().cloned().unwrap_or(ContestResult {
            contest_id: 0,
            new_rating: 0,
            rank: 0,
            date_unix: 0,
        }),
    };
    Json(response)
}
