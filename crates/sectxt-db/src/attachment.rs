use sectxt_core::attachment::{Attachment, AttachmentBuilderError};
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum AttachmentModelError {
    #[error("{0}")]
    Builder(#[from] AttachmentBuilderError),

    #[error("{0} must be length {1}")]
    FieldLength(&'static str, usize),
}

#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct AttachmentModel {
    id: Uuid,
    message_id: Uuid,
    path: String,
    extension: String,
    nonce: Vec<u8>,
}

impl From<Attachment> for AttachmentModel {
    fn from(value: Attachment) -> Self {
        Self {
            id: value.id(),
            message_id: value.message_id(),
            path: value.path().into(),
            extension: value.extension().into(),
            nonce: value.nonce().to_vec(),
        }
    }
}

impl TryFrom<AttachmentModel> for Attachment {
    type Error = AttachmentModelError;
    fn try_from(value: AttachmentModel) -> Result<Self, Self::Error> {
        let nonce: [u8; 12] = value
            .nonce
            .try_into()
            .map_err(|_| AttachmentModelError::FieldLength("nonce", 12))?;

        Self::builder()
            .with_extension(value.extension)
            .with_id(value.id)
            .with_message_id(value.message_id)
            .with_nonce(nonce)
            .with_path(value.path)
            .build()
            .map_err(AttachmentModelError::Builder)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_attachment() {
        let attachment = Attachment::builder()
            .with_extension("txt".into())
            .with_id(Uuid::new_v4())
            .with_message_id(Uuid::new_v4())
            .with_nonce([0u8; 12])
            .with_path("test.txt".into())
            .build()
            .unwrap();

        let sut = AttachmentModel::from(attachment.clone());
        assert_eq!(sut.extension, attachment.extension());
        assert_eq!(sut.id, attachment.id());
        assert_eq!(sut.message_id, attachment.message_id());
        assert_eq!(sut.nonce, attachment.nonce());
        assert_eq!(sut.path, attachment.path());
    }

    #[test]
    fn test_try_from_attachment_model() {
        let sut = AttachmentModel {
            id: Uuid::new_v4(),
            message_id: Uuid::new_v4(),
            path: "test.txt".into(),
            extension: "txt".into(),
            nonce: vec![0u8; 12],
        };

        let result = Attachment::try_from(sut.clone()).unwrap();
        assert_eq!(sut.id, result.id());
        assert_eq!(sut.message_id, result.message_id());
        assert_eq!(sut.path, result.path());
        assert_eq!(sut.extension, result.extension());
        assert_eq!(sut.nonce, result.nonce());
    }

    #[test]
    fn test_try_from_attachment_model_invalid_field_length() {
        let sut = AttachmentModel {
            id: Uuid::new_v4(),
            message_id: Uuid::new_v4(),
            path: "test.txt".into(),
            extension: "txt".into(),
            nonce: vec![0u8; 11],
        };
        assert!(Attachment::try_from(sut).is_err());
    }
}
