use first_read_last_write_cache::cache::{self, FirstReads};

use crate::{
    internal_cache::{StorageInternalCache, ValueReader},
    storage::{StorageKey, StorageValue},
    Storage,
};

// Implementation of `ValueReader` trait for the zk-context. FirstReads is backed by a HashMap internally,
// this is a good default choice. Once we start integrating with a proving system
// we might want to explore other alternatives. For example, in Risc0 we could implement `ValueReader`
// in terms of `env::read()` and fetch values lazily from the host.
impl ValueReader for FirstReads {
    fn read_value(&self, key: StorageKey) -> Option<StorageValue> {
        let key = key.as_cache_key();
        match self.get(&key) {
            cache::ValueExists::Yes(read) => read.map(StorageValue::new_from_cache_value),
            // It is ok to panic here, `ZkStorage` must be able to access all the keys it needs.
            cache::ValueExists::No => panic!("Error: Key {key:?} is inaccessible"),
        }
    }
}

#[derive(Clone)]
pub struct ZkStorage {
    batch_cache: StorageInternalCache,
    value_reader: FirstReads,
    tx_cache: StorageInternalCache,
}

impl ZkStorage {
    pub fn new(value_reader: FirstReads) -> Self {
        Self {
            value_reader,
            tx_cache: StorageInternalCache::default(),
            batch_cache: StorageInternalCache::default(),
        }
    }
}

impl Storage for ZkStorage {
    fn get(&self, key: StorageKey) -> Option<StorageValue> {
        self.tx_cache.get_or_fetch(key, &self.value_reader)
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
        // TODO: calculate JMT root in-circuit and commit it to the zk-proof log
    }
}
