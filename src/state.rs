use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{CanonicalAddr, Storage, StdResult};
use cosmwasm_storage::{PrefixedStorage};
use secret_toolkit::serialization::{Bincode2, Serde};

// === STATICS ===
pub static CONFIG_KEY: &[u8] = b"config";

// === STRUCTS ===
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
  pub max_alias_size: u16,
}

// === FUNCTIONS ===

// Returns StdResult<()> resulting from saving an item to storage
// Arguments:
// storage - a mutable reference to the storage this item should go to
// key - a byte slice representing the key to access the stored item
// value - a reference to the item to store
pub fn save<T: Serialize, S: Storage>(storage: &mut S, key: &[u8], value: &T) -> StdResult<()> {
  storage.set(key, &Bincode2::serialize(value)?);
  Ok(())
}
