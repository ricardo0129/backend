use crate::routes::utils::AppState;
use axum::Json;
use axum::{
    extract::ConnectInfo, extract::Path, extract::State, http::StatusCode, response::IntoResponse,
};
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Problem {
    id: i32,
    title: String,
    description: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Solution {
    id: i64,
    problem_id: i64,
    solution: String,
    language: String,
}

#[derive(serde::Deserialize)]
pub struct ContributePayload {
    problem_id: i32,
    addition: String,
    language: String,
}

pub async fn get_problems(State(state): State<Arc<AppState>>) -> Json<Vec<Problem>> {
    let mut problems = vec![]; // Fetch problems from database
    for row in state
        .db_client
        .query("SELECT id, title, description FROM problem", &[])
        .await
        .unwrap()
    {
        problems.push(Problem {
            id: row.get(0),
            title: row.get(1),
            description: row.get(2),
        });
    }
    Json(problems)
}

pub async fn get_solution(
    State(state): State<Arc<AppState>>,
    Path(prob_id): Path<i32>,
) -> Json<Solution> {
    Json(Solution {
        id: 1,
        problem_id: prob_id as i64,
        solution: "print('Hello, World!')".to_string(),
        language: "Python".to_string(),
    })
}

pub async fn contribute_solution(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ContributePayload>,
) -> impl IntoResponse {
    let q = "UPDATE solution SET code = code || $1 where id = $2 and language = $3";
    state
        .db_client
        .execute(
            q,
            &[&payload.addition, &payload.problem_id, &payload.language],
        )
        .await
        .unwrap();

    StatusCode::OK
}
