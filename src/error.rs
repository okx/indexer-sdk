pub use thiserror::Error;

pub type IndexerResult<T> = Result<T, IndexerError>;

#[derive(Debug, Error)]
pub enum IndexerError {
    #[error("bitcoin client error:{0}")]
    BitCoinClientError(#[from] bitcoincore_rpc::Error),
}