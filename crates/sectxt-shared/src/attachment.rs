use sectxt_core::attachment::Attachment;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttachmentReadDto {
    pub extension: String,
    pub id: Uuid,
    pub message_id: Uuid,
    pub nonce: Vec<u8>,
    pub path: String,
}

impl From<Attachment> for AttachmentReadDto {
    fn from(value: Attachment) -> Self {
        Self {
            extension: value.extension().to_string(),
            id: value.id(),
            message_id: value.message_id(),
            nonce: value.nonce().to_vec(),
            path: value.path().to_string(),
        }
    }
}

#[cfg(test)]
mod attachment_read_dto_tests {
    use super::*;

    #[test]
    fn test_from_attachment() {
        let id = Uuid::new_v4();
        let message_id = Uuid::new_v4();
        let attachment = Attachment::builder()
            .with_extension("txt".into())
            .with_id(id)
            .with_message_id(message_id)
            .with_nonce([0; 12])
            .with_path("test.txt".into())
            .build()
            .unwrap();

        let sut = AttachmentReadDto::from(attachment);
        assert_eq!(sut.extension, "txt");
        assert_eq!(sut.id, id);
        assert_eq!(sut.message_id, message_id);
        assert_eq!(sut.nonce, [0; 12]);
        assert_eq!(sut.path, "test.txt");
    }
}
