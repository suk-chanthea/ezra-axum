//! PayWay payment gateway integration, mirroring the Go `infrastructure/payment` package
//! (HMAC-SHA512 request hashing, QR/card initiation, callback verification).

use std::collections::HashMap;

use async_trait::async_trait;
use base64::Engine;
use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::Sha512;

use crate::config::PaywayConfig;
use crate::error::{AppError, AppResult};

type HmacSha512 = Hmac<Sha512>;

#[derive(Debug, Default, Deserialize)]
pub struct PaymentResponse {
    #[serde(default)]
    pub status: i64,
    #[serde(default)]
    pub message: String,
    #[serde(default, rename = "tran_id")]
    pub transaction_id: String,
    #[serde(default)]
    pub payment_url: String,
    #[serde(default)]
    pub qr_code: String,
    #[serde(default)]
    pub hash: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct TransactionStatusResponse {
    #[serde(default, rename = "tran_id")]
    pub transaction_id: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub amount: String,
    #[serde(default)]
    pub currency: String,
    #[serde(default, rename = "payment_option")]
    pub payment_method: String,
    #[serde(default)]
    pub message: String,
}

#[derive(Debug, Default)]
pub struct PaymentCallbackData {
    pub transaction_id: String,
    pub status: String,
    pub amount: String,
    pub currency: String,
    pub hash: String,
    pub payment_method: String,
}

#[async_trait]
pub trait PaywayService: Send + Sync {
    async fn initiate_qr_payment(
        &self,
        transaction_id: &str,
        amount: &str,
        currency: &str,
        customer_name: &str,
        customer_email: &str,
        customer_phone: &str,
        items: &str,
    ) -> AppResult<PaymentResponse>;

    async fn initiate_card_payment(
        &self,
        transaction_id: &str,
        amount: &str,
        currency: &str,
        customer_name: &str,
        customer_email: &str,
        customer_phone: &str,
        items: &str,
    ) -> AppResult<PaymentResponse>;

    fn verify_callback(&self, data: &PaymentCallbackData) -> bool;
    fn generate_hash(&self, data: &HashMap<String, String>) -> String;
}

/// Formats an amount for PayWay (USD in cents, KHR whole units), mirroring Go's `FormatAmount`.
pub fn format_amount(amount: f64, currency: &str) -> String {
    if currency == "KHR" {
        format!("{:.0}", amount)
    } else {
        format!("{:.0}", amount * 100.0)
    }
}

pub struct HttpPaywayService {
    config: PaywayConfig,
    client: reqwest::Client,
}

impl HttpPaywayService {
    pub fn new(config: PaywayConfig) -> Self {
        HttpPaywayService {
            config,
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
        }
    }

    async fn send_payment_request(
        &self,
        transaction_id: &str,
        amount: &str,
        currency: &str,
        payment_option: &str,
        customer_name: &str,
        customer_email: &str,
        customer_phone: &str,
        items: &str,
    ) -> AppResult<PaymentResponse> {
        let mut data: HashMap<String, String> = HashMap::new();
        data.insert("merchant_id".into(), self.config.merchant_id.clone());
        data.insert("tran_id".into(), transaction_id.into());
        data.insert("amount".into(), amount.into());
        data.insert("currency".into(), currency.into());
        data.insert("payment_option".into(), payment_option.into());
        data.insert("return_url".into(), self.config.return_url.clone());
        data.insert("continue_success_url".into(), self.config.continue_url.clone());
        data.insert("firstname".into(), customer_name.into());
        data.insert("email".into(), customer_email.into());
        data.insert("phone".into(), customer_phone.into());
        data.insert("items".into(), items.into());
        data.insert("shipping".into(), "NA".into());

        let hash = self.generate_hash(&data);
        data.insert("hash".into(), hash);
        data.insert("type".into(), "purchase".into());

        let url = format!("{}/api/payment-gateway/v1/payments/purchase", self.config.base_url);
        let resp = self
            .client
            .post(&url)
            .basic_auth(&self.config.api_username, Some(&self.config.api_key))
            .json(&data)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("failed to send request: {e}")))?;

        let payment_resp: PaymentResponse = resp
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("failed to parse response: {e}")))?;

        if payment_resp.status != 200 && payment_resp.status != 0 {
            return Err(AppError::Internal(format!(
                "payway error: {} (status: {})",
                payment_resp.message, payment_resp.status
            )));
        }

        Ok(payment_resp)
    }
}

#[async_trait]
impl PaywayService for HttpPaywayService {
    async fn initiate_qr_payment(
        &self,
        transaction_id: &str,
        amount: &str,
        currency: &str,
        customer_name: &str,
        customer_email: &str,
        customer_phone: &str,
        items: &str,
    ) -> AppResult<PaymentResponse> {
        self.send_payment_request(transaction_id, amount, currency, "qr", customer_name, customer_email, customer_phone, items)
            .await
    }

    async fn initiate_card_payment(
        &self,
        transaction_id: &str,
        amount: &str,
        currency: &str,
        customer_name: &str,
        customer_email: &str,
        customer_phone: &str,
        items: &str,
    ) -> AppResult<PaymentResponse> {
        self.send_payment_request(transaction_id, amount, currency, "card", customer_name, customer_email, customer_phone, items)
            .await
    }

    fn verify_callback(&self, data: &PaymentCallbackData) -> bool {
        let mut req: HashMap<String, String> = HashMap::new();
        req.insert("tran_id".into(), data.transaction_id.clone());
        req.insert("status".into(), data.status.clone());
        req.insert("amount".into(), data.amount.clone());
        req.insert("currency".into(), data.currency.clone());
        req.insert("payment_option".into(), data.payment_method.clone());
        self.generate_hash(&req) == data.hash
    }

    fn generate_hash(&self, data: &HashMap<String, String>) -> String {
        let mut hash_string = String::new();
        for key in ["merchant_id", "tran_id", "amount", "currency", "status", "payment_option"] {
            if let Some(val) = data.get(key) {
                hash_string.push_str(val);
            }
        }

        let mut mac = HmacSha512::new_from_slice(self.config.api_key.as_bytes())
            .expect("HMAC accepts any key length");
        mac.update(hash_string.as_bytes());
        base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes())
    }
}

pub struct DummyPaywayService;

#[async_trait]
impl PaywayService for DummyPaywayService {
    async fn initiate_qr_payment(&self, _: &str, _: &str, _: &str, _: &str, _: &str, _: &str, _: &str) -> AppResult<PaymentResponse> {
        Err(AppError::BadRequest("payway service disabled".to_string()))
    }
    async fn initiate_card_payment(&self, _: &str, _: &str, _: &str, _: &str, _: &str, _: &str, _: &str) -> AppResult<PaymentResponse> {
        Err(AppError::BadRequest("payway service disabled".to_string()))
    }
    fn verify_callback(&self, _data: &PaymentCallbackData) -> bool {
        false
    }
    fn generate_hash(&self, _data: &HashMap<String, String>) -> String {
        String::new()
    }
}
