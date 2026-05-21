use crate::attachment::AttachmentReadDto;
use chrono::{DateTime, Utc};
use sectxt_core::crypto::hash_data;
use sectxt_core::message::{Message, MessageBuilderError, MessageWithAttachments};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum MessageDtoError {
    #[error("{0}")]
    Builder(#[from] MessageBuilderError),

    #[error("{0} must be length {1}")]
    FieldLength(&'static str, usize),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageReadDto {
    pub auth_hash: Vec<u8>,
    pub ciphertext: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub id: Uuid,
    pub nonce: Vec<u8>,
    pub salt: Vec<u8>,
}

impl From<Message> for MessageReadDto {
    fn from(value: Message) -> Self {
        Self {
            auth_hash: value.auth_hash().to_vec(),
            ciphertext: value.ciphertext().to_vec(),
            created_at: value.created_at(),
            expires_at: value.expires_at(),
            id: value.id(),
            nonce: value.nonce().to_vec(),
            salt: value.salt().to_vec(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageWriteDto {
    pub auth_token: Vec<u8>,
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
    pub salt: Vec<u8>,
}

impl TryFrom<MessageWriteDto> for Message {
    type Error = MessageDtoError;

    fn try_from(value: MessageWriteDto) -> Result<Self, Self::Error> {
        let auth_token: [u8; 32] = value
            .auth_token
            .try_into()
            .map_err(|_| MessageDtoError::FieldLength("authentication hash", 32))?;
        let nonce: [u8; 12] = value
            .nonce
            .try_into()
            .map_err(|_| MessageDtoError::FieldLength("nonce", 12))?;
        let salt: [u8; 16] = value
            .salt
            .try_into()
            .map_err(|_| MessageDtoError::FieldLength("salt", 16))?;

        Self::builder()
            .with_auth_hash(hash_data(&auth_token))
            .with_ciphertext(value.ciphertext)
            .with_nonce(nonce)
            .with_salt(salt)
            .build()
            .map_err(MessageDtoError::Builder)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageWithAttachmentsWriteDto {
    pub id: Uuid,
    pub auth_token: [u8; 32],
}

impl MessageWithAttachmentsWriteDto {
    #[inline]
    pub fn new(id: Uuid, auth_token: Vec<u8>) -> Result<Self, MessageDtoError> {
        let auth_token: [u8; 32] = auth_token
            .try_into()
            .map_err(|_| MessageDtoError::FieldLength("auth token", 32))?;
        Ok(Self { id, auth_token })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageWithAttachmentsReadDto {
    pub message: MessageReadDto,
    pub attachments: Vec<AttachmentReadDto>,
}

impl From<MessageWithAttachments> for MessageWithAttachmentsReadDto {
    fn from(mwa: MessageWithAttachments) -> Self {
        Self {
            message: MessageReadDto::from(mwa.message),
            attachments: mwa.attachments.into_iter().map(AttachmentReadDto::from).collect(),
        }
    }
}

#[cfg(test)]
mod message_read_dto_tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_from_message() {
        let id = Uuid::new_v4();
        let created_at = Utc::now();
        let expires_at = created_at + Duration::days(7);
        let message = Message::builder()
            .with_auth_hash([0; 32])
            .with_ciphertext([0; 128])
            .with_created_at(created_at)
            .with_expires_at(expires_at)
            .with_id(id)
            .with_nonce([0; 12])
            .with_salt([0; 16])
            .build()
            .unwrap();

        let sut = MessageReadDto::from(message);
        assert_eq!(sut.auth_hash, [0; 32]);
        assert_eq!(sut.ciphertext, [0; 128]);
        assert_eq!(sut.nonce, [0; 12]);
        assert_eq!(sut.salt, [0; 16]);
        assert_eq!(sut.created_at, created_at);
        assert_eq!(sut.expires_at, expires_at);
        assert_eq!(sut.id, id);
    }
}

#[cfg(test)]
mod message_write_dto_tests {
    use super::*;

    #[test]
    fn test_try_from_message_write_dto() {
        let dto = MessageWriteDto {
            auth_token: vec![0; 32],
            ciphertext: vec![0; 128],
            nonce: vec![0; 12],
            salt: vec![0; 16],
        };
        let sut = Message::try_from(dto).unwrap();
        assert_eq!(sut.auth_hash(), hash_data(&[0; 32]));
        assert_eq!(sut.ciphertext(), [0; 128]);
        assert_eq!(sut.nonce(), [0; 12]);
        assert_eq!(sut.salt(), [0; 16]);
    }

    #[test]
    fn test_try_from_message_write_dto_invalid_length() {
        let dto = MessageWriteDto {
            auth_token: vec![0; 33],
            ciphertext: vec![0; 128],
            nonce: vec![0; 12],
            salt: vec![0; 16],
        };
        assert!(Message::try_from(dto).is_err());

        let dto = MessageWriteDto {
            auth_token: vec![0; 32],
            ciphertext: vec![0; 128],
            nonce: vec![0; 100],
            salt: vec![0; 16],
        };
        assert!(Message::try_from(dto).is_err());

        let dto = MessageWriteDto {
            auth_token: vec![0; 32],
            ciphertext: vec![0; 128],
            nonce: vec![0; 12],
            salt: vec![0; 200],
        };
        assert!(Message::try_from(dto).is_err());
    }
}

#[cfg(test)]
mod message_with_attachments_write_dto_tests {
    use super::*;

    #[test]
    fn test_new() {
        let id = Uuid::new_v4();
        let auth_hash = vec![0; 32];
        let sut = MessageWithAttachmentsWriteDto::new(id, auth_hash).unwrap();
        assert_eq!(sut.id, id);
        assert_eq!(sut.auth_token, [0; 32]);
    }

    #[test]
    fn test_new_bad_length() {
        let id = Uuid::new_v4();
        let auth_hash = vec![0; 33];
        let sut = MessageWithAttachmentsWriteDto::new(id, auth_hash);
        assert!(sut.is_err());
    }
}

#[cfg(test)]
mod message_with_attachments_read_dto_tests {
    use super::*;
    use sectxt_core::attachment::Attachment;

    #[test]
    fn test_from_message_with_attachments() {
        let message = Message::builder()
            .with_auth_hash([0; 32])
            .with_ciphertext([0; 128])
            .with_nonce([0; 12])
            .with_salt([0; 16])
            .build()
            .unwrap();
        let attachment = Attachment::builder()
            .with_extension("txt".into())
            .with_message_id(message.id())
            .with_nonce([0; 12])
            .with_path("test.txt".into())
            .build()
            .unwrap();
        let mwa = MessageWithAttachments::new(message.clone(), vec![attachment.clone()]);
        let sut = MessageWithAttachmentsReadDto::from(mwa);
        assert_eq!(sut.message, MessageReadDto::from(message));
        assert_eq!(sut.attachments, vec![AttachmentReadDto::from(attachment)]);
    }
}
