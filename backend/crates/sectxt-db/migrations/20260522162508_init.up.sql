CREATE TABLE "public"."messages" (
  "id" UUID PRIMARY KEY,
  "created_at" TIMESTAMPTZ NOT NULL,
  "expires_at" TIMESTAMPTZ NOT NULL,
  "burn_on_read" BOOL NOT NULL DEFAULT FALSE,
  "has_password" BOOL NOT NULL DEFAULT FALSE,
  "ciphertext" BYTEA NOT NULL,
  "nonce" BYTEA NOT NULL,
  "salt" BYTEA NOT NULL
);

CREATE INDEX "messages_created_at_idx" ON "public"."messages" ("created_at");
CREATE INDEX "messages_expires_at_idx" ON "public"."messages" ("expires_at");
