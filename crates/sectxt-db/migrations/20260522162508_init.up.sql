CREATE TABLE "public"."messages" (
  "id" UUID PRIMARY KEY,
  "created_at" TIMESTAMPTZ NOT NULL,
  "expires_at" TIMESTAMPTZ NOT NULL,
  "salt" BYTEA NOT NULL,
  "auth_hash" BYTEA NOT NULL,
  "nonce" BYTEA NOT NULL,
  "ciphertext" BYTEA NOT NULL
);

CREATE INDEX "messages_created_at_idx" ON "public"."messages" ("created_at");
CREATE INDEX "messages_expires_at_idx" ON "public"."messages" ("expires_at");
CREATE UNIQUE INDEX "messages_auth_hash_key" ON "public"."messages" ("auth_hash");

CREATE TABLE "public"."attachments" (
  "id" UUID PRIMARY KEY,
  "message_id" UUID NOT NULL REFERENCES "messages" ("id") ON DELETE CASCADE,
  "path" TEXT NOT NULL,
  "extension" TEXT NOT NULL,
  "nonce" BYTEA NOT NULL
);

CREATE INDEX "attachments_message_id_idx" ON "public"."attachments" ("message_id");
