use std::{fs, path::Path, sync::Arc};

use crate::{
    internal_cache::{StorageInternalCache, ValueReader},
    storage::{StorageKey, StorageValue},
    Storage,
};
use first_read_last_write_cache::cache::FirstReads;
use jmt::{
    storage::{NodeBatch, TreeWriter},
    KeyHash, Version,
};
use sovereign_db::state_db::StateDB;
use sovereign_sdk::core::crypto;

#[derive(Clone)]
struct StateDBAndVersion {
    db: StateDB,
    version: Version,
}

impl ValueReader for StateDBAndVersion {
    fn read_value(&self, key: StorageKey) -> Option<StorageValue> {
        match self.db.get_value_option_by_key(self.version, key.as_ref()) {
            Ok(value) => value.map(StorageValue::new_from_bytes),
            // It is ok to panic here, we assume the db is available and consistent.
            Err(e) => panic!("Unable to read value from db: {e}"),
        }
    }
}

#[derive(Clone)]
pub struct JmtStorage {
    batch_cache: StorageInternalCache,
    tx_cache: StorageInternalCache,
    db_and_version: StateDBAndVersion,
}

impl JmtStorage {
    #[cfg(any(test, feature = "temp"))]
    pub fn temporary() -> Self {
        let db = StateDB::temporary();
        Self::with_db(db).unwrap()
    }

    pub fn with_path(path: impl AsRef<Path>) -> Result<Self, anyhow::Error> {
        let db = StateDB::with_path(&path)?;
        Self::with_db(db)
    }

    fn with_db(db: StateDB) -> Result<Self, anyhow::Error> {
        let version = db.last_version()?.map(|v| v + 1).unwrap_or_default();
        Ok(Self {
            batch_cache: StorageInternalCache::default(),
            tx_cache: StorageInternalCache::default(),
            db_and_version: StateDBAndVersion { db, version },
        })
    }

    /// Gets the first reads from the JmtStorage.
    pub fn get_first_reads(&self) -> FirstReads {
        self.tx_cache.borrow().get_first_reads()
    }
}

impl Storage for JmtStorage {
    fn get(&self, key: StorageKey) -> Option<StorageValue> {
        self.tx_cache.get_or_fetch(key, &self.db_and_version)
    }

    fn set(&mut self, key: StorageKey, value: StorageValue) {
        self.tx_cache.set(key, value)
    }

    fn delete(&mut self, key: StorageKey) {
        self.tx_cache.delete(key)
    }

    fn merge(&mut self) {
        self.batch_cache
            .merge(&mut self.tx_cache)
            .unwrap_or_else(|e| panic!("Cache merge error: {e}"));
    }

    fn finalize(&mut self) {
        let mut batch = NodeBatch::default();
        let cache = &mut self.batch_cache.borrow_mut();

        let mut data = Vec::with_capacity(cache.len());

        let mut save_in_db = false;
        for (cache_key, cache_value) in cache.get_all_writes_and_clear_cache() {
            save_in_db = true;
            let key = &cache_key.key;
            // TODO: Don't hardcode the hashing algorithm
            // https://github.com/Sovereign-Labs/sovereign/issues/113
            let key_hash = KeyHash(crypto::hash::sha2(key.as_ref()).0);

            self.db_and_version
                .db
                .put_preimage(key_hash, key)
                .unwrap_or_else(|e| panic!("Database error: {e}"));

            let value = cache_value.map(|v| Arc::try_unwrap(v.value).unwrap());
            data.push(((self.db_and_version.version, key_hash), value));
        }

        // TODO: Question, should we bump version even if nothing was saved in the db?
        if save_in_db {
            self.db_and_version.version += 1;
            batch.extend(vec![], data);
            self.db_and_version.db.write_node_batch(&batch).unwrap();
        }
    }
}

pub fn delete_storage(path: impl AsRef<Path>) {
    fs::remove_dir_all(&path)
        .or_else(|_| fs::remove_file(&path))
        .unwrap_or(());
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_jmt_storage() {
        let path = schemadb::temppath::TempPath::new();

        {
            let key_1 = StorageKey::from("key_1");
            let value_1 = StorageValue::from("value_1");

            let mut storage = JmtStorage::with_path(&path).unwrap();
            storage.set(key_1, value_1);
            storage.merge();
            storage.finalize();

            assert_eq!(storage.db_and_version.version, 1);
        }

        {
            let key_1 = StorageKey::from("key_1");
            let value_1 = StorageValue::from("value_1");

            let mut storage = JmtStorage::with_path(&path).unwrap();

            assert_eq!(value_1, storage.get(key_1).unwrap());

            assert_eq!(storage.db_and_version.version, 1);

            let key = StorageKey::from("key_2");
            let value = StorageValue::from("value_2");
            assert_eq!(storage.db_and_version.version, 1);
            storage.set(key, value);
            storage.merge();
            storage.finalize();
            assert_eq!(storage.db_and_version.version, 2);
        }

        {
            let mut storage = JmtStorage::with_path(&path).unwrap();
            assert_eq!(storage.db_and_version.version, 2);

            let key = StorageKey::from("key_2");
            let value = StorageValue::from("value_2");

            storage.set(key, value);
            storage.merge();
            storage.finalize();
            assert_eq!(storage.db_and_version.version, 3);
        }
    }
}
