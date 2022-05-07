use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::HumanAddr;

use crate::state::File;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub owner: Option<HumanAddr> //- don't need this?
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {//can add entropy after testing InitAddress without viewing key
    InitAddress {/*entropy: String*/ },
    SendFile { to: HumanAddr, contents: String },
    SetViewingKey {
        key: String,
        padding: Option<String>,
    },

}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    GetFiles { address: HumanAddr},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ViewingPermissions {
    pub key: Option<String>
    //got rid of permit
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
//not sure if we need this
pub struct fileResponse {
    pub files: Vec<File>,
    pub length: u32
}

pub enum HandleAnswer {
    DefaultAnswer { status:ResponseStatus},
    //CreateViewingKey { key: ViewingKey },
}
pub enum ResponseStatus {
    Success,
    Failure,
}
