use crate::routes::utils::api_request;
use axum::Json;

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

pub async fn get_cf_stats() -> Json<CodeforcesResponse> {
    // Placeholder implementation
    let query_params: Vec<(&str, &str)> = vec![("handle", "ruizr274")];
    let response = api_request(
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
