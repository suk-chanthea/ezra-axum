//! Google ID token verification. The Go service used the official `idtoken` package; here we
//! validate via Google's tokeninfo endpoint, which checks the signature and expiry server-side.

use serde::Deserialize;

use crate::error::{AppError, AppResult};

#[derive(Debug, Deserialize)]
pub struct GoogleClaims {
    #[serde(default)]
    pub sub: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub picture: String,
}

#[derive(Clone)]
pub struct GoogleVerifier {
    client: reqwest::Client,
}

impl GoogleVerifier {
    pub fn new() -> Self {
        GoogleVerifier {
            client: reqwest::Client::new(),
        }
    }

    /// Validates the ID token and returns its claims, or an unauthorized error.
    pub async fn validate(&self, id_token: &str) -> AppResult<GoogleClaims> {
        let url = format!("https://oauth2.googleapis.com/tokeninfo?id_token={id_token}");
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|_| AppError::Unauthorized("invalid Google ID token".to_string()))?;

        if !resp.status().is_success() {
            return Err(AppError::Unauthorized("invalid Google ID token".to_string()));
        }

        resp.json::<GoogleClaims>()
            .await
            .map_err(|_| AppError::Unauthorized("invalid Google ID token".to_string()))
    }
}

impl Default for GoogleVerifier {
    fn default() -> Self {
        Self::new()
    }
}
