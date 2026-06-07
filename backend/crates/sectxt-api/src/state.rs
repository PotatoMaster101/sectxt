use sectxt_core::message::repo::MessageRepo;

#[derive(Clone, Debug)]
pub struct AppEnvironment {
    pub database_url: String,
}

impl AppEnvironment {
    #[inline]
    pub fn from_env() -> Result<Self, std::env::VarError> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")?,
        })
    }
}

pub struct AppState {
    pub message_repo: Box<dyn MessageRepo>,
}

impl AppState {
    #[inline]
    #[must_use]
    pub const fn new(message_repo: Box<dyn MessageRepo>) -> Self {
        Self { message_repo }
    }
}
