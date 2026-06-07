use crate::dto::{CreateMessageDto, ReadMessageDto};
use crate::state::AppState;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use sectxt_core::message::Message;
use sqlx::types::Uuid;
use std::sync::Arc;

pub enum HandlerError {
    BadRequest(String),
    InternalServer(String),
    NotFound(String),
}

impl IntoResponse for HandlerError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::BadRequest(message) => {
                tracing::warn!(error = %message, "bad request");
                (StatusCode::BAD_REQUEST, Json(message))
            },
            Self::InternalServer(message) => {
                tracing::error!(error = %message, "internal server error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json("Internal Server Error".to_string()),
                )
            },
            Self::NotFound(message) => {
                tracing::warn!(error = %message, "not found");
                (StatusCode::NOT_FOUND, Json(message))
            },
        }
        .into_response()
    }
}

pub async fn create_message(
    State(state): State<Arc<AppState>>,
    Json(dto): Json<CreateMessageDto>,
) -> Result<impl IntoResponse, HandlerError> {
    let message = Message::try_from(dto).map_err(|e| HandlerError::BadRequest(e.to_string()))?;
    let id = state
        .message_repo
        .create(message)
        .await
        .map_err(|e| HandlerError::InternalServer(e.to_string()))?;

    Ok((StatusCode::CREATED, Json(id)))
}

pub async fn read_message(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    let message = state
        .message_repo
        .get(id)
        .await
        .map_err(|e| HandlerError::InternalServer(e.to_string()))?
        .ok_or_else(|| HandlerError::NotFound("message not found".to_string()))?;

    Ok((StatusCode::OK, Json(ReadMessageDto::from(message))))
}
