use chrono::{DateTime, Utc};
use sectxt_core::message::{Message, MessageBuilderError};
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum MessageModelError {
    #[error("{0}")]
    Builder(#[from] MessageBuilderError),

    #[error("{0} must be length {1}")]
    FieldLength(&'static str, usize),
}

#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct MessageModel {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub salt: Vec<u8>,
    pub auth_hash: Vec<u8>,
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
}

impl From<Message> for MessageModel {
    fn from(value: Message) -> Self {
        Self {
            id: value.id(),
            created_at: value.created_at(),
            expires_at: value.expires_at(),
            salt: value.salt().to_vec(),
            auth_hash: value.auth_hash().to_vec(),
            nonce: value.nonce().to_vec(),
            ciphertext: value.ciphertext().to_vec(),
        }
    }
}

impl TryFrom<MessageModel> for Message {
    type Error = MessageModelError;

    fn try_from(value: MessageModel) -> Result<Self, Self::Error> {
        let auth_hash: [u8; 32] = value
            .auth_hash
            .try_into()
            .map_err(|_| MessageModelError::FieldLength("authentication hash", 32))?;
        let nonce: [u8; 12] = value
            .nonce
            .try_into()
            .map_err(|_| MessageModelError::FieldLength("nonce", 12))?;
        let salt: [u8; 16] = value
            .salt
            .try_into()
            .map_err(|_| MessageModelError::FieldLength("salt", 16))?;

        Self::builder()
            .with_auth_hash(auth_hash)
            .with_ciphertext(value.ciphertext)
            .with_created_at(value.created_at)
            .with_expires_at(value.expires_at)
            .with_id(value.id)
            .with_nonce(nonce)
            .with_salt(salt)
            .build()
            .map_err(MessageModelError::Builder)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_from_message() {
        let message = Message::builder()
            .with_auth_hash([0; 32])
            .with_ciphertext([0; 128])
            .with_nonce([0; 12])
            .with_salt([0; 16])
            .build()
            .unwrap();

        let sut = MessageModel::from(message.clone());
        assert_eq!(sut.id, message.id());
        assert_eq!(sut.auth_hash, message.auth_hash());
        assert_eq!(sut.ciphertext, message.ciphertext());
        assert_eq!(sut.created_at, message.created_at());
        assert_eq!(sut.expires_at, message.expires_at());
        assert_eq!(sut.salt, message.salt());
        assert_eq!(sut.nonce, message.nonce());
    }

    #[test]
    fn test_try_from_message_model() {
        let now = Utc::now();
        let sut = MessageModel {
            id: Uuid::new_v4(),
            created_at: now,
            expires_at: now + Duration::seconds(86400),
            salt: vec![0; 16],
            auth_hash: vec![0; 32],
            nonce: vec![0; 12],
            ciphertext: vec![0; 128],
        };

        let result = Message::try_from(sut.clone()).unwrap();
        assert_eq!(sut.id, result.id());
        assert_eq!(sut.auth_hash, result.auth_hash());
        assert_eq!(sut.ciphertext, result.ciphertext());
        assert_eq!(sut.created_at, result.created_at());
        assert_eq!(sut.expires_at, result.expires_at());
        assert_eq!(sut.nonce, result.nonce());
        assert_eq!(sut.salt, result.salt());
    }

    #[test]
    fn test_try_from_message_model_invalid_field_length() {
        let sut = MessageModel {
            id: Uuid::new_v4(),
            created_at: Utc::now(),
            expires_at: Utc::now(),
            salt: vec![0; 16],
            auth_hash: vec![0; 31],
            nonce: vec![0; 12],
            ciphertext: vec![0; 128],
        };
        assert!(Message::try_from(sut).is_err());

        let sut = MessageModel {
            id: Uuid::new_v4(),
            created_at: Utc::now(),
            expires_at: Utc::now(),
            salt: vec![0; 16],
            auth_hash: vec![0; 32],
            nonce: vec![0; 10],
            ciphertext: vec![0; 128],
        };
        assert!(Message::try_from(sut).is_err());
    }
}
