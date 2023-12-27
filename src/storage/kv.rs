use crate::error::IndexerResult;
use crate::event::{AddressType, BalanceType, TokenType, TxIdType};
use crate::storage::db::DB;
use crate::storage::prefix::{DeltaStatus, KeyPrefix};
use crate::storage::StorageProcessor;
use crate::types::delta::TransactionDelta;
use bitcoincore_rpc::bitcoin::Transaction;
use chrono::Local;
use log::info;
use rusty_leveldb::WriteBatch;
use serde::{Deserialize, Serialize};

const MAX_DELAY: i64 = 60 * 60 * 24 * 5; // five days
pub struct KVStorageProcessor<T: DB + Send + Sync> {
    db: T,
}

#[async_trait::async_trait]
impl<T: DB + Send + Sync> StorageProcessor for KVStorageProcessor<T> {
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
        self.wrap_transaction_delta(&mut batch, DeltaStatus::Active, next_state, transaction);
        self.wrap_update_state(&mut batch, next_state);
        // build user utxo
        self.wrap_address_utxo(&mut batch, transaction, true)?;

        self.db.write_batch(batch, true)?;
        Ok(())
    }

    async fn remove_transaction_delta(&mut self, tx_id: &TxIdType) -> IndexerResult<()> {
        let delta = self.get_transaction_delta_by_tx_id(tx_id)?;
        if delta.is_none() {
            info!("tx_id,delta:{:?} not found", tx_id);
            return Ok(());
        }
        let (delta, index) = delta.unwrap();
        if delta.status == DeltaStatus::Inactive.to_u8() {
            info!("tx_id,delta:{:?} is inactive,already consumed", tx_id);
            return Ok(());
        }

        let mut batch = WriteBatch::new();
        self.wrap_transaction_delta(&mut batch, DeltaStatus::Inactive, index, &delta.data);
        self.wrap_address_utxo(&mut batch, &delta.data, false)?;

        self.db.write_batch(batch, true)?;
        Ok(())
    }

    async fn seen_and_store_txs(&mut self, tx: Transaction) -> IndexerResult<bool> {
        let tx_id: TxIdType = tx.txid().into();
        let ret = self.seen_tx(tx_id.clone()).await?;
        if ret {
            return Ok(true);
        }
        let key = KeyPrefix::build_seen_tx_key(&tx_id);
        let dt = Local::now();
        let ts = dt.timestamp();
        info!("tx_id:{:?} is not seen,store it", tx_id);
        self.db.set(key.as_slice(), ts.to_le_bytes().as_slice())?;
        return Ok(false);
    }

    async fn seen_tx(&mut self, tx_id: TxIdType) -> IndexerResult<bool> {
        let key = KeyPrefix::build_seen_tx_key(&tx_id);
        let ret = self.db.get(key.as_slice())?;
        Ok(ret.is_some())
    }

    async fn get_all_un_consumed_txs(&mut self) -> IndexerResult<Vec<TxIdType>> {
        let now = Local::now().timestamp();
        let iter = self.db.iter_all(
            KeyPrefix::SeenTx.get_prefix(),
            |k| k,
            |v| {
                let ts = i64::from_le_bytes(v.as_slice().try_into().unwrap());
                if now - ts > MAX_DELAY {
                    return None;
                }
                Some(ts)
            },
        )?;
        let txs: Vec<TxIdType> = iter
            .into_iter()
            .map(|(k, _)| KeyPrefix::get_tx_id_from_seen_key(k.as_slice()))
            .collect();
        Ok(txs)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TransactionDeltaWrapper {
    pub data: TransactionDelta,
    pub status: u8,
}
impl<T: DB + Send + Sync> KVStorageProcessor<T> {
    pub fn new(db: T) -> IndexerResult<Self> {
        Ok(Self { db })
    }

    fn get_transaction_delta_by_tx_id(
        &mut self,
        tx_id: &TxIdType,
    ) -> IndexerResult<Option<(TransactionDeltaWrapper, u32)>> {
        let key = KeyPrefix::build_transaction_index_map_key(tx_id);
        let index = self.db.get(key.as_slice())?;
        if index.is_none() {
            info!("tx_id:{:?} not found", tx_id);
            return Ok(None);
        }
        let index = index.unwrap();
        let index = u32::from_le_bytes(index.as_slice().try_into().unwrap());
        let delta = self
            .get_transaction_delta_by_index(index)?
            .expect("impossible");
        Ok(Some((delta, index)))
    }
    fn get_transaction_delta_by_index(
        &mut self,
        index: u32,
    ) -> IndexerResult<Option<TransactionDeltaWrapper>> {
        let key = KeyPrefix::TransactionDelta.build_prefix(&index.to_le_bytes());
        let value = self.db.get(key.as_slice())?;
        if value.is_none() {
            return Ok(None);
        }
        let value = value.unwrap();
        let wrapper: TransactionDeltaWrapper = serde_json::from_slice(value.as_slice()).unwrap();
        Ok(Some(wrapper))
    }
    fn wrap_transaction_delta(
        &self,
        batch: &mut WriteBatch,
        status: DeltaStatus,
        index: u32,
        data: &TransactionDelta,
    ) {
        let wrapper = TransactionDeltaWrapper {
            data: data.clone(),
            status: status.to_u8(),
        };
        let value = serde_json::to_vec(&wrapper).unwrap();
        let key = KeyPrefix::build_transaction_data_key(index);
        batch.put(key.as_slice(), value.as_slice());

        let key = KeyPrefix::build_transaction_index_map_key(&data.tx_id);
        let value = index.to_le_bytes().to_vec();
        batch.put(key.as_slice(), value.as_slice());
    }
    fn wrap_update_state(&self, batch: &mut WriteBatch, index: u32) {
        let binding = KeyPrefix::build_state_key();
        let key = binding.as_slice();
        batch.put(key, index.to_le_bytes().as_slice());
    }
    // 是为了get_balance 服务的,
    // 输入地址可以得到这个账户的全部 token 的余额
    // address|token -> balance
    pub(crate) fn wrap_address_utxo(
        &mut self,
        batch: &mut WriteBatch,
        data: &TransactionDelta,
        add: bool,
    ) -> IndexerResult<()> {
        for (address, delta) in &data.deltas {
            for (token_type, bal) in delta {
                let key = KeyPrefix::build_address_token_key(address, token_type);
                let mut value = self.db.get(key.as_slice())?.unwrap_or(vec![]);
                let mut balance = BalanceType::default();
                if !value.is_empty() {
                    balance = serde_json::from_slice(value.as_slice()).unwrap();
                }
                if add {
                    balance.0 = balance.0.clone() + bal.0.clone();
                } else {
                    balance.0 = balance.0.clone() - bal.0.clone();
                    // todo: if balance=0 ,remove key
                }
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
