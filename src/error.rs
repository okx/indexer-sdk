pub use thiserror::Error;

pub type IndexerResult<T> = Result<T, IndexerError>;

#[derive(Debug, Error)]
pub enum IndexerError {
    #[error("bitcoin client error:{0}")]
    BitCoinClientError(#[from] bitcoincore_rpc::Error),

    #[error("hex error:{0}")]
    HexError(#[from] hex::FromHexError),

    #[error("bitcoin encode error:{0}")]
    BitCoinEncodeError(#[from] bitcoincore_rpc::bitcoin::consensus::encode::Error),
}