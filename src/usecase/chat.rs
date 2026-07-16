//! Chat use case: private (1:1), group, and band conversations with role/permission control.

use std::collections::HashMap;
use std::sync::Arc;

use crate::domain::entity::{
    chat_message_type, chat_role, conversation_type, ChatMessage, Conversation, ConversationMember,
};
use crate::domain::repository::{BandRepository, ChatRepository, MusicRepository, UserRepository};
use crate::error::{AppError, AppResult};
use crate::infrastructure::fcm::FcmService;
use crate::infrastructure::s3::S3Service;
use crate::interface::http::dto::request::{
    AddConversationMemberRequest, CreateConversationRequest, SendChatMessageRequest,
    UpdateConversationMemberRequest,
};
use crate::interface::http::dto::response::{
    ChatMessageResponse, ConversationMemberResponse, ConversationResponse, PaginationMetadata,
};

pub struct ChatUseCase {
    chat_repo: Arc<dyn ChatRepository>,
    band_repo: Arc<dyn BandRepository>,
    music_repo: Arc<dyn MusicRepository>,
    user_repo: Arc<dyn UserRepository>,
    fcm_service: Arc<dyn FcmService>,
    s3_service: Arc<dyn S3Service>,
}

impl ChatUseCase {
    pub fn new(
        chat_repo: Arc<dyn ChatRepository>,
        band_repo: Arc<dyn BandRepository>,
        music_repo: Arc<dyn MusicRepository>,
        user_repo: Arc<dyn UserRepository>,
        fcm_service: Arc<dyn FcmService>,
        s3_service: Arc<dyn S3Service>,
    ) -> Self {
        ChatUseCase { chat_repo, band_repo, music_repo, user_repo, fcm_service, s3_service }
    }

    // ---- Permission helpers ----

    async fn require_member(
        &self,
        conversation_id: i64,
        user_id: i64,
    ) -> AppResult<ConversationMember> {
        self.chat_repo
            .find_member(conversation_id, user_id)
            .await?
            .ok_or_else(|| {
                AppError::Forbidden("you are not a member of this conversation".to_string())
            })
    }

    async fn require_privileged(
        &self,
        conversation_id: i64,
        user_id: i64,
    ) -> AppResult<ConversationMember> {
        let member = self.require_member(conversation_id, user_id).await?;
        if !member.is_privileged() {
            return Err(AppError::Forbidden(
                "only the owner or an admin can perform this action".to_string(),
            ));
        }
        Ok(member)
    }

    async fn ensure_user_exists(&self, user_id: i64) -> AppResult<()> {
        self.user_repo
            .find_by_id(user_id)
            .await
            .map_err(|_| AppError::BadRequest(format!("user {user_id} does not exist")))?;
        Ok(())
    }

    // ---- Conversations ----

    pub async fn create_conversation(
        &self,
        actor_id: i64,
        req: CreateConversationRequest,
    ) -> AppResult<ConversationResponse> {
        let conv_type = req.r#type.trim().to_lowercase();
        let conversation = match conv_type.as_str() {
            conversation_type::PRIVATE => self.create_private(actor_id, &req).await?,
            conversation_type::GROUP => self.create_group(actor_id, &req).await?,
            conversation_type::BAND => self.create_band(actor_id, &req).await?,
            other => {
                return Err(AppError::BadRequest(format!(
                    "invalid conversation type '{other}' (expected private, group or band)"
                )))
            }
        };
        let full = self.chat_repo.find_conversation(conversation.id).await?;
        Ok(ConversationResponse::from_entity(&full))
    }

    async fn create_private(
        &self,
        actor_id: i64,
        req: &CreateConversationRequest,
    ) -> AppResult<Conversation> {
        let others: Vec<i64> = req.member_ids.iter().copied().filter(|id| *id != actor_id).collect();
        if others.len() != 1 {
            return Err(AppError::BadRequest(
                "a private conversation needs exactly one other member".to_string(),
            ));
        }
        let other_id = others[0];
        self.ensure_user_exists(other_id).await?;

        if let Some(existing) = self.chat_repo.find_private_between(actor_id, other_id).await? {
            return Ok(existing);
        }

        let mut conversation =
            Conversation::new(conversation_type::PRIVATE.to_string(), String::new(), actor_id, None);
        self.chat_repo.create_conversation(&mut conversation).await?;

        for uid in [actor_id, other_id] {
            let mut member =
                ConversationMember::new(conversation.id, uid, chat_role::MEMBER.to_string(), true);
            self.chat_repo.add_member(&mut member).await?;
        }
        Ok(conversation)
    }

    async fn create_group(
        &self,
        actor_id: i64,
        req: &CreateConversationRequest,
    ) -> AppResult<Conversation> {
        let name = req.name.trim();
        if name.is_empty() {
            return Err(AppError::BadRequest("group name is required".to_string()));
        }

        let mut conversation =
            Conversation::new(conversation_type::GROUP.to_string(), name.to_string(), actor_id, None);
        self.chat_repo.create_conversation(&mut conversation).await?;

        let mut owner =
            ConversationMember::new(conversation.id, actor_id, chat_role::OWNER.to_string(), true);
        self.chat_repo.add_member(&mut owner).await?;

        for uid in req.member_ids.iter().copied().filter(|id| *id != actor_id) {
            self.ensure_user_exists(uid).await?;
            let mut member =
                ConversationMember::new(conversation.id, uid, chat_role::MEMBER.to_string(), true);
            self.chat_repo.add_member(&mut member).await?;
        }
        Ok(conversation)
    }

    async fn create_band(
        &self,
        actor_id: i64,
        req: &CreateConversationRequest,
    ) -> AppResult<Conversation> {
        let band_id = req
            .band_id
            .ok_or_else(|| AppError::BadRequest("band_id is required for a band chat".to_string()))?;
        let band = self
            .band_repo
            .find_by_id(band_id)
            .await
            .map_err(|_| AppError::NotFound("band not found".to_string()))?;

        if !self.is_band_member(band_id, band.user_id, actor_id).await? {
            return Err(AppError::Forbidden(
                "you are not a member of this band".to_string(),
            ));
        }

        if let Some(existing) = self.chat_repo.find_band_conversation(band_id).await? {
            return Ok(existing);
        }

        let mut conversation = Conversation::new(
            conversation_type::BAND.to_string(),
            band.name.clone(),
            band.user_id,
            Some(band_id),
        );
        self.chat_repo.create_conversation(&mut conversation).await?;

        // Band owner is the conversation owner; all band members join as members.
        let mut owner = ConversationMember::new(
            conversation.id,
            band.user_id,
            chat_role::OWNER.to_string(),
            true,
        );
        self.chat_repo.add_member(&mut owner).await?;

        let members = self.band_repo.get_band_members(band_id).await.unwrap_or_default();
        for user in members {
            if user.id == band.user_id {
                continue;
            }
            let mut member = ConversationMember::new(
                conversation.id,
                user.id,
                chat_role::MEMBER.to_string(),
                true,
            );
            self.chat_repo.add_member(&mut member).await?;
        }

        // Ensure the actor is a participant even if not yet linked via users.band_id.
        if self.chat_repo.find_member(conversation.id, actor_id).await?.is_none() {
            let mut member = ConversationMember::new(
                conversation.id,
                actor_id,
                chat_role::MEMBER.to_string(),
                true,
            );
            self.chat_repo.add_member(&mut member).await?;
        }
        Ok(conversation)
    }

    async fn is_band_member(
        &self,
        band_id: i64,
        band_owner_id: i64,
        user_id: i64,
    ) -> AppResult<bool> {
        if band_owner_id == user_id {
            return Ok(true);
        }
        let user = self.user_repo.find_by_id(user_id).await?;
        Ok(user.band_id == Some(band_id))
    }

    pub async fn get_conversations(&self, user_id: i64) -> AppResult<Vec<ConversationResponse>> {
        let conversations = self.chat_repo.find_conversations_for_user(user_id).await?;
        Ok(ConversationResponse::list(&conversations))
    }

    pub async fn get_conversation(
        &self,
        conversation_id: i64,
        user_id: i64,
    ) -> AppResult<ConversationResponse> {
        self.require_member(conversation_id, user_id).await?;
        let conversation = self.chat_repo.find_conversation(conversation_id).await?;
        Ok(ConversationResponse::from_entity(&conversation))
    }

    pub async fn delete_conversation(
        &self,
        conversation_id: i64,
        actor_id: i64,
    ) -> AppResult<()> {
        let conversation = self.chat_repo.find_conversation(conversation_id).await?;
        if conversation.owner_id != actor_id {
            return Err(AppError::Forbidden(
                "only the owner can delete this conversation".to_string(),
            ));
        }
        self.chat_repo.delete_conversation(conversation_id).await
    }

    // ---- Members ----

    pub async fn get_members(
        &self,
        conversation_id: i64,
        user_id: i64,
    ) -> AppResult<Vec<ConversationMemberResponse>> {
        self.require_member(conversation_id, user_id).await?;
        let members = self.chat_repo.find_members(conversation_id).await?;
        Ok(ConversationMemberResponse::list(&members))
    }

    pub async fn add_member(
        &self,
        conversation_id: i64,
        actor_id: i64,
        req: AddConversationMemberRequest,
    ) -> AppResult<ConversationResponse> {
        let conversation = self.chat_repo.find_conversation(conversation_id).await?;
        if conversation.is_private() {
            return Err(AppError::BadRequest(
                "cannot add members to a private conversation".to_string(),
            ));
        }
        self.require_privileged(conversation_id, actor_id).await?;
        self.ensure_user_exists(req.user_id).await?;

        let role = if req.role.trim().to_lowercase() == chat_role::ADMIN {
            chat_role::ADMIN
        } else {
            chat_role::MEMBER
        };
        let mut member =
            ConversationMember::new(conversation_id, req.user_id, role.to_string(), true);
        self.chat_repo.add_member(&mut member).await?;

        let full = self.chat_repo.find_conversation(conversation_id).await?;
        Ok(ConversationResponse::from_entity(&full))
    }

    pub async fn remove_member(
        &self,
        conversation_id: i64,
        actor_id: i64,
        target_id: i64,
    ) -> AppResult<()> {
        let conversation = self.chat_repo.find_conversation(conversation_id).await?;
        if conversation.is_private() {
            return Err(AppError::BadRequest(
                "cannot remove members from a private conversation".to_string(),
            ));
        }
        if target_id == conversation.owner_id {
            return Err(AppError::BadRequest(
                "the owner cannot be removed; delete the conversation instead".to_string(),
            ));
        }
        // Leaving yourself is always allowed; removing others requires privilege.
        if target_id != actor_id {
            self.require_privileged(conversation_id, actor_id).await?;
        } else {
            self.require_member(conversation_id, actor_id).await?;
        }
        self.chat_repo.remove_member(conversation_id, target_id).await
    }

    pub async fn update_member(
        &self,
        conversation_id: i64,
        actor_id: i64,
        target_id: i64,
        req: UpdateConversationMemberRequest,
    ) -> AppResult<ConversationMemberResponse> {
        let conversation = self.chat_repo.find_conversation(conversation_id).await?;
        if conversation.is_private() {
            return Err(AppError::BadRequest(
                "permissions cannot be changed in a private conversation".to_string(),
            ));
        }
        self.require_privileged(conversation_id, actor_id).await?;
        if target_id == conversation.owner_id {
            return Err(AppError::BadRequest("the owner's role cannot be changed".to_string()));
        }

        let mut member = self
            .chat_repo
            .find_member(conversation_id, target_id)
            .await?
            .ok_or_else(|| AppError::NotFound("member not found".to_string()))?;

        let role = req.role.trim().to_lowercase();
        if !role.is_empty() {
            member.role = match role.as_str() {
                chat_role::ADMIN => chat_role::ADMIN.to_string(),
                chat_role::MEMBER => chat_role::MEMBER.to_string(),
                _ => {
                    return Err(AppError::BadRequest(
                        "role must be 'admin' or 'member'".to_string(),
                    ))
                }
            };
        }
        if let Some(can_send) = req.can_send {
            member.can_send = can_send;
        }
        self.chat_repo.update_member(&member).await?;
        Ok(ConversationMemberResponse::from_entity(&member))
    }

    // ---- Messages ----

    pub async fn send_message(
        &self,
        conversation_id: i64,
        sender_id: i64,
        req: SendChatMessageRequest,
    ) -> AppResult<ChatMessageResponse> {
        let member = self.require_member(conversation_id, sender_id).await?;
        if !member.may_send() {
            return Err(AppError::Forbidden(
                "you do not have permission to send messages in this conversation".to_string(),
            ));
        }

        let message_type = req.message_type.trim().to_lowercase();
        let valid_types = [
            chat_message_type::TEXT,
            chat_message_type::VOICE,
            chat_message_type::IMAGE,
            chat_message_type::SONG,
        ];
        if !valid_types.contains(&message_type.as_str()) {
            return Err(AppError::BadRequest(format!(
                "invalid message_type '{message_type}' (expected text, voice, image or song)"
            )));
        }

        let mut music_ids = Vec::new();
        let mut media_url = req.media_url.clone();
        let mut content = req.content.clone();

        if message_type == chat_message_type::SONG {
            if !req.music_ids.is_empty() {
                let musics = self
                    .music_repo
                    .find_by_ids(&req.music_ids)
                    .await
                    .map_err(|_| AppError::BadRequest("failed to validate music IDs".to_string()))?;
                if musics.len() != req.music_ids.len() {
                    return Err(AppError::BadRequest(
                        "one or more music IDs do not exist".to_string(),
                    ));
                }

                // Concatenate titles for media_url
                let titles: Vec<String> = musics.iter().map(|m| m.title.clone()).collect();
                let joined = titles.join(", ");
                media_url = if joined.len() > 500 {
                    let mut trunc = joined[..497].to_string();
                    trunc.push_str("...");
                    trunc
                } else {
                    joined
                };

                music_ids = req.music_ids;
            }
        }
        if message_type == chat_message_type::VOICE {
            let base64_str = if req.media_url.starts_with("data:") {
                &req.media_url
            } else {
                &req.content
            };

            if base64_str.starts_with("data:") {
                if let Some(pos) = base64_str.find(";base64,") {
                    let base64_data = &base64_str[pos + 8..];
                    use base64::Engine;
                    let decoded = base64::prelude::BASE64_STANDARD
                        .decode(base64_data.trim())
                        .map_err(|e| AppError::BadRequest(format!("Failed to decode base64 voice note: {}", e)))?;
                    
                    let timesecond = chrono::Utc::now().timestamp();
                    let filename = format!("voice-{}.opus", timesecond);
                    
                    let room_id = ChatMessage::room_id_for_convo(conversation_id);
                    
                    let file_key = format!("voice/{}/{}", room_id, filename);
                    
                    let _ = self.s3_service.upload_file(decoded, &file_key, "audio/opus").await?;
                    
                    content = filename;
                    media_url = String::new();
                }
            } else {
                content = base64_str.clone();
                media_url = String::new();
            }
        }

        let mut message = ChatMessage::new(
            conversation_id,
            sender_id,
            message_type,
            content,
            media_url,
            req.duration,
            music_ids,
        );
        if !message.is_valid() {
            return Err(AppError::BadRequest(
                "message has no content for its type".to_string(),
            ));
        }

        self.chat_repo.save_message(&mut message).await?;
        let saved = self.chat_repo.find_message(message.id).await?;

        self.notify_conversation(conversation_id, &saved).await;

        Ok(ChatMessageResponse::from_entity(&saved))
    }

    pub async fn get_messages(
        &self,
        conversation_id: i64,
        user_id: i64,
        page: i64,
        page_size: i64,
    ) -> AppResult<(Vec<ChatMessageResponse>, PaginationMetadata)> {
        self.require_member(conversation_id, user_id).await?;
        let _ = self.chat_repo.mark_chat_messages_as_read(conversation_id, user_id).await;
        let offset = (page - 1) * page_size;
        let (messages, total) =
            self.chat_repo.find_messages(conversation_id, offset, page_size).await?;
        Ok((
            ChatMessageResponse::list(&messages),
            PaginationMetadata::new(page, page_size, total),
        ))
    }

    pub async fn get_message(
        &self,
        conversation_id: i64,
        message_id: i64,
        user_id: i64,
    ) -> AppResult<ChatMessageResponse> {
        self.require_member(conversation_id, user_id).await?;
        let message = self.chat_repo.find_message(message_id).await?;
        if message.conversation_id != conversation_id {
            return Err(AppError::NotFound("message not found".to_string()));
        }
        Ok(ChatMessageResponse::from_entity(&message))
    }

    pub async fn delete_message(
        &self,
        conversation_id: i64,
        message_id: i64,
        user_id: i64,
    ) -> AppResult<()> {
        let member = self.require_member(conversation_id, user_id).await?;
        let message = self.chat_repo.find_message(message_id).await?;
        if message.conversation_id != conversation_id {
            return Err(AppError::NotFound("message not found".to_string()));
        }
        if message.sender_id != user_id && !member.is_privileged() {
            return Err(AppError::Forbidden(
                "you can only delete your own messages".to_string(),
            ));
        }
        self.chat_repo.delete_message(message_id).await
    }

    pub async fn edit_message(
        &self,
        conversation_id: i64,
        message_id: i64,
        user_id: i64,
        content: String,
    ) -> AppResult<ChatMessageResponse> {
        let member = self.require_member(conversation_id, user_id).await?;
        let message = self.chat_repo.find_message(message_id).await?;
        if message.conversation_id != conversation_id {
            return Err(AppError::NotFound("message not found".to_string()));
        }
        if message.sender_id != user_id && !member.is_privileged() {
            return Err(AppError::Forbidden(
                "you can only edit your own messages".to_string(),
            ));
        }
        self.chat_repo.update_message(message_id, &content).await?;
        let updated = self.chat_repo.find_message(message_id).await?;
        Ok(ChatMessageResponse::from_entity(&updated))
    }

    /// Best-effort push notification to the other participants.
    async fn notify_conversation(&self, conversation_id: i64, message: &ChatMessage) {
        let sender_name = message
            .sender
            .as_ref()
            .map(|u| u.name.clone())
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| "Someone".to_string());

        let body = match message.message_type.as_str() {
            chat_message_type::TEXT => message.content.clone(),
            chat_message_type::VOICE => "Sent a voice message".to_string(),
            chat_message_type::IMAGE => "Sent an image".to_string(),
            chat_message_type::SONG => {
                if message.musics.len() == 1 {
                    "Shared a song".to_string()
                } else {
                    format!("Shared {} songs", message.musics.len())
                }
            }
            _ => "New message".to_string(),
        };

        let mut data = HashMap::new();
        data.insert("type".to_string(), "chat".to_string());
        data.insert("conversation_id".to_string(), conversation_id.to_string());
        data.insert("message_id".to_string(), message.id.to_string());
        data.insert("sender_id".to_string(), message.sender_id.to_string());
        data.insert("message_type".to_string(), message.message_type.clone());

        let members = match self.chat_repo.find_members(conversation_id).await {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!("failed to load members for chat notification: {e}");
                return;
            }
        };
        for member in members {
            if member.user_id == message.sender_id {
                continue;
            }
            // Save unread notification to the DB
            let _ = self
                .chat_repo
                .create_chat_notification(
                    member.user_id,
                    message.sender_id,
                    conversation_id,
                    &sender_name,
                    &body,
                )
                .await;

            if let Err(e) = self
                .fcm_service
                .send_to_user(member.user_id, &sender_name, &body, data.clone())
                .await
            {
                tracing::warn!("failed to send chat notification to user {}: {e}", member.user_id);
            }
        }
    }
}
