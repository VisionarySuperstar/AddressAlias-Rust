use crate::state::Alias;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// === STRUCTS ===
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub max_alias_size: i32,
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ShowResponse {
    pub alias: Option<Alias>,
}

// === ENUMS ===

// Response from handle functions
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    Create { message: String },
    Destroy { message: String, alias: Alias },
    Update { message: String, alias: Alias },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    Create { alias_string: String },
    Destroy { alias_string: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Show { alias_string: String },
}
