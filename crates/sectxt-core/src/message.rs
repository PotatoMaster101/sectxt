use crate::attachment::Attachment;
use crate::crypto::{CryptoError, DerivedKeys, aes_decrypt, aes_encrypt};
use aes_gcm::aead::OsRng;
use aes_gcm::aead::rand_core::RngCore;
use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

const MAX_CIPHERTEXT_LENGTH: usize = 10000;

#[derive(Debug, thiserror::Error)]
pub enum MessageError {
    #[error("{0}")]
    Crypto(#[from] CryptoError),

    #[error("incorrect password")]
    IncorrectPassword,

    #[error("{0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

/// Represents an encrypted message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Message {
    /// The ID of the message.
    id: Uuid,

    /// The UTC creation timestamp of the message.
    created_at: DateTime<Utc>,

    /// The UTC expiration timestamp of the message.
    expires_at: DateTime<Utc>,

    /// The encryption salt.
    salt: [u8; 16],

    /// The hashed client verification key used by the server for identity validation.
    auth_hash: [u8; 32],

    /// The IV for client-side decryption.
    nonce: [u8; 12],

    /// The encrypted message bytes.
    ciphertext: Vec<u8>,
}

impl Message {
    pub fn encrypt(plaintext: &str, password: &str) -> Result<Self, MessageError> {
        let mut salt = [0u8; 16];
        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut salt);
        OsRng.fill_bytes(&mut nonce);

        let keys = DerivedKeys::derive(password, &salt)?;
        let ciphertext = aes_encrypt(plaintext.as_bytes(), &nonce, &keys.enc_key)?;
        let created_at = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            created_at,
            expires_at: created_at + Duration::days(1),
            salt,
            auth_hash: keys.auth_hash(),
            nonce,
            ciphertext,
        })
    }

    pub fn decrypt(self, password: &str) -> Result<String, MessageError> {
        let keys = DerivedKeys::derive(password, &self.salt)?;
        if keys.auth_hash() != self.auth_hash {
            return Err(MessageError::IncorrectPassword);
        }

        let plaintext = aes_decrypt(&self.ciphertext, &self.nonce, &keys.enc_key)?;
        Ok(String::from_utf8(plaintext)?)
    }

    #[inline]
    #[must_use]
    pub fn builder() -> MessageBuilder {
        MessageBuilder::new()
    }

    #[inline]
    #[must_use]
    pub fn expired(&self) -> bool {
        self.expires_at < Utc::now()
    }

    #[inline]
    #[must_use]
    pub const fn id(&self) -> Uuid {
        self.id
    }

    #[inline]
    #[must_use]
    pub const fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    #[inline]
    #[must_use]
    pub const fn expires_at(&self) -> DateTime<Utc> {
        self.expires_at
    }

    #[inline]
    #[must_use]
    pub const fn salt(&self) -> &[u8] {
        self.salt.as_slice()
    }

    #[inline]
    #[must_use]
    pub const fn auth_hash(&self) -> &[u8] {
        self.auth_hash.as_slice()
    }

    #[inline]
    #[must_use]
    pub const fn nonce(&self) -> &[u8] {
        self.nonce.as_slice()
    }

    #[inline]
    #[must_use]
    pub const fn ciphertext(&self) -> &[u8] {
        self.ciphertext.as_slice()
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum MessageBuilderError {
    #[error("ciphertext length invalid")]
    InvalidCiphertext,

    #[error("expire time must be greater than creation time")]
    InvalidExpireTime,

    #[error("missing authentication hash")]
    MissingAuthHash,

    #[error("missing ciphertext")]
    MissingCiphertext,

    #[error("missing nonce")]
    MissingNonce,

    #[error("missing salt")]
    MissingSalt,
}

#[derive(Default)]
pub struct MessageBuilder {
    auth_hash: Option<[u8; 32]>,
    ciphertext: Option<Vec<u8>>,
    created_at: Option<DateTime<Utc>>,
    expires_at: Option<DateTime<Utc>>,
    id: Option<Uuid>,
    nonce: Option<[u8; 12]>,
    salt: Option<[u8; 16]>,
}

impl MessageBuilder {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    #[must_use]
    pub const fn with_auth_hash(mut self, auth_hash: [u8; 32]) -> Self {
        self.auth_hash = Some(auth_hash);
        self
    }

    #[inline]
    #[must_use]
    pub fn with_ciphertext(mut self, ciphertext: impl IntoIterator<Item = u8>) -> Self {
        self.ciphertext = Some(ciphertext.into_iter().collect());
        self
    }

    #[inline]
    #[must_use]
    pub const fn with_created_at(mut self, created_at: DateTime<Utc>) -> Self {
        self.created_at = Some(created_at);
        self
    }

    #[inline]
    #[must_use]
    pub const fn with_id(mut self, id: Uuid) -> Self {
        self.id = Some(id);
        self
    }

    #[inline]
    #[must_use]
    pub const fn with_expires_at(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    #[inline]
    #[must_use]
    pub const fn with_nonce(mut self, nonce: [u8; 12]) -> Self {
        self.nonce = Some(nonce);
        self
    }

    #[inline]
    #[must_use]
    pub const fn with_salt(mut self, salt: [u8; 16]) -> Self {
        self.salt = Some(salt);
        self
    }

    pub fn build(self) -> Result<Message, MessageBuilderError> {
        let auth_hash = self.auth_hash.ok_or(MessageBuilderError::MissingAuthHash)?;
        let ciphertext = self.ciphertext.ok_or(MessageBuilderError::MissingCiphertext)?;
        let created_at = self.created_at.unwrap_or_else(Utc::now);
        let expires_at = self.expires_at.unwrap_or(created_at + Duration::days(1));
        let id = self.id.unwrap_or_else(Uuid::new_v4);
        let nonce = self.nonce.ok_or(MessageBuilderError::MissingNonce)?;
        let salt = self.salt.ok_or(MessageBuilderError::MissingSalt)?;

        if expires_at < created_at {
            return Err(MessageBuilderError::InvalidExpireTime);
        }
        if ciphertext.is_empty() || ciphertext.len() > MAX_CIPHERTEXT_LENGTH {
            return Err(MessageBuilderError::InvalidCiphertext);
        }

        Ok(Message {
            id,
            created_at,
            expires_at,
            salt,
            auth_hash,
            nonce,
            ciphertext,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessageWithAttachments {
    pub message: Message,
    pub attachments: Vec<Attachment>,
}

impl MessageWithAttachments {
    #[inline]
    #[must_use]
    pub fn new(message: Message, attachments: impl IntoIterator<Item = Attachment>) -> Self {
        Self {
            message,
            attachments: attachments.into_iter().collect(),
        }
    }
}

#[cfg(test)]
mod message_tests {
    use super::*;

    #[test]
    fn test_expired() {
        let mut sut = get_sut();
        assert!(!sut.expired());

        sut.expires_at = Utc::now() - Duration::seconds(100);
        assert!(sut.expired());
    }

    #[test]
    fn test_encrypt() {
        let plaintext = "plaintext";
        let password = "password";
        let sut1 = Message::encrypt(plaintext, password).unwrap();
        assert_ne!(sut1.id, Uuid::nil());
        assert!(sut1.expires_at > sut1.created_at);
        assert_ne!(sut1.ciphertext, plaintext.as_bytes());

        let sut2 = Message::encrypt(plaintext, password).unwrap();
        assert_ne!(sut1.salt, sut2.salt);
        assert_ne!(sut1.nonce, sut2.nonce);
        assert_ne!(sut1.auth_hash, sut2.auth_hash);
        assert_ne!(sut1.ciphertext, sut2.ciphertext);
    }

    #[test]
    fn test_decrypt() {
        let plaintext = "plaintext";
        let password = "password";
        let sut = Message::encrypt(plaintext, password).unwrap();
        let decrypted = sut.decrypt(password).unwrap();
        assert_eq!(decrypted, plaintext);

        let sut = Message::encrypt(plaintext, "wrong password").unwrap();
        assert!(sut.decrypt(password).is_err());
    }

    #[inline]
    #[must_use]
    fn get_sut() -> Message {
        MessageBuilder::new()
            .with_auth_hash([0; 32])
            .with_ciphertext([0; 128])
            .with_nonce([0; 12])
            .with_salt([0; 16])
            .build()
            .unwrap()
    }
}

#[cfg(test)]
mod message_builder_tests {
    use super::*;

    #[test]
    fn test_build_all_fields() {
        let created_at = Utc::now();
        let expires_at = created_at + Duration::seconds(9999);
        let id = Uuid::new_v4();
        let sut = get_sut()
            .with_created_at(created_at)
            .with_expires_at(expires_at)
            .with_id(id)
            .build()
            .unwrap();

        assert_eq!(sut.id, id);
        assert_eq!(sut.created_at, created_at);
        assert_eq!(sut.expires_at, expires_at);
        assert_eq!(sut.auth_hash, [0; 32]);
        assert_eq!(sut.nonce, [0; 12]);
        assert_eq!(sut.ciphertext.len(), 128);
    }

    #[test]
    fn test_build_mandatory_fields() {
        let sut = get_sut().build().unwrap();
        assert_ne!(sut.id, Uuid::nil());
        assert!(sut.created_at > DateTime::<Utc>::MIN_UTC);
        assert_eq!(sut.expires_at, sut.created_at + Duration::days(1));
        assert_eq!(sut.auth_hash, [0; 32]);
        assert_eq!(sut.nonce, [0; 12]);
        assert_eq!(sut.ciphertext.len(), 128);
    }

    #[test]
    fn test_build_invalid_ciphertext() {
        let sut = get_sut().with_ciphertext([0; 0]).build();
        assert_eq!(sut, Err(MessageBuilderError::InvalidCiphertext));

        let sut = get_sut().with_ciphertext(vec![0; MAX_CIPHERTEXT_LENGTH + 1]).build();
        assert_eq!(sut, Err(MessageBuilderError::InvalidCiphertext));
    }

    #[test]
    fn test_build_invalid_expires_at() {
        let now = Utc::now();
        let sut = get_sut().with_expires_at(now - Duration::seconds(100)).build();
        assert_eq!(sut, Err(MessageBuilderError::InvalidExpireTime));
    }

    #[test]
    fn test_build_missing_auth_hash() {
        let sut = MessageBuilder::new()
            .with_ciphertext([0; 128])
            .with_nonce([0; 12])
            .with_salt([0; 16])
            .build();
        assert_eq!(sut, Err(MessageBuilderError::MissingAuthHash));
    }

    #[test]
    fn test_build_missing_ciphertext() {
        let sut = MessageBuilder::new()
            .with_auth_hash([0; 32])
            .with_nonce([0; 12])
            .with_salt([0; 16])
            .build();
        assert_eq!(sut, Err(MessageBuilderError::MissingCiphertext));
    }

    #[test]
    fn test_build_missing_nonce() {
        let sut = MessageBuilder::new()
            .with_auth_hash([0; 32])
            .with_ciphertext([0; 128])
            .with_salt([0; 16])
            .build();
        assert_eq!(sut, Err(MessageBuilderError::MissingNonce));
    }

    #[test]
    fn test_build_missing_salt() {
        let sut = MessageBuilder::new()
            .with_auth_hash([0; 32])
            .with_ciphertext([0; 128])
            .with_nonce([0; 12])
            .build();
        assert_eq!(sut, Err(MessageBuilderError::MissingSalt));
    }

    #[inline]
    #[must_use]
    fn get_sut() -> MessageBuilder {
        MessageBuilder::new()
            .with_auth_hash([0; 32])
            .with_ciphertext([0; 128])
            .with_nonce([0; 12])
            .with_salt([0; 16])
    }
}
