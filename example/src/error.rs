use thiserror::Error;

#[derive(Error, Debug)]
pub enum TestError {
    #[error("sqlx Error")]
    SqlxError(#[from] sqlx::Error),
}
