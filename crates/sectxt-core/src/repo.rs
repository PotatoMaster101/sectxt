use crate::message::MessageWithAttachments;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum MessageRepoError {
    #[error("{0}")]
    Database(String),

    #[error("{0}")]
    Model(String),
}

/// Represents a message repository.
#[async_trait::async_trait]
pub trait MessageRepo: Send + Sync {
    /// Creates a new message.
    async fn create(&self, mwa: MessageWithAttachments) -> Result<Uuid, MessageRepoError>;

    /// Checks if a message with the given ID exists.
    async fn exists(&self, id: Uuid) -> Result<bool, MessageRepoError>;

    /// Returns the message with the given ID and authentication hash (if it exists) and deletes the message.
    async fn consume(&self, id: Uuid, auth_hash: [u8; 32]) -> Result<Option<MessageWithAttachments>, MessageRepoError>;

    /// Deletes all messages that have expired.
    async fn delete_expired(&self) -> Result<u64, MessageRepoError>;
}
