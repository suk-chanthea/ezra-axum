//! AWS S3 service abstraction.

use async_trait::async_trait;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client as S3Client;
use aws_sdk_s3::config::Credentials;
use base64::Engine;
use uuid::Uuid;

use crate::error::{AppError, AppResult};

#[async_trait]
pub trait S3Service: Send + Sync {
    async fn upload_image(&self, base64_image: &str) -> AppResult<String>;
}

pub struct DummyS3Service;

#[async_trait]
impl S3Service for DummyS3Service {
    async fn upload_image(&self, base64_image: &str) -> AppResult<String> {
        let (_, extension) = decode_base64_image(base64_image)?;
        let filename = format!("profiles/dummy-{}.{}", Uuid::new_v4(), extension);
        tracing::info!("[DUMMY S3] Uploading image: returning simulated key {}", filename);
        Ok(filename)
    }
}

pub struct AwsS3Service {
    client: S3Client,
    bucket: String,
}

impl AwsS3Service {
    pub fn new(
        bucket: String,
        region: String,
        access_key_id: String,
        secret_access_key: String,
        endpoint: String,
    ) -> Self {
        let credentials = Credentials::new(
            access_key_id,
            secret_access_key,
            None,
            None,
            "ezra-axum",
        );

        let mut config_builder = aws_sdk_s3::config::Builder::new()
            .region(aws_sdk_s3::config::Region::new(region))
            .credentials_provider(credentials);

        if !endpoint.is_empty() {
            config_builder = config_builder.endpoint_url(endpoint);
        }

        let config = config_builder.build();
        let client = S3Client::from_conf(config);

        AwsS3Service { client, bucket }
    }
}

#[async_trait]
impl S3Service for AwsS3Service {
    async fn upload_image(&self, base64_image: &str) -> AppResult<String> {
        let (data, extension) = decode_base64_image(base64_image)?;
        let file_key = format!("profiles/{}.{}", Uuid::new_v4(), extension);

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&file_key)
            .body(ByteStream::from(data))
            .content_type(format!("image/{}", extension))
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("S3 upload error: {:?}", e)))?;

        Ok(file_key)
    }
}

fn decode_base64_image(base64_str: &str) -> AppResult<(Vec<u8>, &'static str)> {
    let clean_str = if base64_str.starts_with("data:image/") {
        let parts: Vec<&str> = base64_str.split(";base64,").collect();
        if parts.len() != 2 {
            return Err(AppError::BadRequest("invalid base64 image data URL".to_string()));
        }
        parts[1]
    } else {
        base64_str
    };

    let decoded = base64::prelude::BASE64_STANDARD
        .decode(clean_str.trim())
        .map_err(|e| AppError::BadRequest(format!("failed to decode base64: {}", e)))?;

    let ext = if decoded.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
        "png"
    } else if decoded.starts_with(&[0xFF, 0xD8, 0xFF]) {
        "jpg"
    } else if decoded.starts_with(&[0x47, 0x49, 0x46, 0x38]) {
        "gif"
    } else if decoded.len() > 12 && &decoded[0..4] == b"RIFF" && &decoded[8..12] == b"WEBP" {
        "webp"
    } else {
        "png"
    };

    Ok((decoded, ext))
}
