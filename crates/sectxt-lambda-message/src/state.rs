use sectxt_core::repo::MessageRepo;

#[derive(Debug, thiserror::Error)]
pub enum AppStateError {
    #[error("{0}")]
    EnvVar(#[from] std::env::VarError),
}

#[derive(Clone, Debug)]
pub struct AppEnvironment {
    database_url: String,
}

impl AppEnvironment {
    pub fn from_env() -> Result<Self, AppStateError> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")?,
        })
    }

    #[inline]
    #[must_use]
    pub fn database_url(&self) -> &str {
        &self.database_url
    }
}

pub struct AppState {
    repo: Box<dyn MessageRepo>,
}

impl AppState {
    #[inline]
    #[must_use]
    pub fn new(repo: Box<dyn MessageRepo>) -> Self {
        Self { repo }
    }

    #[inline]
    #[must_use]
    pub fn repo(&self) -> &dyn MessageRepo {
        self.repo.as_ref()
    }
}
