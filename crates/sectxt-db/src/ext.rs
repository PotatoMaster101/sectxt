use crate::attachment::AttachmentModelError;
use crate::message::MessageModelError;
use sectxt_core::repo::MessageRepoError;

pub trait ResultExt<T> {
    fn map_core_err(self) -> Result<T, MessageRepoError>;
}

impl<T> ResultExt<T> for Result<T, sqlx::Error> {
    #[inline]
    fn map_core_err(self) -> Result<T, MessageRepoError> {
        self.map_err(|e| MessageRepoError::Database(e.to_string()))
    }
}

impl<T> ResultExt<T> for Result<T, MessageModelError> {
    #[inline]
    fn map_core_err(self) -> Result<T, MessageRepoError> {
        self.map_err(|e| MessageRepoError::Model(e.to_string()))
    }
}

impl<T> ResultExt<T> for Result<T, AttachmentModelError> {
    #[inline]
    fn map_core_err(self) -> Result<T, MessageRepoError> {
        self.map_err(|e| MessageRepoError::Model(e.to_string()))
    }
}
