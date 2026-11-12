-- V6.2 (TS §52, v6.2implementation D-462): the AI plan's usage, the long
-- path's grants (used from V6.4) and the month's spend. Additive only.
-- IF NOT EXISTS: after a rollback to V6.1 (its record of this version
-- deleted, the tables kept; v6.2implementation §14.4), V6.2 runs it again.
CREATE TABLE IF NOT EXISTS ai_usage (
  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  month   DATE NOT NULL,
  kind    TEXT NOT NULL CHECK (kind IN ('plan','shorts','chat')),
  count   INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (user_id, month, kind)
);
CREATE TABLE IF NOT EXISTS ai_shorts_grants (
  user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  month       DATE NOT NULL,
  prefix_hash BYTEA NOT NULL,                -- hash of the client's sourceHashPrefix; no content
  PRIMARY KEY (user_id, month, prefix_hash)
);
CREATE TABLE IF NOT EXISTS ai_spend (
  month     DATE PRIMARY KEY,
  micro_usd BIGINT NOT NULL DEFAULT 0
);
