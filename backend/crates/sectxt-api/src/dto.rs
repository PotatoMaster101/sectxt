use sectxt_core::message::{Message, MessageError};
use serde::{Deserialize, Serialize};
use sqlx::types::chrono;
use std::time::Duration;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMessageDto {
    pub burn_on_read: bool,
    pub has_password: bool,
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; 12],
    pub salt: [u8; 16],
    #[serde(default)]
    pub ttl_seconds: Option<u64>,
}

impl TryFrom<CreateMessageDto> for Message {
    type Error = MessageError;

    fn try_from(value: CreateMessageDto) -> Result<Self, Self::Error> {
        let now = chrono::Utc::now();
        Self::builder()
            .created_at(now)
            .expires_at(now + Duration::from_secs(value.ttl_seconds.unwrap_or(3600)))
            .burn_on_read(value.burn_on_read)
            .ciphertext(value.ciphertext)
            .nonce(value.nonce)
            .salt(value.salt)
            .build()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadMessageDto {
    pub has_password: bool,
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
    pub salt: Vec<u8>,
}

impl From<Message> for ReadMessageDto {
    fn from(value: Message) -> Self {
        Self {
            has_password: value.has_password(),
            ciphertext: value.ciphertext().to_vec(),
            nonce: value.nonce().to_vec(),
            salt: value.salt().to_vec(),
        }
    }
}
