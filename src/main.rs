use axum::{Router, routing::get};
use std::fmt;
use std::net::SocketAddr;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;
use std::{future::Future, pin::Pin};
use tokio::net::TcpListener;
use tower::Service;
use tower::ServiceBuilder;
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};

use std::env;

pub mod routes;
use crate::routes::general_routes::{handler, handler_404, health_check, uptime};
use crate::routes::{codeforces, github, leetcode, utils};
use pin_project::pin_project;
use tokio::time::Sleep;
use tokio::time::sleep;
use utils::AppState;

#[derive(Debug, Default)]
pub struct TimeoutError(());

impl fmt::Display for TimeoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad("request timed out")
    }
}

impl std::error::Error for TimeoutError {}
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

#[pin_project]
pub struct ResponseFuture<F> {
    #[pin]
    response_future: F,
    #[pin]
    sleep: Sleep,
}

#[derive(Clone, Debug)]
struct Timeout<S> {
    inner: S,
    timeout: Duration,
}

impl<S> Timeout<S> {
    pub fn new(inner: S, timeout: Duration) -> Self {
        Timeout { inner, timeout }
    }
}

impl<F, Response, Error> Future for ResponseFuture<F>
where
    F: Future<Output = Result<Response, Error>>,
    Error: Into<BoxError>,
{
    type Output = Result<Response, BoxError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.response_future.poll(cx) {
            Poll::Ready(result) => {
                let result = result.map_err(Into::into);
                return Poll::Ready(result);
            }
            Poll::Pending => {}
        }
        match this.sleep.poll(cx) {
            Poll::Ready(()) => {
                let error = Box::new(TimeoutError(()));
                return Poll::Ready(Err(error));
            }
            Poll::Pending => {}
        }
        Poll::Pending
    }
}

impl<S, Request> Service<Request> for Timeout<S>
where
    S: Service<Request>,
    S::Error: Into<BoxError>,
{
    type Response = S::Response;
    type Error = BoxError;
    type Future = ResponseFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        //
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        let response_future = self.inner.call(request);

        let sleep = tokio::time::sleep(self.timeout);
        ResponseFuture {
            response_future,
            sleep,
        }
    }
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
