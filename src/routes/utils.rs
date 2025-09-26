#[derive(Clone)]
pub struct AppState {
    pub client: reqwest::Client,
    pub start_time: std::time::Instant,
    pub gh_token: String,
}
