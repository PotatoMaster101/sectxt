mod state;

use crate::state::{AppEnvironment, AppState};
use lambda_http::http::StatusCode;
use lambda_http::{Body, Error, Request, RequestPayloadExt, Response, run, service_fn, tracing};
use sectxt_core::crypto::hash_data;
use sectxt_core::message::{Message, MessageWithAttachments};
use sectxt_db::repo::PostgresMessageRepo;
use sectxt_shared::message::{MessageWithAttachmentsReadDto, MessageWithAttachmentsWriteDto, MessageWriteDto};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;

#[inline]
fn init_tracing() {
    tracing_subscriber::fmt()
        .json()
        .with_max_level(tracing::Level::INFO)
        .with_current_span(false)
        .with_ansi(false)
        .without_time()
        .with_target(false)
        .init();
}

#[inline]
fn get_create_payload(req: &Request) -> Result<Message, Error> {
    req.payload::<MessageWriteDto>()
        .map_err(|e| Error::from(format!("bad json: {e}")))?
        .ok_or("bad payload")?
        .try_into()
        .map_err(Error::from)
}

#[inline]
fn get_consume_payload(req: &Request) -> Result<MessageWithAttachmentsWriteDto, Error> {
    req.payload::<MessageWithAttachmentsWriteDto>()
        .map_err(|e| Error::from(format!("bad json: {e}")))?
        .ok_or_else(|| Error::from("bad payload"))
}

#[inline]
fn build_response(status: StatusCode, message: &str) -> Response<Body> {
    Response::builder()
        .status(status)
        .body(Body::from(message))
        .expect("failed to build response")
}

#[inline]
async fn get_db_pool(db_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(1)
        .min_connections(0)
        .idle_timeout(std::time::Duration::from_secs(10))
        .connect(db_url)
        .await
}

async fn handle_create(req: &Request, state: Arc<AppState>) -> Response<Body> {
    let dto = match get_create_payload(req) {
        Ok(payload) => payload,
        Err(e) => return build_response(StatusCode::BAD_REQUEST, &e.to_string()),
    };

    state
        .repo()
        .create(MessageWithAttachments::new(dto, []))
        .await
        .map_or_else(
            |e| build_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            |id| build_response(StatusCode::OK, &id.to_string()),
        )
}

async fn handle_get(req: &Request, state: Arc<AppState>) -> Response<Body> {
    let dto = match get_consume_payload(req) {
        Ok(payload) => payload,
        Err(e) => return build_response(StatusCode::BAD_REQUEST, &e.to_string()),
    };

    let auth_hash = hash_data(&dto.auth_token);
    let consume = state.repo().consume(dto.id, auth_hash).await;
    let mwa_opt = match consume {
        Ok(opt) => opt,
        Err(e) => return build_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    mwa_opt.map_or_else(
        || build_response(StatusCode::NOT_FOUND, "not found"),
        |mwa| {
            let read_dto = MessageWithAttachmentsReadDto::from(mwa);
            match serde_json::to_string(&read_dto) {
                Ok(json) => build_response(StatusCode::OK, &json),
                Err(e) => build_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        },
    )
}

async fn func_handler(req: Request, state: Arc<AppState>) -> Response<Body> {
    let path = req.uri().path();
    match path {
        "/message/create" => handle_create(&req, state.clone()).await,
        "/message/consume" => handle_get(&req, state.clone()).await,
        _ => build_response(StatusCode::NOT_FOUND, "not found"),
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    init_tracing();
    let env = AppEnvironment::from_env()?;
    let pool = get_db_pool(env.database_url()).await?;
    let repo = PostgresMessageRepo::new(pool.clone());
    let state = Arc::new(AppState::new(Box::new(repo)));
    run(service_fn(move |req| {
        let state = state.clone();
        async move { Ok::<_, std::convert::Infallible>(func_handler(req, state).await) }
    }))
    .await
}
