use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::HumanAddr;

use crate::{state::File, viewing_key::ViewingKey};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub owner: Option<HumanAddr>, //- don't need this?
    pub prng_seed: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    InitAddress {entropy: String},
    SendFile { to: HumanAddr, contents: String },
    CreateViewingKey {
        entropy: String,
        padding: Option<String>,
    },

}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    GetContents { behalf: HumanAddr, path: String, key: String },//this used?
    GetFiles { address: HumanAddr},
    // YouUpBro{address: String},
    GetWalletInfo { behalf: HumanAddr, key: String},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ViewingPermissions {
    pub key: Option<String>
    //got rid of permit
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
//not sure if we need this
pub struct FileResponse {
    pub files: Vec<File>,
    pub length: u32
} //make one for single file?

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    DefaultAnswer { status:ResponseStatus},
    CreateViewingKey { key: ViewingKey },
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WalletInfoResponse {
    pub init: bool,
    pub all_paths: Vec<String>
}

impl QueryMsg {
    pub fn get_validation_params(&self) -> (Vec<&HumanAddr>, ViewingKey) {
        match self {
            Self::GetContents { behalf, key, .. } => (vec![behalf], ViewingKey(key.clone())),
            Self::GetWalletInfo { behalf, key, .. } => (vec![behalf], ViewingKey(key.clone())),
            _ => panic!("This query type does not require authentication"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    Success,
    Failure,
}


