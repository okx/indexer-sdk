use crate::error::IndexerResult;
use crate::event::{AddressType, BalanceType, TokenType, TxIdType};
use crate::storage::db::DB;
use crate::storage::prefix::SEEN_DATA_STATUS_INDEX;
use crate::storage::prefix::{DeltaStatus, KeyPrefix, SeenStatus};
use crate::storage::{SeenStatusResponse, StorageProcessor};
use crate::types::delta::TransactionDelta;
use bitcoincore_rpc::bitcoin::Transaction;
use chrono::Local;
use log::{error, info};
use rusty_leveldb::WriteBatch;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const MAX_DELAY: i64 = 60 * 60 * 24 * 5; // five days

#[derive(Clone)]
pub struct KVStorageProcessor<T: DB + Send + Sync + Clone> {
    db: T,
}
impl<T: DB + Send + Sync + Clone + Default> Default for KVStorageProcessor<T> {
    fn default() -> Self {
        Self { db: T::default() }
    }
}

unsafe impl<T: DB + Send + Sync + Clone> Send for KVStorageProcessor<T> {}
unsafe impl<T: DB + Send + Sync + Clone> Sync for KVStorageProcessor<T> {}

#[async_trait::async_trait]
impl<T: DB + Send + Sync + Clone> StorageProcessor for KVStorageProcessor<T> {
    async fn get_balance(
        &mut self,
        address: &AddressType,
        token_type: &TokenType,
    ) -> IndexerResult<BalanceType> {
        let key = KeyPrefix::build_address_token_key(address, token_type);
        let value = self
            .db
            .get(key.as_slice())?
            .map_or(BalanceType::default(), |v| {
                let bal: BalanceType = serde_json::from_slice(v.as_slice()).unwrap();
                bal
            });
        Ok(value)
    }

    async fn add_transaction_delta(&mut self, transaction: &TransactionDelta) -> IndexerResult<()> {
        info!(
            "tx_id:{:?} is finished,add_transaction_delta:{:?}",
            &transaction.tx_id, transaction
        );
        if transaction.deltas.is_empty() {
            let mut batch = WriteBatch::new();
            self.wrap_seen_txs(&mut batch, &transaction.tx_id, SeenStatus::Executed)?;
            self.db.write_batch(batch, true)?;
            return Ok(());
        }
        let mut batch = WriteBatch::new();
        let next_state = self.acquire_next_state()?;
        info!(
            "tx_id:{:?},add transaction delta, next state: {:?},delta:{:?}",
            transaction.tx_id, next_state, transaction
        );
        self.wrap_transaction_delta(&mut batch, DeltaStatus::Executed, next_state, transaction);
        self.wrap_update_state(&mut batch, next_state);
        // build user utxo
        self.wrap_address_utxo(&mut batch, transaction, true)?;
        self.wrap_seen_txs(&mut batch, &transaction.tx_id, SeenStatus::Executed)?;

        self.db.write_batch(batch, true)?;
        Ok(())
    }

    async fn remove_transaction_delta(
        &mut self,
        tx_id: &TxIdType,
        status: DeltaStatus,
    ) -> IndexerResult<()> {
        let delta = self.get_transaction_delta_by_tx_id(tx_id)?;
        if delta.is_none() {
            info!("tx_id,delta:{:?} not found", tx_id);
            return Ok(());
        }
        let (delta, index) = delta.unwrap();
        if delta.status != DeltaStatus::Default.to_u8() {
            info!("tx_id,delta:{:?} is inactive,already consumed", tx_id);
            return Ok(());
        }

        let mut batch = WriteBatch::new();
        self.wrap_transaction_delta(&mut batch, status, index, &delta.data);
        self.wrap_address_utxo(&mut batch, &delta.data, false)?;
        self.rm_seen_tx(&mut batch, tx_id);

        self.db.write_batch(batch, true)?;
        Ok(())
    }

    async fn seen_and_store_txs(&mut self, tx: &Transaction) -> IndexerResult<SeenStatusResponse> {
        let tx_id: TxIdType = tx.txid().into();
        let seen_status = self.seen_tx(tx_id.clone()).await?;
        if seen_status.is_seen() {
            return Ok(seen_status);
        }
        let key = KeyPrefix::build_seen_tx_key(&tx_id);
        let dt = Local::now();
        let ts = dt.timestamp();
        let mut data = ts.to_le_bytes().to_vec();
        data.extend_from_slice(SeenStatus::UnExecuted.to_u8().to_le_bytes().as_slice());
        info!("tx_id:{:?} is not seen,store it", tx_id);
        self.db.set(key.as_slice(), data.as_slice())?;
        let vv = self.db.get(key.as_slice())?;
        return Ok(SeenStatusResponse {
            seen: false,
            status: SeenStatus::UnExecuted,
        });
    }

    async fn seen_tx(&mut self, tx_id: TxIdType) -> IndexerResult<SeenStatusResponse> {
        let key = KeyPrefix::build_seen_tx_key(&tx_id);
        let ret = self.db.get(key.as_slice())?;
        if ret.is_none() {
            return Ok(SeenStatusResponse {
                seen: false,
                status: SeenStatus::UnExecuted,
            });
        }
        let ret = ret.unwrap();
        let last = ret[ret.len() - 1];
        Ok(SeenStatusResponse {
            seen: true,
            status: SeenStatus::from_u8(last),
        })
    }

    async fn get_all_un_consumed_txs(&mut self) -> IndexerResult<HashMap<TxIdType, i64>> {
        let now = Local::now().timestamp();
        let iter = self.db.iter_all(
            KeyPrefix::SeenTx.get_prefix(),
            |k| k,
            |v| {
                let last = v[v.len() - 1];
                if last == SeenStatus::Executed.to_u8() {
                    return None;
                }
                let ts = i64::from_le_bytes(v[..8].try_into().unwrap());
                if now - ts > MAX_DELAY {
                    return None;
                }
                Some(ts)
            },
        )?;
        let txs: HashMap<TxIdType, i64> = iter
            .into_iter()
            .map(|(k, ts)| (KeyPrefix::get_tx_id_from_seen_key(k.as_slice()), ts))
            .collect();
        Ok(txs)
    }

    async fn is_tx_executed(&mut self, tx_id: &TxIdType) -> IndexerResult<bool> {
        let resp = self.seen_tx(tx_id.clone()).await?;
        Ok(resp.is_executed())
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TransactionDeltaWrapper {
    pub data: TransactionDelta,
    pub status: u8,
}
impl<T: DB + Send + Sync + Clone> KVStorageProcessor<T> {
    pub fn new(db: T) -> Self {
        Self { db }
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
    fn rm_seen_tx(&self, batch: &mut WriteBatch, tx_id: &TxIdType) {
        let key = KeyPrefix::build_seen_tx_key(tx_id);
        batch.delete(key.as_slice());
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
    pub(crate) fn wrap_seen_txs(
        &mut self,
        write_batch: &mut WriteBatch,
        tx_id: &TxIdType,
        status: SeenStatus,
    ) -> IndexerResult<()> {
        let key = KeyPrefix::build_seen_tx_key(tx_id);
        let ret = self.db.get(key.as_slice())?;
        if ret.is_none() {
            error!(
                "wrap_seen_txs tx_id:{:?} not found,this should not happen",
                tx_id
            );
            return Ok(());
        }
        let mut data = ret.unwrap();
        data[SEEN_DATA_STATUS_INDEX] = status.to_u8();
        write_batch.put(key.as_slice(), data.as_slice());
        Ok(())
    }
    pub(crate) fn rm_seen_record(&mut self, batch: &mut WriteBatch, tx_id: &TxIdType) {
        let key = KeyPrefix::build_seen_tx_key(tx_id);
        batch.delete(key.as_slice());
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
            .map_or(0, |v| u32::from_le_bytes(v.as_slice().try_into().unwrap()));
        Ok(ret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::db::memory::MemoryDB;
    use std::collections::HashMap;

    #[tokio::test]
    pub async fn test_get_balance() {
        let db = MemoryDB::default();
        let mut storage = KVStorageProcessor::new(db);

        let tx_id = [0u8; 32];
        let mut delta = HashMap::default();

        let address = AddressType::from_bytes(&[0u8; 20]);
        {
            let mut deltas = vec![];
            deltas.push((TokenType::from_bytes(&[0u8; 20]), BalanceType::from(1)));
            deltas.push((TokenType::from_bytes(&[1u8; 20]), BalanceType::from(2)));
            delta.insert(address.clone(), deltas);
        }
        let delta = TransactionDelta {
            tx_id: TxIdType::from_bytes(&tx_id),
            deltas: delta,
        };
        storage.add_transaction_delta(&delta).await.unwrap();

        let bal = storage
            .get_balance(&address, &TokenType::from_bytes(&[0u8; 20]))
            .await
            .unwrap();
        println!("{:?}", bal);
        assert_eq!(bal, BalanceType::from(1i32))
    }
    #[tokio::test]
    pub async fn test_revert() {
        let db = MemoryDB::default();
        let mut storage = KVStorageProcessor::new(db);

        let tx_id = [0u8; 32];
        let mut delta = HashMap::default();

        let address = AddressType::from_bytes(&[0u8; 20]);
        {
            let mut deltas = vec![];
            deltas.push((TokenType::from_bytes(&[0u8; 20]), BalanceType::from(1)));
            deltas.push((TokenType::from_bytes(&[1u8; 20]), BalanceType::from(2)));
            delta.insert(address.clone(), deltas);
        }
        let delta = TransactionDelta {
            tx_id: TxIdType::from_bytes(&tx_id),
            deltas: delta,
        };
        storage.add_transaction_delta(&delta).await.unwrap();

        let bal = storage
            .get_balance(&address, &TokenType::from_bytes(&[0u8; 20]))
            .await
            .unwrap();
        println!("{:?}", bal);
        assert_eq!(bal, BalanceType::from(1i32));

        let tx_id = [1u8; 32];
        let mut delta = HashMap::default();
        {
            let mut deltas = vec![];
            deltas.push((TokenType::from_bytes(&[0u8; 20]), BalanceType::from(10)));
            deltas.push((TokenType::from_bytes(&[1u8; 20]), BalanceType::from(20)));
            delta.insert(address.clone(), deltas);
        }
        let delta = TransactionDelta {
            tx_id: TxIdType::from_bytes(&tx_id),
            deltas: delta,
        };
        storage.add_transaction_delta(&delta).await.unwrap();

        let bal = storage
            .get_balance(&address, &TokenType::from_bytes(&[0u8; 20]))
            .await
            .unwrap();
        println!("{:?}", bal);
        assert_eq!(bal, BalanceType::from(11i32));

        storage
            .remove_transaction_delta(&TxIdType::from_bytes(&tx_id), DeltaStatus::Confirmed)
            .await
            .unwrap();
        let bal = storage
            .get_balance(&address, &TokenType::from_bytes(&[0u8; 20]))
            .await
            .unwrap();
        println!("{:?}", bal);
        assert_eq!(bal, BalanceType::from(1i32));
    }
}
