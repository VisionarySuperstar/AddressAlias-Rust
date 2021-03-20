use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{HumanAddr, Uint128};

// === STRUCTS ===
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
  pub max_alias_size: i32
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Alias {
  alias: String
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ShowResponse {
  pub alias: Alias,
}

// === ENUMS ===

// Response from handle functions
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
  Create {
    status: ResponseStatus,
    message: String,
    alias: Alias
  },
  Destroy {
    status: ResponseStatus,
    message: String,
    alias: Alias
  },
  Update {
    status: ResponseStatus,
    message: String,
    alias: Alias
  },
  Status {
    status: ResponseStatus,
    message: String
  }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
  Create { alias: String },
  Destroy { alias: String },
  Update { alias: String }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
  Index {},
  Show { alias: String },
}

// success or failure response
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub enum ResponseStatus {
    Success,
    Failure,
}
