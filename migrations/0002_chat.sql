-- Chat: private (1:1), group, and band conversations with role/permission-based membership.
-- Messages support text, voice, image, and sharing one or more songs.

CREATE TABLE IF NOT EXISTS conversations (
    id         BIGSERIAL PRIMARY KEY,
    type       VARCHAR(20)  NOT NULL DEFAULT 'group', -- private | group | band
    name       VARCHAR(255) NOT NULL DEFAULT '',
    band_id    BIGINT,
    owner_id   BIGINT       NOT NULL,
    created_at TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_conversations_type ON conversations (type);
CREATE INDEX IF NOT EXISTS idx_conversations_owner_id ON conversations (owner_id);
CREATE INDEX IF NOT EXISTS idx_conversations_band_id ON conversations (band_id);

CREATE TABLE IF NOT EXISTS conversation_members (
    id              BIGSERIAL PRIMARY KEY,
    conversation_id BIGINT      NOT NULL,
    user_id         BIGINT      NOT NULL,
    role            VARCHAR(20) NOT NULL DEFAULT 'member', -- owner | admin | member
    can_send        BOOLEAN     NOT NULL DEFAULT TRUE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (conversation_id, user_id)
);
CREATE INDEX IF NOT EXISTS idx_conversation_members_conversation_id ON conversation_members (conversation_id);
CREATE INDEX IF NOT EXISTS idx_conversation_members_user_id ON conversation_members (user_id);

CREATE TABLE IF NOT EXISTS chat_messages (
    id              BIGSERIAL PRIMARY KEY,
    conversation_id BIGINT       NOT NULL,
    sender_id       BIGINT       NOT NULL,
    message_type    VARCHAR(20)  NOT NULL DEFAULT 'text', -- text | voice | image | song
    content         TEXT         NOT NULL DEFAULT '',
    media_url       VARCHAR(512) NOT NULL DEFAULT '',
    duration        INTEGER      NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_chat_messages_conversation_id ON chat_messages (conversation_id);
CREATE INDEX IF NOT EXISTS idx_chat_messages_sender_id ON chat_messages (sender_id);
CREATE INDEX IF NOT EXISTS idx_chat_messages_created_at ON chat_messages (created_at DESC);

-- Songs attached to a chat message (supports sharing multiple songs at once).
CREATE TABLE IF NOT EXISTS chat_message_musics (
    id            BIGSERIAL PRIMARY KEY,
    message_id    BIGINT      NOT NULL,
    music_id      BIGINT      NOT NULL,
    display_order INTEGER     NOT NULL DEFAULT 0,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_chat_message_musics_message_id ON chat_message_musics (message_id);
CREATE INDEX IF NOT EXISTS idx_chat_message_musics_music_id ON chat_message_musics (music_id);
