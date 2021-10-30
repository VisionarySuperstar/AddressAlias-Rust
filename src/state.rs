use cosmwasm_std::{HumanAddr, ReadonlyStorage, StdResult, Storage};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use schemars::JsonSchema;
use secret_toolkit::serialization::{Bincode2, Serde};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

// === CONSTANTS ===
pub const ADDRESSES_ALIASES_PREFIX: &[u8] = b"addresses_aliases";
pub const ALIASES_PREFIX: &[u8] = b"aliases";

// === STRUCTS ===
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Alias {
    pub human_address: HumanAddr,
    pub avatar_url: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Config {
    pub buttcoin: SecretContract,
    pub profit_distributor: SecretContract,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, JsonSchema)]
pub struct SecretContract {
    pub address: HumanAddr,
    pub contract_hash: String,
}

// === Aliases Storage ===
pub struct AliasesReadonlyStorage<'a, S: Storage> {
    storage: ReadonlyPrefixedStorage<'a, S>,
}
impl<'a, S: Storage> AliasesReadonlyStorage<'a, S> {
    pub fn from_storage(storage: &'a S) -> Self {
        Self {
            storage: ReadonlyPrefixedStorage::new(ALIASES_PREFIX, storage),
        }
    }

    pub fn get_alias(&self, key: &[u8]) -> Option<Alias> {
        self.as_readonly().get(key)
    }

    // private

    fn as_readonly(&self) -> ReadonlyAliasesStorageImpl<ReadonlyPrefixedStorage<S>> {
        ReadonlyAliasesStorageImpl(&self.storage)
    }
}

pub struct AliasesStorage<'a, S: Storage> {
    storage: PrefixedStorage<'a, S>,
}
impl<'a, S: Storage> AliasesStorage<'a, S> {
    pub fn from_storage(storage: &'a mut S) -> Self {
        Self {
            storage: PrefixedStorage::new(ALIASES_PREFIX, storage),
        }
    }

    pub fn get_alias(&mut self, key: &[u8]) -> Option<Alias> {
        self.as_readonly().get(key)
    }

    pub fn remove_alias(&mut self, key: &[u8]) {
        remove(&mut self.storage, &key);
    }

    pub fn set_alias(&mut self, key: &[u8], value: Alias) {
        save(&mut self.storage, &key, &value).ok();
    }

    // private

    fn as_readonly(&self) -> ReadonlyAliasesStorageImpl<PrefixedStorage<S>> {
        ReadonlyAliasesStorageImpl(&self.storage)
    }
}

struct ReadonlyAliasesStorageImpl<'a, S: ReadonlyStorage>(&'a S);
impl<'a, S: ReadonlyStorage> ReadonlyAliasesStorageImpl<'a, S> {
    pub fn get(&self, key: &[u8]) -> Option<Alias> {
        let alias: Option<Alias> = may_load(self.0, &key).ok().unwrap();
        alias
    }
}

// === AddressesAliases Storage ===

pub struct AddressesAliasesReadonlyStorage<'a, S: Storage> {
    storage: ReadonlyPrefixedStorage<'a, S>,
}
impl<'a, S: Storage> AddressesAliasesReadonlyStorage<'a, S> {
    pub fn from_storage(storage: &'a S) -> Self {
        Self {
            storage: ReadonlyPrefixedStorage::new(ADDRESSES_ALIASES_PREFIX, storage),
        }
    }

    pub fn get_alias(&self, key: &String) -> Option<Vec<u8>> {
        self.as_readonly().get(key)
    }

    // private

    fn as_readonly(&self) -> ReadonlyAddressesAliasesStorageImpl<ReadonlyPrefixedStorage<S>> {
        ReadonlyAddressesAliasesStorageImpl(&self.storage)
    }
}

pub struct AddressesAliasesStorage<'a, S: Storage> {
    storage: PrefixedStorage<'a, S>,
}
impl<'a, S: Storage> AddressesAliasesStorage<'a, S> {
    pub fn from_storage(storage: &'a mut S) -> Self {
        Self {
            storage: PrefixedStorage::new(ADDRESSES_ALIASES_PREFIX, storage),
        }
    }

    pub fn get_alias(&mut self, key: &String) -> Option<Vec<u8>> {
        self.as_readonly().get(key)
    }

    pub fn remove_alias(&mut self, key: &[u8]) {
        remove(&mut self.storage, &key);
    }

    pub fn set_alias(&mut self, key: &[u8], value: &String) {
        save(&mut self.storage, key, value).ok();
    }

    // private

    fn as_readonly(&self) -> ReadonlyAddressesAliasesStorageImpl<PrefixedStorage<S>> {
        ReadonlyAddressesAliasesStorageImpl(&self.storage)
    }
}

struct ReadonlyAddressesAliasesStorageImpl<'a, S: ReadonlyStorage>(&'a S);
impl<'a, S: ReadonlyStorage> ReadonlyAddressesAliasesStorageImpl<'a, S> {
    pub fn get(&self, key: &String) -> Option<Vec<u8>> {
        let alias: Option<Vec<u8>> = may_load(self.0, &key.as_bytes()).ok().unwrap();
        alias
    }
}

// === FUNCTIONS ===
fn may_load<T: DeserializeOwned, S: ReadonlyStorage>(
    storage: &S,
    key: &[u8],
) -> StdResult<Option<T>> {
    match storage.get(key) {
        Some(value) => Bincode2::deserialize(&value).map(Some),
        None => Ok(None),
    }
}

fn remove<S: Storage>(storage: &mut S, key: &[u8]) {
    storage.remove(key);
}

fn save<T: Serialize, S: Storage>(storage: &mut S, key: &[u8], value: &T) -> StdResult<()> {
    storage.set(key, &Bincode2::serialize(value)?);
    Ok(())
}
