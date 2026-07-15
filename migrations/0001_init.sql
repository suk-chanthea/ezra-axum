-- Initial schema mirroring the GORM AutoMigrate models from the Go service.
-- IDs use BIGSERIAL/BIGINT to match GORM's mapping of Go `uint` on PostgreSQL.

CREATE TABLE IF NOT EXISTS users (
    id             BIGSERIAL PRIMARY KEY,
    username       VARCHAR(100) NOT NULL,
    name           VARCHAR(100) NOT NULL,
    profile        VARCHAR(255) NOT NULL DEFAULT '',
    email          VARCHAR(100) NOT NULL UNIQUE,
    email_verified BOOLEAN      NOT NULL DEFAULT FALSE,
    phone          VARCHAR(50)  NOT NULL DEFAULT '',
    password       VARCHAR(255) NOT NULL DEFAULT '',
    role           VARCHAR(20)  NOT NULL DEFAULT 'user',
    token          VARCHAR(255) NOT NULL DEFAULT '',
    provider       VARCHAR(50)  NOT NULL DEFAULT 'local',
    provider_id    VARCHAR(255) NOT NULL DEFAULT '',
    birthday       DATE,
    church_id      BIGINT,
    church_status  VARCHAR(20)  NOT NULL DEFAULT 'pending',
    band_id        BIGINT,
    bio            TEXT         NOT NULL DEFAULT '',
    created_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_users_provider_id ON users (provider_id);
CREATE INDEX IF NOT EXISTS idx_users_church_id ON users (church_id);
CREATE INDEX IF NOT EXISTS idx_users_band_id ON users (band_id);

CREATE TABLE IF NOT EXISTS musics (
    id          BIGSERIAL PRIMARY KEY,
    title       VARCHAR(255) NOT NULL,
    artist      VARCHAR(255) NOT NULL DEFAULT '',
    album       VARCHAR(255) NOT NULL DEFAULT '',
    genre       VARCHAR(100) NOT NULL DEFAULT '',
    duration    INTEGER      NOT NULL DEFAULT 0,
    bpm         INTEGER      NOT NULL DEFAULT 0,
    key         VARCHAR(10)  NOT NULL DEFAULT '',
    cover       VARCHAR(255) NOT NULL DEFAULT '',
    lyrics      TEXT         NOT NULL DEFAULT '',
    description TEXT         NOT NULL DEFAULT '',
    user_id     BIGINT       NOT NULL,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS bands (
    id          BIGSERIAL PRIMARY KEY,
    name        VARCHAR(255) NOT NULL,
    description TEXT         NOT NULL DEFAULT '',
    cover       VARCHAR(255) NOT NULL DEFAULT '',
    is_public   BOOLEAN      NOT NULL DEFAULT FALSE,
    user_id     BIGINT       NOT NULL,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS band_musics (
    id            BIGSERIAL PRIMARY KEY,
    band_id       BIGINT      NOT NULL,
    music_id      BIGINT      NOT NULL,
    display_order INTEGER     NOT NULL DEFAULT 0,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS events (
    id         BIGSERIAL PRIMARY KEY,
    title      VARCHAR(255) NOT NULL,
    content    TEXT         NOT NULL DEFAULT '',
    cover      VARCHAR(255) NOT NULL DEFAULT '',
    location   TEXT         NOT NULL,
    start_time TIMESTAMPTZ  NOT NULL,
    end_time   TIMESTAMPTZ  NOT NULL,
    user_id    BIGINT       NOT NULL,
    created_at TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS event_musics (
    id            BIGSERIAL PRIMARY KEY,
    event_id      BIGINT      NOT NULL,
    music_id      BIGINT      NOT NULL,
    display_order INTEGER     NOT NULL DEFAULT 0,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS bookings (
    id         BIGSERIAL PRIMARY KEY,
    event_id   BIGINT      NOT NULL,
    user_id    BIGINT      NOT NULL,
    status     VARCHAR(50) NOT NULL DEFAULT 'pending',
    notes      TEXT        NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_bookings_event_id ON bookings (event_id);
CREATE INDEX IF NOT EXISTS idx_bookings_user_id ON bookings (user_id);

CREATE TABLE IF NOT EXISTS favorites (
    id         BIGSERIAL PRIMARY KEY,
    user_id    BIGINT      NOT NULL,
    music_id   BIGINT      NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS notifications (
    id             BIGSERIAL PRIMARY KEY,
    user_id        BIGINT,
    sender_id      BIGINT,
    band_id        BIGINT,
    recipient_type VARCHAR(20)  NOT NULL DEFAULT 'user',
    title          VARCHAR(255) NOT NULL,
    message        TEXT         NOT NULL,
    type           VARCHAR(50)  NOT NULL DEFAULT 'info',
    related_type   VARCHAR(50)  NOT NULL DEFAULT '',
    related_id     BIGINT,
    is_read        BOOLEAN      NOT NULL DEFAULT FALSE,
    read_at        TIMESTAMPTZ,
    created_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_notifications_user_id ON notifications (user_id);
CREATE INDEX IF NOT EXISTS idx_notifications_band_id ON notifications (band_id);
CREATE INDEX IF NOT EXISTS idx_notifications_recipient_type ON notifications (recipient_type);
CREATE INDEX IF NOT EXISTS idx_notifications_is_read ON notifications (is_read);
CREATE INDEX IF NOT EXISTS idx_notifications_created_at ON notifications (created_at DESC);

CREATE TABLE IF NOT EXISTS device_tokens (
    id         BIGSERIAL PRIMARY KEY,
    user_id    BIGINT      NOT NULL,
    token      TEXT        NOT NULL,
    platform   VARCHAR(20) NOT NULL,
    is_active  BOOLEAN     NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_device_tokens_user_id ON device_tokens (user_id);
CREATE INDEX IF NOT EXISTS idx_device_tokens_is_active ON device_tokens (is_active);

CREATE TABLE IF NOT EXISTS otps (
    id         BIGSERIAL PRIMARY KEY,
    email      VARCHAR(100) NOT NULL,
    code       VARCHAR(10)  NOT NULL,
    purpose    VARCHAR(50)  NOT NULL,
    expires_at TIMESTAMPTZ  NOT NULL,
    verified   BOOLEAN      NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_otps_email ON otps (email);
CREATE INDEX IF NOT EXISTS idx_otps_purpose ON otps (purpose);

CREATE TABLE IF NOT EXISTS supporters (
    id          BIGSERIAL PRIMARY KEY,
    name        VARCHAR(255) NOT NULL,
    email       VARCHAR(255) NOT NULL UNIQUE,
    phone       VARCHAR(50)  NOT NULL DEFAULT '',
    type        VARCHAR(50)  NOT NULL DEFAULT 'company',
    website     VARCHAR(255) NOT NULL DEFAULT '',
    address     TEXT         NOT NULL DEFAULT '',
    logo        VARCHAR(255) NOT NULL DEFAULT '',
    description TEXT         NOT NULL DEFAULT '',
    user_id     BIGINT,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_supporters_user_id ON supporters (user_id);

CREATE TABLE IF NOT EXISTS donations (
    id             BIGSERIAL PRIMARY KEY,
    type           VARCHAR(50)      NOT NULL,
    donor_type     VARCHAR(50)      NOT NULL,
    user_id        BIGINT,
    supporter_id   BIGINT,
    company_name   VARCHAR(255)     NOT NULL DEFAULT '',
    company_email  VARCHAR(255)     NOT NULL DEFAULT '',
    company_phone  VARCHAR(50)      NOT NULL DEFAULT '',
    amount         DOUBLE PRECISION NOT NULL,
    currency       VARCHAR(10)      NOT NULL DEFAULT 'USD',
    message        TEXT             NOT NULL DEFAULT '',
    status         VARCHAR(50)      NOT NULL DEFAULT 'pending',
    transaction_id VARCHAR(255)     NOT NULL DEFAULT '',
    payment_method VARCHAR(100)     NOT NULL DEFAULT '',
    qr_expires_at  TIMESTAMPTZ,
    event_id       BIGINT,
    created_at     TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMPTZ      NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_donations_type ON donations (type);
CREATE INDEX IF NOT EXISTS idx_donations_donor_type ON donations (donor_type);
CREATE INDEX IF NOT EXISTS idx_donations_user_id ON donations (user_id);
CREATE INDEX IF NOT EXISTS idx_donations_supporter_id ON donations (supporter_id);
CREATE INDEX IF NOT EXISTS idx_donations_status ON donations (status);
CREATE INDEX IF NOT EXISTS idx_donations_transaction_id ON donations (transaction_id);
CREATE INDEX IF NOT EXISTS idx_donations_event_id ON donations (event_id);

CREATE TABLE IF NOT EXISTS settings (
    id                        BIGSERIAL PRIMARY KEY,
    user_id                   BIGINT      NOT NULL UNIQUE,
    language                  VARCHAR(10) NOT NULL DEFAULT 'en',
    theme                     VARCHAR(20) NOT NULL DEFAULT 'light',
    notify_on_booking         BOOLEAN     NOT NULL DEFAULT TRUE,
    notify_on_music           BOOLEAN     NOT NULL DEFAULT FALSE,
    notify_on_event           BOOLEAN     NOT NULL DEFAULT TRUE,
    enable_push_notifications BOOLEAN     NOT NULL DEFAULT TRUE,
    created_at                TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at                TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS churches (
    id               BIGSERIAL PRIMARY KEY,
    name             VARCHAR(255) NOT NULL,
    address          TEXT         NOT NULL DEFAULT '',
    phone            VARCHAR(50)  NOT NULL DEFAULT '',
    email            VARCHAR(255) NOT NULL DEFAULT '',
    website          VARCHAR(255) NOT NULL DEFAULT '',
    pastor_name      VARCHAR(255) NOT NULL DEFAULT '',
    description      TEXT         NOT NULL DEFAULT '',
    logo             VARCHAR(255) NOT NULL DEFAULT '',
    established_date  TIMESTAMPTZ,
    denomination     VARCHAR(100) NOT NULL DEFAULT '',
    owner_id         BIGINT,
    created_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_churches_owner_id ON churches (owner_id);
