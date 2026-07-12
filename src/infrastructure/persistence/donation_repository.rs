//! Donation repository (sqlx/PostgreSQL), preloading user, supporter, and event.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::domain::entity::Donation;
use crate::domain::repository::DonationRepository;
use crate::error::{AppError, AppResult};
use crate::infrastructure::persistence::rows::{DonationRow, EventRow, SupporterRow, UserRow};

pub struct PgDonationRepository {
    pool: PgPool,
}

impl PgDonationRepository {
    pub fn new(pool: PgPool) -> Self {
        PgDonationRepository { pool }
    }

    fn limit_offset(limit: i64, offset: i64) -> String {
        let mut s = String::new();
        if limit > 0 {
            s.push_str(&format!(" LIMIT {limit}"));
        }
        if offset > 0 {
            s.push_str(&format!(" OFFSET {offset}"));
        }
        s
    }

    async fn attach_relations(&self, donation: &mut Donation) -> AppResult<()> {
        if let Some(user_id) = donation.user_id {
            if let Some(row) = sqlx::query_as::<_, UserRow>("SELECT * FROM users WHERE id = $1")
                .bind(user_id)
                .fetch_optional(&self.pool)
                .await?
            {
                donation.user = Some(row.into_entity());
            }
        }
        if let Some(supporter_id) = donation.supporter_id {
            if let Some(row) = sqlx::query_as::<_, SupporterRow>("SELECT * FROM supporters WHERE id = $1")
                .bind(supporter_id)
                .fetch_optional(&self.pool)
                .await?
            {
                donation.supporter = Some(row.into_entity());
            }
        }
        if let Some(event_id) = donation.event_id {
            if let Some(row) = sqlx::query_as::<_, EventRow>("SELECT * FROM events WHERE id = $1")
                .bind(event_id)
                .fetch_optional(&self.pool)
                .await?
            {
                donation.event = Some(row.into_entity());
            }
        }
        Ok(())
    }

    async fn attach_many(&self, donations: &mut [Donation]) -> AppResult<()> {
        for donation in donations.iter_mut() {
            self.attach_relations(donation).await?;
        }
        Ok(())
    }

    async fn list_where(&self, where_clause: &str, bind: Option<&str>, limit: i64, offset: i64) -> AppResult<Vec<Donation>> {
        let sql = format!(
            "SELECT * FROM donations{} ORDER BY created_at DESC{}",
            if where_clause.is_empty() { String::new() } else { format!(" WHERE {where_clause}") },
            Self::limit_offset(limit, offset)
        );
        let mut query = sqlx::query_as::<_, DonationRow>(&sql);
        if let Some(b) = bind {
            query = query.bind(b.to_string());
        }
        let rows = query.fetch_all(&self.pool).await?;
        let mut donations: Vec<Donation> = rows.into_iter().map(DonationRow::into_entity).collect();
        self.attach_many(&mut donations).await?;
        Ok(donations)
    }
}

#[async_trait]
impl DonationRepository for PgDonationRepository {
    async fn save(&self, donation: &mut Donation) -> AppResult<()> {
        let rec: (i64, DateTime<Utc>, DateTime<Utc>) = sqlx::query_as(
            r#"
            INSERT INTO donations
                (type, donor_type, user_id, supporter_id, company_name, company_email,
                 company_phone, amount, currency, message, status, transaction_id, payment_method,
                 qr_expires_at, event_id, created_at, updated_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15, NOW(), NOW())
            RETURNING id, created_at, updated_at
            "#,
        )
        .bind(&donation.r#type)
        .bind(&donation.donor_type)
        .bind(donation.user_id)
        .bind(donation.supporter_id)
        .bind(&donation.company_name)
        .bind(&donation.company_email)
        .bind(&donation.company_phone)
        .bind(donation.amount)
        .bind(&donation.currency)
        .bind(&donation.message)
        .bind(&donation.status)
        .bind(&donation.transaction_id)
        .bind(&donation.payment_method)
        .bind(donation.qr_expires_at)
        .bind(donation.event_id)
        .fetch_one(&self.pool)
        .await?;
        donation.id = rec.0;
        donation.created_at = rec.1;
        donation.updated_at = rec.2;
        Ok(())
    }

    async fn find_by_id(&self, id: i64) -> AppResult<Donation> {
        let row = sqlx::query_as::<_, DonationRow>("SELECT * FROM donations WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("donation not found".to_string()))?;
        let mut donation = row.into_entity();
        self.attach_relations(&mut donation).await?;
        Ok(donation)
    }

    async fn find_all(&self, limit: i64, offset: i64) -> AppResult<Vec<Donation>> {
        self.list_where("", None, limit, offset).await
    }

    async fn find_by_user_id(&self, user_id: i64, limit: i64, offset: i64) -> AppResult<Vec<Donation>> {
        let sql = format!(
            "SELECT * FROM donations WHERE user_id = $1 ORDER BY created_at DESC{}",
            Self::limit_offset(limit, offset)
        );
        let rows = sqlx::query_as::<_, DonationRow>(&sql)
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?;
        let mut donations: Vec<Donation> = rows.into_iter().map(DonationRow::into_entity).collect();
        self.attach_many(&mut donations).await?;
        Ok(donations)
    }

    async fn find_by_type(&self, donation_type: &str, limit: i64, offset: i64) -> AppResult<Vec<Donation>> {
        self.list_where("type = $1", Some(donation_type), limit, offset).await
    }

    async fn find_by_donor_type(&self, donor_type: &str, limit: i64, offset: i64) -> AppResult<Vec<Donation>> {
        self.list_where("donor_type = $1", Some(donor_type), limit, offset).await
    }

    async fn find_by_event_id(&self, event_id: i64, limit: i64, offset: i64) -> AppResult<Vec<Donation>> {
        let sql = format!(
            "SELECT * FROM donations WHERE event_id = $1 ORDER BY created_at DESC{}",
            Self::limit_offset(limit, offset)
        );
        let rows = sqlx::query_as::<_, DonationRow>(&sql)
            .bind(event_id)
            .fetch_all(&self.pool)
            .await?;
        let mut donations: Vec<Donation> = rows.into_iter().map(DonationRow::into_entity).collect();
        self.attach_many(&mut donations).await?;
        Ok(donations)
    }

    async fn find_by_status(&self, status: &str, limit: i64, offset: i64) -> AppResult<Vec<Donation>> {
        self.list_where("status = $1", Some(status), limit, offset).await
    }

    async fn update(&self, donation: &Donation) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE donations SET
                type = $1, donor_type = $2, user_id = $3, supporter_id = $4, company_name = $5,
                company_email = $6, company_phone = $7, amount = $8, currency = $9, message = $10,
                status = $11, transaction_id = $12, payment_method = $13, qr_expires_at = $14,
                event_id = $15, updated_at = NOW()
            WHERE id = $16
            "#,
        )
        .bind(&donation.r#type)
        .bind(&donation.donor_type)
        .bind(donation.user_id)
        .bind(donation.supporter_id)
        .bind(&donation.company_name)
        .bind(&donation.company_email)
        .bind(&donation.company_phone)
        .bind(donation.amount)
        .bind(&donation.currency)
        .bind(&donation.message)
        .bind(&donation.status)
        .bind(&donation.transaction_id)
        .bind(&donation.payment_method)
        .bind(donation.qr_expires_at)
        .bind(donation.event_id)
        .bind(donation.id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update_status(&self, id: i64, status: &str, transaction_id: &str, payment_method: &str) -> AppResult<()> {
        sqlx::query(
            "UPDATE donations SET status = $1, transaction_id = $2, payment_method = $3, updated_at = NOW() WHERE id = $4",
        )
        .bind(status)
        .bind(transaction_id)
        .bind(payment_method)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, id: i64) -> AppResult<()> {
        sqlx::query("DELETE FROM donations WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_total_amount(&self) -> AppResult<f64> {
        let total: f64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(amount), 0) FROM donations WHERE status = 'completed'",
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(total)
    }

    async fn get_total_amount_by_type(&self, donation_type: &str) -> AppResult<f64> {
        let total: f64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(amount), 0) FROM donations WHERE type = $1 AND status = 'completed'",
        )
        .bind(donation_type)
        .fetch_one(&self.pool)
        .await?;
        Ok(total)
    }

    async fn get_total_amount_by_event_id(&self, event_id: i64) -> AppResult<f64> {
        let total: f64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(amount), 0) FROM donations WHERE event_id = $1 AND status = 'completed'",
        )
        .bind(event_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(total)
    }

    async fn count(&self) -> AppResult<i64> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM donations")
            .fetch_one(&self.pool)
            .await?;
        Ok(count)
    }

    async fn count_by_type(&self, donation_type: &str) -> AppResult<i64> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM donations WHERE type = $1")
            .bind(donation_type)
            .fetch_one(&self.pool)
            .await?;
        Ok(count)
    }
}
