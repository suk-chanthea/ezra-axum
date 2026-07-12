//! Donation handlers, mirroring Go `DonationHandler`.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use serde_json::json;

use crate::error::{AppError, AppResult};
use crate::interface::http::dto::request::{
    CreateDonationRequest, DonationFilterQuery, LimitOffsetQuery, UpdateDonationStatusRequest,
};
use crate::interface::http::dto::response::{PaginatedResponse, SuccessResponse};
use crate::interface::http::dto::ValidatedJson;
use crate::interface::http::middleware::AuthUser;
use crate::state::AppState;

pub async fn create(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    ValidatedJson(req): ValidatedJson<CreateDonationRequest>,
) -> AppResult<impl IntoResponse> {
    if req.donor_type == "company" && (req.company_name.is_empty() || req.company_email.is_empty()) {
        return Err(AppError::BadRequest(
            "company name and email are required for company donations".to_string(),
        ));
    }

    let initiate_payment = req.initiate_payment;
    let donation = state.donation.create_donation(req, Some(user_id)).await?;

    let mut message = "donation created successfully".to_string();
    if initiate_payment && donation.payment_info.is_some() {
        message = if donation.r#type == "donate" {
            "donation created successfully. Please scan the QR code to complete payment".to_string()
        } else {
            "donation created successfully. Please complete payment via the provided URL".to_string()
        };
    }

    let data = serde_json::to_value(donation).map_err(|e| AppError::Internal(e.to_string()))?;
    Ok((StatusCode::CREATED, Json(SuccessResponse::with_data(&message, data))))
}

pub async fn get_all(
    State(state): State<AppState>,
    Query(filter): Query<DonationFilterQuery>,
) -> AppResult<impl IntoResponse> {
    let (data, meta) = state.donation.get_all_donations(&filter).await?;
    Ok(Json(PaginatedResponse { data, pagination: meta }))
}

pub async fn get_by_id(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<impl IntoResponse> {
    let donation = state
        .donation
        .get_donation_by_id(id)
        .await
        .map_err(|_| AppError::NotFound("donation not found".to_string()))?;
    Ok(Json(donation))
}

pub async fn get_by_user(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Query(lo): Query<LimitOffsetQuery>,
) -> AppResult<impl IntoResponse> {
    let donations = state.donation.get_donations_by_user_id(user_id, lo.limit, lo.offset).await?;
    Ok(Json(donations))
}

pub async fn get_by_type(
    State(state): State<AppState>,
    Path(donation_type): Path<String>,
    Query(lo): Query<LimitOffsetQuery>,
) -> AppResult<impl IntoResponse> {
    if donation_type != "donate" && donation_type != "sponsor" {
        return Err(AppError::BadRequest(
            "invalid donation type, must be 'donate' or 'sponsor'".to_string(),
        ));
    }
    let donations = state.donation.get_donations_by_type(&donation_type, lo.limit, lo.offset).await?;
    Ok(Json(donations))
}

pub async fn get_by_event(
    State(state): State<AppState>,
    Path(event_id): Path<i64>,
    Query(lo): Query<LimitOffsetQuery>,
) -> AppResult<impl IntoResponse> {
    let donations = state.donation.get_donations_by_event_id(event_id, lo.limit, lo.offset).await?;
    Ok(Json(donations))
}

pub async fn update_status(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    ValidatedJson(req): ValidatedJson<UpdateDonationStatusRequest>,
) -> AppResult<impl IntoResponse> {
    state.donation.update_donation_status(id, req).await?;
    Ok(Json(SuccessResponse::message("donation status updated successfully")))
}

pub async fn delete(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<i64>,
) -> AppResult<impl IntoResponse> {
    state.donation.delete_donation(id, user_id).await?;
    Ok(Json(SuccessResponse::message("donation deleted successfully")))
}

pub async fn get_stats(State(state): State<AppState>) -> AppResult<impl IntoResponse> {
    let stats = state.donation.get_donation_stats().await?;
    Ok(Json(stats))
}

pub async fn get_stats_by_event(
    State(state): State<AppState>,
    Path(event_id): Path<i64>,
) -> AppResult<impl IntoResponse> {
    let stats = state.donation.get_donation_stats_by_event_id(event_id).await?;
    Ok(Json(stats))
}

pub async fn initiate_payment(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<impl IntoResponse> {
    let resp = state.donation.initiate_payment(id).await?;
    Ok(Json(resp))
}

pub async fn check_qr_status(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<impl IntoResponse> {
    let donation = state
        .donation
        .get_donation_by_id(id)
        .await
        .map_err(|_| AppError::NotFound("donation not found".to_string()))?;

    if donation.r#type != "donate" {
        return Err(AppError::BadRequest("not a QR payment".to_string()));
    }

    Ok(Json(json!({
        "donation_id": donation.id,
        "status": donation.status,
        "expired": donation.status == "pending",
    })))
}

pub async fn regenerate_qr(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<impl IntoResponse> {
    let resp = state.donation.initiate_payment(id).await?;
    let data = serde_json::to_value(resp).map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(SuccessResponse::with_data("QR code regenerated successfully", data)))
}

#[derive(Debug, Deserialize)]
pub struct PaywayWebhookRequest {
    #[serde(default, rename = "tran_id")]
    pub transaction_id: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub approval_code: String,
    #[serde(default, rename = "payment_option")]
    pub payment_method: String,
    #[serde(default)]
    pub hash: String,
}

pub async fn handle_payway_webhook(
    State(state): State<AppState>,
    Json(webhook): Json<PaywayWebhookRequest>,
) -> AppResult<impl IntoResponse> {
    let _ = webhook.hash;
    state
        .donation
        .handle_payment_callback(
            &webhook.transaction_id,
            &webhook.status,
            &webhook.approval_code,
            &webhook.payment_method,
        )
        .await?;
    Ok(Json(SuccessResponse::message("webhook processed successfully")))
}
