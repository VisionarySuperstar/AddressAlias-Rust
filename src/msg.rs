use crate::state::Alias;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// === STRUCTS ===

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ShowResponse {
    pub alias: Option<Alias>,
}

// === ENUMS ===

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    Create { alias: String },
    Destroy { alias: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Show { alias: String },
}
