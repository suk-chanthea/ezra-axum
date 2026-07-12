//! OTP handlers, mirroring Go `OTPHandler`.

use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;

use crate::error::AppResult;
use crate::interface::http::dto::request::{SendOtpRequest, VerifyOtpRequest};
use crate::interface::http::dto::ValidatedJson;
use crate::state::AppState;

pub async fn send_otp(
    State(state): State<AppState>,
    ValidatedJson(req): ValidatedJson<SendOtpRequest>,
) -> AppResult<impl IntoResponse> {
    let resp = state.otp.send_otp(req).await?;
    Ok(Json(resp))
}

pub async fn verify_otp(
    State(state): State<AppState>,
    ValidatedJson(req): ValidatedJson<VerifyOtpRequest>,
) -> AppResult<impl IntoResponse> {
    let resp = state.otp.verify_otp(req).await?;
    Ok(Json(resp))
}
