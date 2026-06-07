mod dto;
mod handler;
mod state;
mod worker;

use crate::state::{AppEnvironment, AppState};
use axum::Router;
use axum::routing::{get, post};
use sectxt_db::message::repo::PgMessageRepo;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, HttpMakeClassifier, TraceLayer};
use tracing::Level;

fn get_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/message/{id}", get(handler::read_message))
        .route("/message", post(handler::create_message))
        .layer(get_trace_layer())
        .with_state(state)
}

async fn get_pg_pool(db_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(20)
        .idle_timeout(std::time::Duration::from_secs(30))
        .connect(db_url)
        .await
}

fn get_trace_layer() -> TraceLayer<HttpMakeClassifier> {
    TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
        .on_response(DefaultOnResponse::new().level(Level::INFO))
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();
    let env = AppEnvironment::from_env().unwrap();
    let db_pool = get_pg_pool(&env.database_url).await.unwrap();
    let message_repo = PgMessageRepo::new(db_pool);
    let state = Arc::new(AppState::new(Box::new(message_repo)));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();

    worker::clean_expired_messages(state.clone(), 10);
    tracing::info!("Listening on http://0.0.0.0:8080");
    axum::serve(listener, get_router(state)).await.unwrap();
}
