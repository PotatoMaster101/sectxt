use crate::message::MessageModel;
use sectxt_core::message::Message;
use sectxt_core::message::repo::{MessageRepo, MessageRepoError};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct PgMessageRepo {
    pool: PgPool,
}

impl PgMessageRepo {
    #[inline]
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn fetch_helper(&self, sql: &'static str, id: Uuid) -> Result<Option<Message>, MessageRepoError> {
        sqlx::query_as::<_, MessageModel>(sql)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| MessageRepoError::Database(e.to_string()))?
            .map(Message::try_from)
            .transpose()
            .map_err(|e| MessageRepoError::Database(e.to_string()))
    }
}

#[async_trait::async_trait]
impl MessageRepo for PgMessageRepo {
    async fn clean(&self) -> Result<u64, MessageRepoError> {
        let result = sqlx::query(r#"DELETE FROM "public"."messages" WHERE "expires_at" <= NOW();"#)
            .execute(&self.pool)
            .await
            .map_err(|e| MessageRepoError::Database(e.to_string()))?;
        Ok(result.rows_affected())
    }

    async fn create(&self, message: Message) -> Result<Uuid, MessageRepoError> {
        let sql = r#"
            INSERT INTO "public"."messages" ("id", "created_at", "expires_at", "burn_on_read", "has_password", "ciphertext", "nonce", "salt")
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING "id";"#;

        Ok(sqlx::query_scalar::<_, Uuid>(sql)
            .bind(message.id())
            .bind(message.created_at())
            .bind(message.expires_at())
            .bind(message.burn_on_read())
            .bind(message.has_password())
            .bind(message.ciphertext())
            .bind(message.nonce())
            .bind(message.salt())
            .fetch_one(&self.pool)
            .await
            .map_err(|e| MessageRepoError::Database(e.to_string()))?)
    }

    async fn delete(&self, id: Uuid) -> Result<Option<Message>, MessageRepoError> {
        let sql = r#"
            DELETE FROM "public"."messages" WHERE "id" = $1
            RETURNING "id", "created_at", "expires_at", "burn_on_read", "has_password", "ciphertext", "nonce", "salt";"#;
        self.fetch_helper(sql, id).await
    }

    async fn get(&self, id: Uuid) -> Result<Option<Message>, MessageRepoError> {
        let sql = r#"
            WITH target AS (
                SELECT "id", "created_at", "expires_at", "burn_on_read", "has_password", "ciphertext", "nonce", "salt"
                FROM "public"."messages" WHERE "id" = $1 AND "expires_at" > NOW()
            ),
            deleted AS (
                DELETE FROM "public"."messages"
                WHERE "id" = (SELECT "id" FROM target WHERE "burn_on_read" = TRUE)
            )
            SELECT "id", "created_at", "expires_at", "burn_on_read", "has_password", "ciphertext", "nonce", "salt" FROM target;"#;
        self.fetch_helper(sql, id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    fn burnt_message() -> Message {
        Message::builder()
            .burn_on_read(true)
            .ciphertext([0; 32])
            .nonce([0; 12])
            .salt([0; 16])
            .build()
            .unwrap()
    }

    fn expired_message() -> Message {
        Message::builder()
            .created_at(Utc::now() - Duration::days(30))
            .ciphertext([3; 32])
            .nonce([0; 12])
            .salt([0; 16])
            .build()
            .unwrap()
    }

    fn good_message() -> Message {
        Message::builder()
            .ciphertext([0; 32])
            .nonce([0; 12])
            .salt([0; 16])
            .build()
            .unwrap()
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_clean_expired(pool: PgPool) {
        let sut = PgMessageRepo::new(pool);
        let _ = sut.create(expired_message()).await.unwrap();
        let result = sut.clean().await.unwrap();
        assert_eq!(result, 1);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_clean_unaffected(pool: PgPool) {
        let sut = PgMessageRepo::new(pool);
        let _ = sut.create(good_message()).await.unwrap();
        let result = sut.clean().await.unwrap();
        assert_eq!(result, 0);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_create(pool: PgPool) {
        let message = good_message();
        let sut = PgMessageRepo::new(pool);
        let result = sut.create(message.clone()).await.unwrap();
        assert_eq!(result, message.id());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_delete(pool: PgPool) {
        let message = good_message();
        let sut = PgMessageRepo::new(pool);
        let id = sut.create(message.clone()).await.unwrap();
        let result = sut.delete(id).await.unwrap().unwrap();
        assert_eq!(result.ciphertext(), message.ciphertext());
        assert_eq!(result.id(), id);
        assert_eq!(result.nonce(), message.nonce());
        assert_eq!(result.salt(), message.salt());

        let result = sut.delete(id).await.unwrap();
        assert_eq!(result, None);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_delete_not_found(pool: PgPool) {
        let sut = PgMessageRepo::new(pool);
        let _ = sut.create(good_message()).await.unwrap();
        let result = sut.delete(Uuid::now_v7()).await.unwrap();
        assert_eq!(result, None);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_get(pool: PgPool) {
        let message = Message::builder()
            .ciphertext([0; 32])
            .nonce([0; 12])
            .salt([0; 16])
            .build()
            .unwrap();

        let sut = PgMessageRepo::new(pool);
        let id = sut.create(message.clone()).await.unwrap();
        let result = sut.get(id).await.unwrap().unwrap();
        assert_eq!(result.ciphertext(), message.ciphertext());
        assert_eq!(result.nonce(), message.nonce());
        assert_eq!(result.salt(), message.salt());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_get_expired(pool: PgPool) {
        let sut = PgMessageRepo::new(pool);
        let id = sut.create(expired_message()).await.unwrap();
        let result = sut.get(id).await.unwrap();
        assert_eq!(result, None);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_get_burnt(pool: PgPool) {
        let sut = PgMessageRepo::new(pool);
        let id = sut.create(burnt_message()).await.unwrap();
        let result = sut.get(id).await.unwrap();
        assert!(result.is_some());

        let result = sut.get(id).await.unwrap();
        assert!(result.is_none());
    }
}
