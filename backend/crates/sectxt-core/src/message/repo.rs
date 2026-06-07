use crate::message::Message;
use uuid::Uuid;

#[derive(Clone, Debug, thiserror::Error)]
pub enum MessageRepoError {
    #[error("{0}")]
    Database(String),

    #[error("{0}")]
    Model(String),
}

#[async_trait::async_trait]
pub trait MessageRepo: Send + Sync {
    async fn clean(&self) -> Result<u64, MessageRepoError>;
    async fn create(&self, message: Message) -> Result<Uuid, MessageRepoError>;
    async fn delete(&self, id: Uuid) -> Result<Option<Message>, MessageRepoError>;
    async fn get(&self, id: Uuid) -> Result<Option<Message>, MessageRepoError>;
}
