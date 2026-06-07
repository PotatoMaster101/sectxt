pub mod repo;

use chrono::{DateTime, Utc};
use sectxt_core::message::{Message, MessageError};
use uuid::Uuid;

#[derive(Clone, Debug, thiserror::Error)]
pub enum MessageModelError {
    #[error("{0}")]
    Domain(#[from] MessageError),

    #[error("type error: {0}")]
    Type(&'static str),
}

#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct MessageModel {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub burn_on_read: bool,
    pub has_password: bool,
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
    pub salt: Vec<u8>,
}

impl From<Message> for MessageModel {
    fn from(message: Message) -> Self {
        Self {
            id: message.id(),
            created_at: message.created_at(),
            expires_at: message.expires_at(),
            burn_on_read: message.burn_on_read(),
            has_password: message.has_password(),
            ciphertext: message.ciphertext().to_vec(),
            nonce: message.nonce().to_vec(),
            salt: message.salt().to_vec(),
        }
    }
}

impl TryFrom<MessageModel> for Message {
    type Error = MessageModelError;

    fn try_from(value: MessageModel) -> Result<Self, Self::Error> {
        let nonce = <[u8; 12]>::try_from(value.nonce).map_err(|_| MessageModelError::Type("nonce"))?;
        let salt = <[u8; 16]>::try_from(value.salt).map_err(|_| MessageModelError::Type("salt"))?;
        Ok(Self::builder()
            .id(value.id)
            .created_at(value.created_at)
            .expires_at(value.expires_at)
            .burn_on_read(value.burn_on_read)
            .has_password(value.has_password)
            .ciphertext(value.ciphertext)
            .nonce(nonce)
            .salt(salt)
            .build()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::MessageModel;
    use chrono::{Duration, Utc};
    use sectxt_core::message::Message;
    use std::assert_matches;

    #[test]
    fn test_from() {
        let id = Uuid::now_v7();
        let message = Message::builder()
            .id(id)
            .has_password(true)
            .burn_on_read(true)
            .ciphertext(vec![0; 128])
            .nonce([0; 12])
            .salt([0; 16])
            .build()
            .unwrap();

        let sut = MessageModel::from(message.clone());
        assert_eq!(sut.id, message.id());
        assert_eq!(sut.created_at, message.created_at());
        assert_eq!(sut.expires_at, message.expires_at());
        assert_eq!(sut.burn_on_read, message.burn_on_read());
        assert_eq!(sut.has_password, message.has_password());
        assert_eq!(sut.ciphertext, message.ciphertext());
        assert_eq!(sut.nonce, message.nonce());
        assert_eq!(sut.salt, message.salt());
    }

    #[test]
    fn test_try_from() {
        let now = Utc::now();
        let sut = Message::try_from(MessageModel {
            id: Uuid::now_v7(),
            created_at: now,
            expires_at: now + Duration::days(7),
            burn_on_read: true,
            has_password: false,
            ciphertext: vec![0; 128],
            nonce: vec![0; 12],
            salt: vec![0; 16],
        });
        assert!(sut.is_ok());

        let sut = Message::try_from(MessageModel {
            id: Uuid::now_v7(),
            created_at: now,
            expires_at: now,
            burn_on_read: true,
            has_password: false,
            ciphertext: vec![0; 128],
            nonce: vec![0; 11],
            salt: vec![0; 16],
        });
        assert_matches!(sut, Err(MessageModelError::Type(_)));

        let sut = Message::try_from(MessageModel {
            id: Uuid::now_v7(),
            created_at: now,
            expires_at: now,
            burn_on_read: true,
            has_password: false,
            ciphertext: vec![0; 128],
            nonce: vec![0; 12],
            salt: vec![0; 17],
        });
        assert_matches!(sut, Err(MessageModelError::Type(_)));

        let sut = Message::try_from(MessageModel {
            id: Uuid::now_v7(),
            created_at: now,
            expires_at: now - Duration::days(1),
            burn_on_read: true,
            has_password: false,
            ciphertext: vec![0; 128],
            nonce: vec![0; 12],
            salt: vec![0; 16],
        });
        assert_matches!(sut, Err(MessageModelError::Domain(_)));
    }
}
