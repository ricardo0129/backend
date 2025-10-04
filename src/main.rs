use axum::response::Json;
use axum::{Router, routing::get};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};

pub mod routes;
use crate::routes::general_routes::{handler, handler_404, health_check, uptime};
use crate::routes::{codeforces, github, leetcode, utils};
use tokio_postgres::{Error, NoTls};
use utils::AppState;

#[derive(serde::Serialize)]
struct UserInfo {
    id: String,
    name: String,
}

pub async fn fetch_user_info() -> Json<Vec<UserInfo>> {
    // Connect to the database.
    let host: String = std::env::var("DB_HOST").unwrap_or_default();
    let user: String = std::env::var("DB_USER").unwrap_or_default();
    let password: String = std::env::var("DB_PASSWORD").unwrap_or_default();
    let db_url = format!("host={} user={} password={}", host, user, password);

    let (client, connection) = tokio_postgres::connect(&db_url, NoTls).await.unwrap();

    // The connection object performs the actual communication with the database,
    // so spawn it off to run on its own.
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    // Now we can execute a simple statement that just returns its parameter.
    let rows = client.query("SELECT * from userinfo;", &[]).await.unwrap();

    let mut user_info_list = Vec::new();
    for row in rows {
        let id: String = row.get(0);
        let name: String = row.get(1);
        user_info_list.push(UserInfo { id, name });
    }

    Json(user_info_list)
}

fn init_app() -> Router {
    /*
    Initialize the application state and routes.
    */
    let state = Arc::new(AppState::new());
    let app = Router::new()
        .route("/", get(handler))
        .route("/health", get(health_check))
        .route("/github", get(github::github_request))
        .route("/uptime", get(uptime))
        .route("/codeforces", get(codeforces::get_cf_stats))
        .route("/leetcode", get(leetcode::get_lc_stats))
        .route("/userinfo", get(fetch_user_info))
        .with_state(state);

    app.fallback(handler_404)
}

#[tokio::main]
async fn main() {
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(1)
            .burst_size(60)
            .finish()
            .unwrap(),
    );
    let governor_limiter = governor_conf.limiter().clone();
    let interval = Duration::from_secs(1);
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(interval);
            tracing::info!("rate limiting storage size: {}", governor_limiter.len());
            governor_limiter.retain_recent();
        }
    });
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::debug!("listening on {}", addr);
    let app = init_app().layer(ServiceBuilder::new().layer(GovernorLayer::new(governor_conf)));
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
