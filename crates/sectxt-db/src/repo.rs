use crate::attachment::AttachmentModel;
use crate::ext::ResultExt;
use crate::message::MessageModel;
use sectxt_core::attachment::Attachment;
use sectxt_core::message::{Message, MessageWithAttachments};
use sectxt_core::repo::{MessageRepo, MessageRepoError};
use sqlx::PgPool;
use uuid::Uuid;

pub struct PostgresMessageRepo {
    pool: PgPool,
}

impl PostgresMessageRepo {
    #[inline]
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl MessageRepo for PostgresMessageRepo {
    async fn create(&self, mwa: MessageWithAttachments) -> Result<Uuid, MessageRepoError> {
        let mut tx = self.pool.begin().await.map_core_err()?;
        let sql = r#"
            INSERT INTO "public"."messages" ("id", "created_at", "expires_at", "salt", "auth_hash", "nonce", "ciphertext")
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING "id";"#;

        let message_id = sqlx::query_scalar(sql)
            .bind(mwa.message.id())
            .bind(mwa.message.created_at())
            .bind(mwa.message.expires_at())
            .bind(mwa.message.salt())
            .bind(mwa.message.auth_hash())
            .bind(mwa.message.nonce())
            .bind(mwa.message.ciphertext())
            .fetch_one(&mut *tx)
            .await
            .map_core_err()?;

        if !mwa.attachments.is_empty() {
            let sql = r#"
                INSERT INTO "public"."attachments" ("id", "message_id", "path", "extension", "nonce")
                VALUES ($1, $2, $3, $4, $5);"#;

            for attachment in mwa.attachments {
                sqlx::query(sql)
                    .bind(attachment.id())
                    .bind(message_id)
                    .bind(attachment.path())
                    .bind(attachment.extension())
                    .bind(attachment.nonce())
                    .execute(&mut *tx)
                    .await
                    .map_core_err()?;
            }
        }
        tx.commit().await.map_core_err()?;
        Ok(message_id)
    }

    async fn exists(&self, id: Uuid) -> Result<bool, MessageRepoError> {
        let sql = r#"
            SELECT EXISTS (
                SELECT 1
                FROM "public"."messages"
                WHERE "id" = $1 AND "expires_at" > NOW()
            );"#;

        sqlx::query_scalar(sql)
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| MessageRepoError::Database(e.to_string()))
    }

    async fn consume(&self, id: Uuid, auth_hash: [u8; 32]) -> Result<Option<MessageWithAttachments>, MessageRepoError> {
        let mut tx = self.pool.begin().await.map_core_err()?;
        let sql = r#"
            DELETE FROM "public"."attachments"
            WHERE "message_id" = $1 AND EXISTS (
                SELECT 1 FROM "public"."messages"
                WHERE "id" = $1 AND "auth_hash" = $2 AND "expires_at" > NOW()
            ) RETURNING "id", "message_id", "path", "extension", "nonce";"#;
        let attachments = sqlx::query_as::<_, AttachmentModel>(sql)
            .bind(id)
            .bind(auth_hash)
            .fetch_all(&mut *tx)
            .await
            .map_core_err()?;

        let sql = r#"
            DELETE FROM "public"."messages"
            WHERE "id" = $1 AND "auth_hash" = $2 AND "expires_at" > NOW()
            RETURNING "id", "created_at", "expires_at", "salt", "auth_hash", "nonce", "ciphertext";"#;
        let Some(message) = sqlx::query_as::<_, MessageModel>(sql)
            .bind(id)
            .bind(auth_hash)
            .fetch_optional(&mut *tx)
            .await
            .map_core_err()?
        else {
            return Ok(None);
        };

        let message = Message::try_from(message).map_core_err()?;
        let attachments = attachments
            .into_iter()
            .map(Attachment::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map_core_err()?;

        tx.commit().await.map_core_err()?;
        Ok(Some(MessageWithAttachments { message, attachments }))
    }

    async fn delete_expired(&self) -> Result<u64, MessageRepoError> {
        let sql = r#"
            DELETE FROM "public"."messages"
            WHERE "expires_at" < NOW();"#;

        sqlx::query(sql)
            .execute(&self.pool)
            .await
            .map_err(|e| MessageRepoError::Database(e.to_string()))
            .map(|r| r.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};
    use sectxt_core::attachment::AttachmentBuilder;
    use sectxt_core::message::MessageBuilder;

    #[sqlx::test(migrations = "./migrations")]
    async fn test_exists(pool: PgPool) {
        let id = Uuid::new_v4();
        let sut = PostgresMessageRepo::new(pool.clone());
        assert!(!sut.exists(id).await.unwrap());

        let message = create_message(id);
        insert_message(&pool, &message).await;
        assert!(sut.exists(id).await.unwrap());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_create(pool: PgPool) {
        let message_id = Uuid::new_v4();
        assert!(!message_exists(&pool, message_id).await);

        let message = create_message(message_id);
        let attachment_id = Uuid::new_v4();
        let attachment = create_attachment(attachment_id, message_id);
        let sut = PostgresMessageRepo::new(pool.clone());
        let result_id = sut
            .create(MessageWithAttachments::new(message, [attachment]))
            .await
            .unwrap();
        assert_eq!(result_id, message_id);
        assert!(message_exists(&pool, message_id).await);
        assert!(attachment_exists(&pool, attachment_id).await);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_consume(pool: PgPool) {
        let message_id = Uuid::new_v4();
        let message = create_message(message_id);
        insert_message(&pool, &message).await;
        let attachment_id = Uuid::new_v4();
        let attachment = create_attachment(attachment_id, message_id);
        insert_attachment(&pool, &attachment).await;

        let sut = PostgresMessageRepo::new(pool.clone());
        let result = sut.consume(message_id, [5; 32]).await.unwrap().unwrap();
        assert!(!message_exists(&pool, message_id).await);
        assert!(!attachment_exists(&pool, attachment_id).await);
        assert_eq!(result.message.id(), message_id);
        assert_eq!(result.message.auth_hash(), message.auth_hash());
        assert_eq!(result.message.ciphertext(), message.ciphertext());
        assert_eq!(result.message.nonce(), message.nonce());
        assert_eq!(result.attachments.len(), 1);
        assert_eq!(result.attachments[0].id(), attachment_id);
        assert_eq!(result.attachments[0].message_id(), message_id);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_consume_invalid_auth(pool: PgPool) {
        let message_id = Uuid::new_v4();
        let message = create_message(message_id);
        insert_message(&pool, &message).await;
        let attachment_id = Uuid::new_v4();
        let attachment = create_attachment(attachment_id, message_id);
        insert_attachment(&pool, &attachment).await;

        let sut = PostgresMessageRepo::new(pool);
        assert!(sut.consume(message_id, [0; 32]).await.unwrap().is_none());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_delete_expired(pool: PgPool) {
        let id = Uuid::new_v4();
        let created_at = DateTime::<Utc>::from_timestamp(0, 0).unwrap();
        let message = MessageBuilder::new()
            .with_auth_hash([0; 32])
            .with_ciphertext([0; 128])
            .with_created_at(created_at)
            .with_expires_at(created_at + chrono::Duration::seconds(100))
            .with_id(id)
            .with_nonce([0; 12])
            .with_salt([0; 16])
            .build()
            .unwrap();
        insert_message(&pool, &message).await;

        let sut = PostgresMessageRepo::new(pool.clone());
        let result = sut.delete_expired().await.unwrap();
        assert_eq!(result, 1);
        assert!(!message_exists(&pool, id).await);
    }

    fn create_message(id: Uuid) -> Message {
        MessageBuilder::new()
            .with_auth_hash([5; 32])
            .with_ciphertext([5; 128])
            .with_id(id)
            .with_nonce([5; 12])
            .with_salt([5; 16])
            .build()
            .unwrap()
    }

    fn create_attachment(id: Uuid, message_id: Uuid) -> Attachment {
        AttachmentBuilder::new()
            .with_extension("txt".into())
            .with_id(id)
            .with_message_id(message_id)
            .with_nonce([5; 12])
            .with_path("test.txt".into())
            .build()
            .unwrap()
    }

    async fn insert_message(pool: &PgPool, message: &Message) {
        let sql = r#"
            INSERT INTO "public"."messages" ("id", "created_at", "expires_at", "salt", "auth_hash", "nonce", "ciphertext")
            VALUES ($1, $2, $3, $4, $5, $6, $7);"#;

        sqlx::query(sql)
            .bind(message.id())
            .bind(message.created_at())
            .bind(message.expires_at())
            .bind(message.salt())
            .bind(message.auth_hash())
            .bind(message.nonce())
            .bind(message.ciphertext())
            .execute(pool)
            .await
            .unwrap();
    }

    async fn insert_attachment(pool: &PgPool, attachment: &Attachment) {
        let sql = r#"
            INSERT INTO "public"."attachments" ("id", "message_id", "path", "extension", "nonce")
            VALUES ($1, $2, $3, $4, $5);"#;

        sqlx::query(sql)
            .bind(attachment.id())
            .bind(attachment.message_id())
            .bind(attachment.path())
            .bind(attachment.extension())
            .bind(attachment.nonce())
            .execute(pool)
            .await
            .unwrap();
    }

    async fn message_exists(pool: &PgPool, id: Uuid) -> bool {
        let sql = r#"
            SELECT EXISTS (
                SELECT 1
                FROM "public"."messages"
                WHERE "id" = $1
            );"#;

        sqlx::query_scalar(sql).bind(id).fetch_one(pool).await.unwrap()
    }

    async fn attachment_exists(pool: &PgPool, id: Uuid) -> bool {
        let sql = r#"
            SELECT EXISTS (
                SELECT 1
                FROM "public"."attachments"
                WHERE "id" = $1
            );"#;

        sqlx::query_scalar(sql).bind(id).fetch_one(pool).await.unwrap()
    }
}
