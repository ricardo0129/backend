use crate::routes::utils::api_request;
use axum::Json;
use serde::Deserialize;

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
    let json_response = api_request(&client, url, bearer_auth, query_params).await;
    let lc_response: LeetCodeResponse =
        serde_json::from_value(json_response["data"]["userContestRanking"].clone()).unwrap();
    Json(lc_response)
}
