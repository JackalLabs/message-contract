use std::any::type_name;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, Storage, HumanAddr, StdResult, StdError, ReadonlyStorage, HandleResponse};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton, PrefixedStorage, ReadonlyPrefixedStorage};
use secret_toolkit::storage::{AppendStore, AppendStoreMut};
use secret_toolkit::serialization::{Bincode2, Serde};
use serde::de::DeserializeOwned;

use crate::viewing_key::ViewingKey;

pub static CONFIG_KEY: &[u8] = b"config"; //this is for initializing the contract 
pub const PREFIX_MSGS_RECEIVED: &[u8] = b"messages_received"; //A prefix to make namespace longer

pub const PREFIX_VIEWING_KEY: &[u8] = b"viewingkey";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub owner: CanonicalAddr, 
    pub contract: HumanAddr, 
    pub prng_seed: Vec<u8>,

}

pub fn config<S: Storage>(storage: &mut S) -> Singleton<S, State> {
    singleton(storage, CONFIG_KEY)
}

pub fn save<T: Serialize, S: Storage>(storage: &mut S, key: &[u8],value: &T) -> StdResult<()> {
    storage.set(key, &Bincode2::serialize(value)?);
    Ok(())
}

pub fn load<T: DeserializeOwned, S: ReadonlyStorage>(storage: &S, key: &[u8]) -> StdResult<T> {
    Bincode2::deserialize(
        &storage
            .get(key)
            .ok_or_else(|| StdError::not_found(type_name::<T>()))?,
    )
}

pub fn config_read<S: Storage>(storage: &S) -> ReadonlySingleton<S, State> {
    singleton_read(storage, CONFIG_KEY)
}
pub fn write_viewing_key<S: Storage>(store: &mut S, owner: &CanonicalAddr, key: &ViewingKey) {
    let mut user_key_store = PrefixedStorage::new(PREFIX_VIEWING_KEY, store);
    user_key_store.set(owner.as_slice(), &key.to_hashed());
}

pub fn read_viewing_key<S: Storage>(store: &S, owner: &CanonicalAddr) -> Option<Vec<u8>> {
    let user_key_store = ReadonlyPrefixedStorage::new(PREFIX_VIEWING_KEY, store);
    user_key_store.get(owner.as_slice())
}

// HandleMsg Message
#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Debug, Clone)]
pub struct Message{
    
    contents: String, //contents will be a message that Erin will customize on the frontend.
    //Front end will have a way of connecting JACKAL-storage with JACKAL-filesharing in order for this to work.
    owner: String

}

impl Message {

    pub fn new(contents: String, owner: String) -> Self {
        Self {
            contents,
            owner,
        }
    }

    pub fn get_contents(&self) -> &str {
        &self.contents
    }

    pub fn get_owner(&self) -> &str {
        &self.owner
    }

    pub fn store_message<S:Storage>(&self, store: &mut S, to: &HumanAddr) -> StdResult<()>{
        append_message(store, &self, to)
    }

    //returns length of the collection that this message belongs in. Used for testing
    pub fn len<S: ReadonlyStorage>(storage: &S,
                                   for_address: &HumanAddr) -> u32 {
        let store = ReadonlyPrefixedStorage::multilevel(
            &[PREFIX_MSGS_RECEIVED, for_address.0.as_bytes()],
            storage
        );
        let store = AppendStore::<Message, _, _>::attach(&store);
        let store = if let Some(result) = store {
            if result.is_err() {
                return 0;
            } else {
                result.unwrap()
            }
        } else {
            return 0;
        };

        return store.len();
    }
}

//see notes below regarding AppendStore
pub fn append_message<S: Storage> (
    store: &mut S,
    message: &Message,
    for_address: &HumanAddr, 
) -> StdResult<()>{
    
    let option_error_message = format!("Provided storage doesn't seem like an AppendStore");            
    let mut store = PrefixedStorage::multilevel(&[PREFIX_MSGS_RECEIVED, for_address.0.as_bytes()], store);
    let mut store = AppendStoreMut::attach(&mut store).unwrap_or(Err(StdError::generic_err(option_error_message)))?;
    
    store.push(message)
}

pub fn create_empty_collection<S: Storage> (
    store: &mut S,
    for_address: &HumanAddr,
) -> StdResult<HandleResponse>{

    let mut store = PrefixedStorage::multilevel(
        &[PREFIX_MSGS_RECEIVED, for_address.0.as_bytes()],
        store
    );
    let _store = AppendStoreMut::<Message, _, _>::attach_or_create(&mut store)?;
    Ok(HandleResponse::default())
}
/*

Note from append_store.rs:

An "append store" is a storage wrapper that guarantees constant-cost appending to and popping
from a list of items in storage.

This is achieved by storing each item in a separate storage entry. A special key is reserved
for storing the length of the collection so far.

Note from Bi: 

"This is achieved by storing each item in a separate storage entry" - to expand on this:

Every single storage entry that belongs to the list has to have the same namespace. multilevel is simply a 
way of combining 'PREFIX_MSGS_RECEIVED' to 'for_address' to make a long namespace. The benefits of this are
unclear at the moment but preference is to keep the namespace long.

attach_or_create function looked useful but for our purposes, I wanted to have more control over
the list creation process using if-else statements - check out send_message and try_init for some context 

pub const PREFIX_MSGS_SENT: &[u8] = b"messages_sent"; 
Possibly, going to use this as a prefix for saving a collection of messages that user has sent to handle edge cases

*/










