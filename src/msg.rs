use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::HumanAddr;

use crate::{state::Message, viewing_key::ViewingKey};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub owner: Option<HumanAddr>, //- don't need this?
    pub prng_seed: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    InitAddress {entropy: String},
    SendMessage { to: HumanAddr, path: String },
    CreateViewingKey {
        entropy: String,
        padding: Option<String>,
    },

}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {

    GetMessages { behalf: HumanAddr, key: String },//No need to enter path because path pertains to one file only? 

    //You up bro and GetWalletInfo may not be needed
    // YouUpBro{address: String},
    //GetWalletInfo { behalf: HumanAddr, key: String},
}

impl QueryMsg {
    pub fn get_validation_params(&self) -> (Vec<&HumanAddr>, ViewingKey) {
        match self {
            Self::GetMessages { behalf, key, .. } => (vec![behalf], ViewingKey(key.clone())),
            //Self::GetWalletInfo { behalf, key, .. } => (vec![behalf], ViewingKey(key.clone())),
            _ => panic!("This query type does not require authentication"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ViewingPermissions { //we need this?
    pub key: Option<String>
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]

pub struct MessageResponse {
    pub messages: Vec<Message>,
    pub length: u32
} 

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    DefaultAnswer { status:ResponseStatus},
    CreateViewingKey { key: ViewingKey },
}

// Do we need this?
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WalletInfoResponse {
    pub init: bool,
    pub all_paths: Vec<String>
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    Success,
    Failure,
}


