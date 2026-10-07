CREATE TABLE users (
  id               UUID PRIMARY KEY,
  email            TEXT NOT NULL,
  email_normalized TEXT NOT NULL UNIQUE,
  created_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE magic_links (
  token_hash       BYTEA PRIMARY KEY,
  email_normalized TEXT NOT NULL,
  created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
  expires_at       TIMESTAMPTZ NOT NULL,
  consumed_at      TIMESTAMPTZ
);
CREATE INDEX magic_links_email_idx ON magic_links (email_normalized);

CREATE TABLE sessions (
  id                 UUID PRIMARY KEY,
  user_id            UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  refresh_token_hash BYTEA NOT NULL UNIQUE,
  prev_token_hash    BYTEA,
  prev_valid_until   TIMESTAMPTZ,
  created_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
  last_used_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
  expires_at         TIMESTAMPTZ NOT NULL,
  revoked_at         TIMESTAMPTZ
);
CREATE INDEX sessions_user_idx ON sessions (user_id);

CREATE TABLE subscriptions (
  user_id                  UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
  provider_customer_id     TEXT NOT NULL,
  provider_subscription_id TEXT NOT NULL UNIQUE,
  plan                     TEXT NOT NULL CHECK (plan IN ('creator')),
  billing_interval         TEXT NOT NULL CHECK (billing_interval IN ('monthly','annual')),
  status                   TEXT NOT NULL CHECK (status IN ('active','past_due','canceled')),
  cancel_at_period_end     BOOLEAN NOT NULL DEFAULT false,
  current_period_end       TIMESTAMPTZ NOT NULL,
  last_event_at            TIMESTAMPTZ NOT NULL,
  updated_at               TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE webhook_events (
  provider_event_id TEXT PRIMARY KEY,
  event_type        TEXT NOT NULL,
  processed_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE usage_receipts (
  export_id      UUID PRIMARY KEY,
  user_id        UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  plan_at_export TEXT NOT NULL CHECK (plan_at_export IN ('free','creator')),
  created_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX usage_receipts_user_time_idx ON usage_receipts (user_id, created_at);

CREATE TABLE analytics_events (
  id      BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  anon_id UUID NOT NULL,
  name    TEXT NOT NULL,
  props   JSONB NOT NULL,
  ts      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX analytics_events_ts_idx ON analytics_events (ts);
CREATE INDEX analytics_events_name_ts_idx ON analytics_events (name, ts);

CREATE TABLE platform_waitlist (
  email_normalized TEXT NOT NULL,
  wanted           TEXT NOT NULL CHECK (wanted IN ('launch','safari','firefox','mobile','linux')),
  created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (email_normalized, wanted)
);
