//! Chat repository (sqlx/PostgreSQL): conversations, members, and messages.
//! Messages preload their shared songs and sender (GORM `Preload` equivalent).

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::domain::entity::{ChatMessage, Conversation, ConversationMember, Music, User};
use crate::domain::repository::ChatRepository;
use crate::error::{AppError, AppResult};
use crate::infrastructure::persistence::rows::{
    ChatMessageRow, ConversationMemberRow, ConversationRow, MusicRow, UserRow,
};

pub struct PgChatRepository {
    pool: PgPool,
}

impl PgChatRepository {
    pub fn new(pool: PgPool) -> Self {
        PgChatRepository { pool }
    }

    async fn load_message_musics(&self, message_id: i64) -> AppResult<Vec<Music>> {
        let rows = sqlx::query_as::<_, MusicRow>(
            r#"
            SELECT m.* FROM musics m
            JOIN chat_message_musics cmm ON m.id = cmm.music_id
            WHERE cmm.message_id = $1
            ORDER BY cmm.display_order ASC, cmm.id ASC
            "#,
        )
        .bind(message_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(MusicRow::into_entity).collect())
    }

    async fn load_user(&self, user_id: i64) -> AppResult<Option<User>> {
        let row = sqlx::query_as::<_, UserRow>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(UserRow::into_entity))
    }

    async fn attach_message_relations(&self, mut message: ChatMessage) -> AppResult<ChatMessage> {
        let musics = self.load_message_musics(message.id).await?;
        message.music_ids = musics.iter().map(|m| m.id).collect();
        message.musics = musics;
        message.sender = self.load_user(message.sender_id).await?;
        Ok(message)
    }

    async fn load_members_with_users(
        &self,
        conversation_id: i64,
    ) -> AppResult<Vec<ConversationMember>> {
        let rows = sqlx::query_as::<_, ConversationMemberRow>(
            "SELECT * FROM conversation_members WHERE conversation_id = $1 ORDER BY id",
        )
        .bind(conversation_id)
        .fetch_all(&self.pool)
        .await?;
        let mut members = Vec::with_capacity(rows.len());
        for row in rows {
            let mut member = row.into_entity();
            member.user = self.load_user(member.user_id).await?;
            members.push(member);
        }
        Ok(members)
    }
}

#[async_trait]
impl ChatRepository for PgChatRepository {
    async fn create_conversation(&self, conversation: &mut Conversation) -> AppResult<()> {
        let rec: (i64, DateTime<Utc>, DateTime<Utc>) = sqlx::query_as(
            r#"
            INSERT INTO conversations (type, name, band_id, owner_id, created_at, updated_at)
            VALUES ($1,$2,$3,$4, NOW(), NOW())
            RETURNING id, created_at, updated_at
            "#,
        )
        .bind(&conversation.r#type)
        .bind(&conversation.name)
        .bind(conversation.band_id)
        .bind(conversation.owner_id)
        .fetch_one(&self.pool)
        .await?;
        conversation.id = rec.0;
        conversation.created_at = rec.1;
        conversation.updated_at = rec.2;
        Ok(())
    }

    async fn find_conversation(&self, id: i64) -> AppResult<Conversation> {
        let row = sqlx::query_as::<_, ConversationRow>("SELECT * FROM conversations WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("conversation not found".to_string()))?;
        let mut conversation = row.into_entity();
        conversation.members = self.load_members_with_users(conversation.id).await?;
        Ok(conversation)
    }

    async fn find_conversations_for_user(&self, user_id: i64) -> AppResult<Vec<Conversation>> {
        let rows = sqlx::query_as::<_, ConversationRow>(
            r#"
            SELECT c.* FROM conversations c
            JOIN conversation_members cm ON cm.conversation_id = c.id
            WHERE cm.user_id = $1
            ORDER BY c.updated_at DESC, c.id DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        let mut conversations = Vec::with_capacity(rows.len());
        for row in rows {
            let mut conversation = row.into_entity();
            conversation.members = self.load_members_with_users(conversation.id).await?;
            conversations.push(conversation);
        }
        Ok(conversations)
    }

    async fn find_private_between(
        &self,
        user_a: i64,
        user_b: i64,
    ) -> AppResult<Option<Conversation>> {
        let row = sqlx::query_as::<_, ConversationRow>(
            r#"
            SELECT c.* FROM conversations c
            WHERE c.type = 'private'
              AND EXISTS (SELECT 1 FROM conversation_members m WHERE m.conversation_id = c.id AND m.user_id = $1)
              AND EXISTS (SELECT 1 FROM conversation_members m WHERE m.conversation_id = c.id AND m.user_id = $2)
              AND (SELECT COUNT(*) FROM conversation_members m WHERE m.conversation_id = c.id) = 2
            LIMIT 1
            "#,
        )
        .bind(user_a)
        .bind(user_b)
        .fetch_optional(&self.pool)
        .await?;
        match row {
            None => Ok(None),
            Some(row) => {
                let mut conversation = row.into_entity();
                conversation.members = self.load_members_with_users(conversation.id).await?;
                Ok(Some(conversation))
            }
        }
    }

    async fn find_band_conversation(&self, band_id: i64) -> AppResult<Option<Conversation>> {
        let row = sqlx::query_as::<_, ConversationRow>(
            "SELECT * FROM conversations WHERE type = 'band' AND band_id = $1 LIMIT 1",
        )
        .bind(band_id)
        .fetch_optional(&self.pool)
        .await?;
        match row {
            None => Ok(None),
            Some(row) => {
                let mut conversation = row.into_entity();
                conversation.members = self.load_members_with_users(conversation.id).await?;
                Ok(Some(conversation))
            }
        }
    }

    async fn delete_conversation(&self, id: i64) -> AppResult<()> {
        sqlx::query(
            r#"
            DELETE FROM chat_message_musics
            WHERE message_id IN (SELECT id FROM chat_messages WHERE conversation_id = $1)
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        sqlx::query("DELETE FROM chat_messages WHERE conversation_id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM conversation_members WHERE conversation_id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM conversations WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn add_member(&self, member: &mut ConversationMember) -> AppResult<()> {
        let rec: (i64, DateTime<Utc>) = sqlx::query_as(
            r#"
            INSERT INTO conversation_members (conversation_id, user_id, role, can_send, created_at)
            VALUES ($1,$2,$3,$4, NOW())
            ON CONFLICT (conversation_id, user_id)
            DO UPDATE SET role = EXCLUDED.role, can_send = EXCLUDED.can_send
            RETURNING id, created_at
            "#,
        )
        .bind(member.conversation_id)
        .bind(member.user_id)
        .bind(&member.role)
        .bind(member.can_send)
        .fetch_one(&self.pool)
        .await?;
        member.id = rec.0;
        member.created_at = rec.1;
        Ok(())
    }

    async fn remove_member(&self, conversation_id: i64, user_id: i64) -> AppResult<()> {
        sqlx::query("DELETE FROM conversation_members WHERE conversation_id = $1 AND user_id = $2")
            .bind(conversation_id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn find_member(
        &self,
        conversation_id: i64,
        user_id: i64,
    ) -> AppResult<Option<ConversationMember>> {
        let row = sqlx::query_as::<_, ConversationMemberRow>(
            "SELECT * FROM conversation_members WHERE conversation_id = $1 AND user_id = $2",
        )
        .bind(conversation_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(ConversationMemberRow::into_entity))
    }

    async fn find_members(&self, conversation_id: i64) -> AppResult<Vec<ConversationMember>> {
        self.load_members_with_users(conversation_id).await
    }

    async fn update_member(&self, member: &ConversationMember) -> AppResult<()> {
        sqlx::query(
            "UPDATE conversation_members SET role = $1, can_send = $2 WHERE conversation_id = $3 AND user_id = $4",
        )
        .bind(&member.role)
        .bind(member.can_send)
        .bind(member.conversation_id)
        .bind(member.user_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn save_message(&self, message: &mut ChatMessage) -> AppResult<()> {
        let rec: (i64, DateTime<Utc>, DateTime<Utc>) = sqlx::query_as(
            r#"
            INSERT INTO chat_messages
                (conversation_id, sender_id, message_type, content, media_url, duration, created_at, updated_at)
            VALUES ($1,$2,$3,$4,$5,$6, NOW(), NOW())
            RETURNING id, created_at, updated_at
            "#,
        )
        .bind(message.conversation_id)
        .bind(message.sender_id)
        .bind(&message.message_type)
        .bind(&message.content)
        .bind(&message.media_url)
        .bind(message.duration)
        .fetch_one(&self.pool)
        .await?;
        message.id = rec.0;
        message.created_at = rec.1;
        message.updated_at = rec.2;

        for (idx, music_id) in message.music_ids.iter().enumerate() {
            sqlx::query(
                "INSERT INTO chat_message_musics (message_id, music_id, display_order, created_at) VALUES ($1,$2,$3, NOW())",
            )
            .bind(message.id)
            .bind(music_id)
            .bind(idx as i32)
            .execute(&self.pool)
            .await?;
        }

        // Surface the latest activity for conversation ordering.
        sqlx::query("UPDATE conversations SET updated_at = NOW() WHERE id = $1")
            .bind(message.conversation_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn find_message(&self, id: i64) -> AppResult<ChatMessage> {
        let row = sqlx::query_as::<_, ChatMessageRow>("SELECT * FROM chat_messages WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("message not found".to_string()))?;
        self.attach_message_relations(row.into_entity()).await
    }

    async fn find_messages(
        &self,
        conversation_id: i64,
        offset: i64,
        limit: i64,
    ) -> AppResult<(Vec<ChatMessage>, i64)> {
        let total: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM chat_messages WHERE conversation_id = $1")
                .bind(conversation_id)
                .fetch_one(&self.pool)
                .await?;
        let rows = sqlx::query_as::<_, ChatMessageRow>(
            "SELECT * FROM chat_messages WHERE conversation_id = $1 ORDER BY created_at DESC, id DESC OFFSET $2 LIMIT $3",
        )
        .bind(conversation_id)
        .bind(offset)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut messages = Vec::with_capacity(rows.len());
        for row in rows {
            messages.push(self.attach_message_relations(row.into_entity()).await?);
        }
        Ok((messages, total))
    }

    async fn delete_message(&self, id: i64) -> AppResult<()> {
        sqlx::query("DELETE FROM chat_message_musics WHERE message_id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM chat_messages WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
