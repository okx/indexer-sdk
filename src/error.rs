pub use thiserror::Error;

pub type IndexerResult<T> = Result<T, IndexerError>;
#[derive(Debug, Error)]
pub enum IndexerError {


}