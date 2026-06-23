use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Err(Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("bad magic: {1} at {0}")]
    BadMagic(u64, String),
    #[error("assert fail: {0}")]
    AssertFail(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
