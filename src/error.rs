use crate::client::event::ClientEvent;
use crate::dispatcher::event::DispatchEvent;
use rusty_leveldb::Status;
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

    #[error("level db error,msg:{0}")]
    RustLevelDBError(String),

    #[error("error,msg:{0}")]
    MsgError(String),

    #[error("channel error,msg:{0}")]
    ChannelError(#[from] async_channel::SendError<DispatchEvent>),

    #[error("channel error,msg:{0}")]
    ChannelClientError(#[from] async_channel::SendError<ClientEvent>),
}

impl From<Status> for IndexerError {
    fn from(value: Status) -> Self {
        Self::RustLevelDBError(value.err)
    }
}
