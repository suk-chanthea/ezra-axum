//! Chat handlers: private/group/band conversations, membership/permissions, and messages.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use crate::error::AppResult;
use crate::interface::http::dto::request::{
    AddConversationMemberRequest, CreateConversationRequest, PaginationQuery, SendChatMessageRequest,
    UpdateConversationMemberRequest,
};
use crate::interface::http::dto::response::{PaginatedResponse, SuccessResponse};
use crate::interface::http::dto::ValidatedJson;
use crate::interface::http::middleware::AuthUser;
use crate::state::AppState;

// ---- Conversations ----

pub async fn create_conversation(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    ValidatedJson(req): ValidatedJson<CreateConversationRequest>,
) -> AppResult<impl IntoResponse> {
    let conversation = state.chat.create_conversation(user_id, req).await?;
    Ok((StatusCode::CREATED, Json(conversation)))
}

pub async fn get_conversations(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> AppResult<impl IntoResponse> {
    let conversations = state.chat.get_conversations(user_id).await?;
    Ok(Json(conversations))
}

pub async fn get_conversation(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<i64>,
) -> AppResult<impl IntoResponse> {
    let conversation = state.chat.get_conversation(id, user_id).await?;
    Ok(Json(conversation))
}

pub async fn delete_conversation(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<i64>,
) -> AppResult<impl IntoResponse> {
    state.chat.delete_conversation(id, user_id).await?;
    Ok(Json(SuccessResponse::message("conversation deleted successfully")))
}

// ---- Members / permissions ----

pub async fn get_members(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<i64>,
) -> AppResult<impl IntoResponse> {
    let members = state.chat.get_members(id, user_id).await?;
    Ok(Json(members))
}

pub async fn add_member(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<i64>,
    ValidatedJson(req): ValidatedJson<AddConversationMemberRequest>,
) -> AppResult<impl IntoResponse> {
    let conversation = state.chat.add_member(id, user_id, req).await?;
    Ok((StatusCode::CREATED, Json(conversation)))
}

pub async fn update_member(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path((id, member_id)): Path<(i64, i64)>,
    ValidatedJson(req): ValidatedJson<UpdateConversationMemberRequest>,
) -> AppResult<impl IntoResponse> {
    let member = state.chat.update_member(id, user_id, member_id, req).await?;
    Ok(Json(member))
}

pub async fn remove_member(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path((id, member_id)): Path<(i64, i64)>,
) -> AppResult<impl IntoResponse> {
    state.chat.remove_member(id, user_id, member_id).await?;
    Ok(Json(SuccessResponse::message("member removed successfully")))
}

// ---- Messages ----

pub async fn send_message(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<i64>,
    ValidatedJson(req): ValidatedJson<SendChatMessageRequest>,
) -> AppResult<impl IntoResponse> {
    let message = state.chat.send_message(id, user_id, req).await?;
    Ok((StatusCode::CREATED, Json(message)))
}

pub async fn get_messages(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<i64>,
    Query(pagination): Query<PaginationQuery>,
) -> AppResult<impl IntoResponse> {
    let (data, meta) = state
        .chat
        .get_messages(id, user_id, pagination.get_page(), pagination.get_page_size())
        .await?;
    Ok(Json(PaginatedResponse { data, pagination: meta }))
}

pub async fn get_message(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path((id, message_id)): Path<(i64, i64)>,
) -> AppResult<impl IntoResponse> {
    let message = state.chat.get_message(id, message_id, user_id).await?;
    Ok(Json(message))
}

pub async fn delete_message(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path((id, message_id)): Path<(i64, i64)>,
) -> AppResult<impl IntoResponse> {
    state.chat.delete_message(id, message_id, user_id).await?;
    Ok(Json(SuccessResponse::message("message deleted successfully")))
}
