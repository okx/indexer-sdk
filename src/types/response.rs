#[derive(Clone, Debug)]
pub enum DataEnum {
    NewTx,
    TxDropped,
}

impl DataEnum {
    pub fn to_u8(&self) -> u8 {
        match self {
            DataEnum::NewTx => 0,
            DataEnum::TxDropped => 1,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TxResult {}


#[derive(Clone, Debug)]
pub struct GetDataResponse {
    pub data_type: DataEnum,
    pub data: Vec<u8>,
}
