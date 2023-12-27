use crate::error::IndexerResult;
use crate::event::{AddressType, BalanceType, TokenType, TxIdType};
use crate::storage::db::DB;
use crate::storage::StorageProcessor;
use crate::types::delta::TransactionDelta;
use log::info;
use rusty_leveldb::WriteBatch;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub enum KeyPrefix {
    State,
    TransactionDelta,    // index -> TransactionWrapper
    TransactionIndexMap, // tx_id -> index
    AddressTokenBalance, // address|token -> balance
}
pub enum DeltaStatus {
    Active,
    Inactive,
}
impl DeltaStatus {
    pub fn to_u8(&self) -> u8 {
        match self {
            DeltaStatus::Active => 0,
            DeltaStatus::Inactive => 1,
        }
    }
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => DeltaStatus::Active,
            1 => DeltaStatus::Inactive,
            _ => panic!("invalid delta status"),
        }
    }
}
impl KeyPrefix {
    pub fn get_prefix(&self) -> &[u8] {
        match self {
            KeyPrefix::State => b"state",
            KeyPrefix::TransactionDelta => b"td",
            KeyPrefix::AddressTokenBalance => b"atb",
            KeyPrefix::TransactionIndexMap => b"tim",
        }
    }
    pub fn build_prefix(&self, key: &[u8]) -> Vec<u8> {
        let mut ret = self.get_prefix().to_vec();
        ret.extend_from_slice(key);
        ret
    }
}

pub struct KVStorageProcessor {
    db: Box<dyn DB<Batch = WriteBatch> + Send + Sync>,
}

#[async_trait::async_trait]
impl StorageProcessor for KVStorageProcessor {
    async fn get_balance(
        &self,
        token_type: &TokenType,
        address: &AddressType,
    ) -> IndexerResult<BalanceType> {
        todo!()
    }

    async fn add_transaction_delta(&mut self, transaction: &TransactionDelta) -> IndexerResult<()> {
        info!(
            "tx_id:{:?} is finished,add_transaction_delta:{:?}",
            &transaction.tx_id, transaction
        );
        if transaction.deltas.is_empty() {
            return Ok(());
        }
        let mut batch = WriteBatch::new();
        let next_state = self.acquire_next_state()?;
        info!(
            "tx_id:{:?},add transaction delta, next state: {:?},delta:{:?}",
            transaction.tx_id, next_state, transaction
        );
        self.wrap_transaction_delta(&mut batch, next_state, transaction);
        self.wrap_update_state(&mut batch, next_state);
        // build user utxo
        self.wrap_address_utxo(&mut batch, transaction)?;

        self.db.write_batch(batch, true)?;
        Ok(())
    }

    // 删除一个交易对应的delta 数据
    // 所以需要先找到 这个 txid 对应的所有增量信息
    async fn remove_transaction_delta(&mut self, tx_id: &TxIdType) -> IndexerResult<()> {
        todo!()
    }

    async fn seen_and_store_txs(&mut self, tx_id: TxIdType) -> IndexerResult<bool> {
        todo!()
    }

    async fn seen_tx(&self, tx_id: TxIdType) -> IndexerResult<bool> {
        todo!()
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TransactionDeltaWrapper {
    pub data: TransactionDelta,
    pub status: u8,
}
impl KVStorageProcessor {
    pub fn new(db: Box<dyn DB<Batch = WriteBatch> + Send + Sync>) -> IndexerResult<Self> {
        Ok(Self { db })
    }

    fn get_transaction_delta_by_tx_id(
        &self,
        tx_id: &TxIdType,
    ) -> IndexerResult<Option<TransactionDeltaWrapper>> {
        todo!()
    }
    fn wrap_transaction_delta(&self, batch: &mut WriteBatch, index: u32, data: &TransactionDelta) {
        let wrapper = TransactionDeltaWrapper {
            data: data.clone(),
            status: DeltaStatus::Active.to_u8(),
        };
        let value = serde_json::to_vec(&wrapper).unwrap();
        let key = KeyPrefix::TransactionDelta.build_prefix(&index.to_le_bytes());
        batch.put(key.as_slice(), value.as_slice());

        let tx_id_decode = data.tx_id.to_bytes();
        let key = KeyPrefix::TransactionIndexMap.build_prefix(&tx_id_decode);
        let value = index.to_le_bytes().to_vec();
        batch.put(key.as_slice(), value.as_slice());
    }
    fn wrap_update_state(&self, batch: &mut WriteBatch, index: u32) {
        let key = KeyPrefix::State.get_prefix();
        batch.put(key, index.to_le_bytes().as_slice());
    }
    // 是为了get_balance 服务的,
    // 输入地址可以得到这个账户的全部 token 的余额
    // address|token -> balance
    pub(crate) fn wrap_address_utxo(
        &mut self,
        batch: &mut WriteBatch,
        data: &TransactionDelta,
    ) -> IndexerResult<()> {
        for (address, delta) in &data.deltas {
            for (token_type, bal) in delta {
                let mut suffix = address.clone();
                suffix.extend_from_slice(token_type);
                let key = KeyPrefix::AddressTokenBalance.build_prefix(suffix.as_slice());
                let mut value = self.db.get(key.as_slice())?.unwrap_or(vec![]);
                let mut balance = BalanceType::default();
                if !value.is_empty() {
                    balance = serde_json::from_slice(value.as_slice()).unwrap();
                }
                balance.0 = balance.0.clone() + bal.0.clone();
                value = serde_json::to_vec(&balance).unwrap();
                batch.put(key.as_slice(), value.as_slice());
            }
        }
        Ok(())
    }
    pub fn update_state(&mut self, id: u32) -> IndexerResult<()> {
        let key = KeyPrefix::State.get_prefix();
        self.db.set(key, id.to_le_bytes().as_slice())?;
        Ok(())
    }
    pub fn acquire_next_state(&mut self) -> IndexerResult<u32> {
        let ret = self.acquire_latest_state()?;
        Ok(ret + 1)
    }
    pub fn acquire_latest_state(&mut self) -> IndexerResult<u32> {
        let key = KeyPrefix::State.get_prefix();
        let ret = self
            .db
            .get(key)?
            .map_or(1, |v| u32::from_le_bytes(v.as_slice().try_into().unwrap()));
        Ok(ret)
    }
}
