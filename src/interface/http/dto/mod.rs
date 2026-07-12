pub mod local_time;
pub mod request;
pub mod response;

use axum::extract::rejection::JsonRejection;
use axum::extract::FromRequest;
use axum::Json;
use validator::Validate;

use crate::error::AppError;

/// Converts validator errors into a single human-readable message, matching the
/// "field is invalid" style used by the Go service's binding error responses.
pub fn validation_message(errors: &validator::ValidationErrors) -> String {
    if let Some((field, errs)) = errors.field_errors().into_iter().next() {
        if let Some(err) = errs.first() {
            let msg = match err.code.as_ref() {
                "email" => format!("{field} must be a valid email address"),
                "length" => format!("{field} has an invalid length"),
                "range" => format!("{field} is out of range"),
                _ => format!("{field} is invalid"),
            };
            return msg;
        }
    }
    "validation failed".to_string()
}

/// Axum extractor that deserializes JSON and runs `validator` validation,
/// returning `AppError` (consistent JSON error body) on failure.
pub struct ValidatedJson<T>(pub T);

#[axum::async_trait]
impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    S: Send + Sync,
    T: Validate,
    Json<T>: FromRequest<S, Rejection = JsonRejection>,
{
    type Rejection = AppError;

    async fn from_request(req: axum::extract::Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|e| AppError::BadRequest(e.body_text()))?;
        value
            .validate()
            .map_err(|e| AppError::Validation(validation_message(&e)))?;
        Ok(ValidatedJson(value))
    }
}
