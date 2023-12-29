use bitcoincore_rpc::bitcoin::consensus::serialize;
use bitcoincore_rpc::bitcoin::Transaction;

#[derive(Clone)]
pub enum ClientEvent {
    Transaction(Transaction),

    GetHeight,
}
impl ClientEvent {
    pub fn to_bytes(&self) -> Vec<u8> {
        // TODO: add suffix to distinguish different event
        match self {
            ClientEvent::Transaction(tx) => {
                let ret = serialize(tx);
                ret
            }
            ClientEvent::GetHeight => {
                let ret = "get_height".as_bytes().to_vec();
                ret
            }
        }
    }
}
