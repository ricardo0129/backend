use crate::routes::utils::AppState;
use axum::{Json, extract::State};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::Value;

use std::sync::Arc;

pub async fn leetcode_api_request(
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

#[derive(serde::Serialize, Debug, Deserialize)]
pub struct LeetCodeResponse {
    #[serde(rename = "attendedContestsCount")]
    attended_contests_count: i32,
    #[serde(rename = "rating")]
    rating: f64,
    #[serde(rename = "globalRanking")]
    global_ranking: i32,
    #[serde(rename = "totalParticipants")]
    total_participants: i32,
    #[serde(rename = "topPercentage")]
    top_percentage: f64,
}

pub async fn get_lc_stats() -> Json<LeetCodeResponse> {
    let url = "https://leetcode.com/graphql";
    let graphql_query = r#"{
        userProfile(username: "ricky0129") {
            username
            submitStats {
                acSubmissionNum {
                    difficulty
                    count
                }
            }
            profile {
                reputation
                ranking
            }
        }
    }"#;
    let graphql_query = r#"{
        userContestRanking(username:  "ricky0129") 
      {
        attendedContestsCount
        rating
        globalRanking
        totalParticipants
        topPercentage    
      }
    }"#;
    let query_params: Vec<(&str, &str)> = vec![("query", graphql_query)];
    let client = reqwest::Client::new();
    let bearer_auth = ""; // No auth needed for LeetCode public API
    let json_response = leetcode_api_request(&client, url, bearer_auth, query_params).await;
    let lc_response: LeetCodeResponse =
        serde_json::from_value(json_response["data"]["userContestRanking"].clone()).unwrap();
    Json(lc_response)
}
