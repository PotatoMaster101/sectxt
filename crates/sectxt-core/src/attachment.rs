use uuid::Uuid;

/// Represents a file attachment in a message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attachment {
    /// The ID of the file.
    id: Uuid,

    /// The ID of the message.
    message_id: Uuid,

    /// The encrypted file path.
    path: String,

    /// The file extension.
    extension: String,

    /// The IV for client-side decryption.
    nonce: [u8; 12],
}

impl Attachment {
    #[inline]
    #[must_use]
    pub fn builder() -> AttachmentBuilder {
        AttachmentBuilder::new()
    }

    #[inline]
    #[must_use]
    pub const fn id(&self) -> Uuid {
        self.id
    }

    #[inline]
    #[must_use]
    pub const fn message_id(&self) -> Uuid {
        self.message_id
    }

    #[inline]
    #[must_use]
    pub const fn extension(&self) -> &str {
        self.extension.as_str()
    }

    #[inline]
    #[must_use]
    pub const fn path(&self) -> &str {
        self.path.as_str()
    }

    #[inline]
    #[must_use]
    pub const fn nonce(&self) -> &[u8] {
        self.nonce.as_slice()
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum AttachmentBuilderError {
    #[error("invalid extension")]
    InvalidExtension,

    #[error("invalid file path")]
    InvalidFilepath,

    #[error("missing extension")]
    MissingExtension,

    #[error("missing message id")]
    MissingMessageId,

    #[error("missing nonce")]
    MissingNonce,

    #[error("missing path")]
    MissingPath,
}

#[derive(Default)]
pub struct AttachmentBuilder {
    extension: Option<String>,
    id: Option<Uuid>,
    message_id: Option<Uuid>,
    nonce: Option<[u8; 12]>,
    path: Option<String>,
}

impl AttachmentBuilder {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    #[must_use]
    pub fn with_extension(mut self, extension: String) -> Self {
        self.extension = Some(extension);
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
    pub const fn with_message_id(mut self, message_id: Uuid) -> Self {
        self.message_id = Some(message_id);
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
    pub fn with_path(mut self, path: String) -> Self {
        self.path = Some(path);
        self
    }

    pub fn build(self) -> Result<Attachment, AttachmentBuilderError> {
        let extension = self.extension.ok_or(AttachmentBuilderError::MissingExtension)?;
        let id = self.id.unwrap_or_else(Uuid::new_v4);
        let message_id = self.message_id.ok_or(AttachmentBuilderError::MissingMessageId)?;
        let nonce = self.nonce.ok_or(AttachmentBuilderError::MissingNonce)?;
        let path = self.path.ok_or(AttachmentBuilderError::MissingPath)?;

        if extension.is_empty() {
            return Err(AttachmentBuilderError::InvalidExtension);
        }
        if path.is_empty() {
            return Err(AttachmentBuilderError::InvalidFilepath);
        }

        Ok(Attachment {
            id,
            message_id,
            path,
            extension,
            nonce,
        })
    }
}

#[cfg(test)]
mod message_file_builder_tests {
    use super::*;

    #[test]
    fn test_build_fields() {
        let id = Uuid::new_v4();
        let message_id = Uuid::new_v4();
        let sut = get_sut().with_id(id).with_message_id(message_id).build().unwrap();

        assert_eq!(sut.extension, "txt");
        assert_eq!(sut.path, "test.txt");
        assert_eq!(sut.id, id);
        assert_eq!(sut.message_id, message_id);
        assert_eq!(sut.nonce, [0; 12]);
    }

    #[test]
    fn test_build_invalid_extension() {
        let sut = get_sut().with_extension(String::new()).build();
        assert_eq!(sut, Err(AttachmentBuilderError::InvalidExtension));
    }

    #[test]
    fn test_build_invalid_path() {
        let sut = get_sut().with_path(String::new()).build();
        assert_eq!(sut, Err(AttachmentBuilderError::InvalidFilepath));
    }

    #[test]
    fn test_build_missing_path() {
        let sut = AttachmentBuilder::new()
            .with_extension("txt".into())
            .with_message_id(Uuid::new_v4())
            .with_nonce([0; 12])
            .build();
        assert_eq!(sut, Err(AttachmentBuilderError::MissingPath));
    }

    #[test]
    fn test_build_missing_extension() {
        let sut = AttachmentBuilder::new()
            .with_message_id(Uuid::new_v4())
            .with_path("test.txt".into())
            .with_nonce([0; 12])
            .build();
        assert_eq!(sut, Err(AttachmentBuilderError::MissingExtension));
    }

    #[test]
    fn test_build_missing_message_id() {
        let sut = AttachmentBuilder::new()
            .with_extension("txt".into())
            .with_path("test.txt".into())
            .with_nonce([0; 12])
            .build();
        assert_eq!(sut, Err(AttachmentBuilderError::MissingMessageId));
    }

    #[test]
    fn test_build_missing_nonce() {
        let sut = AttachmentBuilder::new()
            .with_extension("txt".into())
            .with_path("test.txt".into())
            .with_message_id(Uuid::new_v4())
            .build();
        assert_eq!(sut, Err(AttachmentBuilderError::MissingNonce));
    }

    #[inline]
    #[must_use]
    fn get_sut() -> AttachmentBuilder {
        AttachmentBuilder::new()
            .with_extension("txt".into())
            .with_path("test.txt".into())
            .with_message_id(Uuid::new_v4())
            .with_nonce([0; 12])
    }
}
