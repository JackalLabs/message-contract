use std::any::type_name;
use std::convert::TryInto;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, Storage, HumanAddr, StdResult, ReadonlyStorage, HandleResponse};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton, PrefixedStorage, ReadonlyPrefixedStorage};
use secret_toolkit::storage::{AppendStore, AppendStoreMut};
use secret_toolkit::serialization::{Bincode2, Serde};
use serde::de::DeserializeOwned;

use crate::viewing_key::ViewingKey;


pub static CONFIG_KEY: &[u8] = b"config"; //this is for initializing the contract 
pub const PREFIX_MSGS: &[u8] = b"collection_of"; //A prefix to make namespace longer (this is NOT the viewing key--it's just key for each value inside of Storage)
//pub const PERFIX_PERMITS: &str = "revoked_permits"; this is for the permit system - likely will delete 
pub const PREFIX_VIEWING_KEY: &[u8] = b"viewingkey";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub owner: CanonicalAddr, //owner address
    pub contract: HumanAddr, //are they saving the contract address? 
    pub prng_seed: Vec<u8>,

}

pub fn config<S: Storage>(storage: &mut S) -> Singleton<S, State> {
    singleton(storage, CONFIG_KEY)
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


// HandleMsg FILE
//Even though we are sending a message that contains link(s) to a file, I decided to call this struct "File" instead of "Message" because it's 
//more intuitive, i.e., this is a struct named File which contains details that allow the recipient to access a specific file
//inside of the sender's folders. I could change it back to Message easily though. 
#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Debug, Clone)]
pub struct File{
    contents: String,
    owner: String,
    path: String, //path for that file that was created inside of JACKAL-storage, associated with the owner of said File. 
    //Front end will have a way of connecting JACKAL-storage with JACKAL-filesharing in order for this to work?
    public: bool
    //removed allow_read_list and allow_write_list. This system allows only a user to view files saved at a storage space associated
    //with their specific address. 
    //Should remove public: bool? 

    //if A sends a file's details (which is inside of A's folders) to B's appendStore space, is the contents and path enough for B to be able to view this file and retrieve it
    //from IPS/file coin, or does 
    //A somehow need to 1. retrieve the file to be sent. 2. update that file's allow_read_list to include B's address,and then 3. send the file. 
    //This would be quite hard because all the code for allow_read_list is inside of JACKAL - storage, and I'm not sure how to make this contract
    //communicate with the storage contract to enable this to happen

}

impl File {

    pub fn new(contents: String, owner: String, path: String, public: bool) -> Self {
        Self {
            contents,
            owner,
            path,
            public
        }
    }

    pub fn get_contents(&self) -> &str {
        &self.contents
    }

    pub fn store_file<S:Storage>(&self, store: &mut S, to: &HumanAddr) -> StdResult<()>{
        append_file(store, &self, to)
    }

    //returns length of the collection that this file belongs in 
    //possibly move it into contract.rs? 
    pub fn len<S: ReadonlyStorage>(storage: &S,
                                   for_address: &HumanAddr) -> u32 {
        let store = ReadonlyPrefixedStorage::multilevel(
            &[PREFIX_MSGS, for_address.0.as_bytes()],
            storage
        );
        let store = AppendStore::<File, _, _>::attach(&store);
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

    //Below fn is taken from Cashmaney/memo - do we need to validate the File?...
    // pub fn validate(&self) -> bool {
    //     return self.message.len() <= usize::from(MAX_LENGTH); //ensure message not longer than max length 
    // }
}

 //do we need to change the prefix everytime? Or will having their address just give us access to all their files? 
//I think the collection is all connected so long as you have for_address
//update - attach_or_create will either attach a Storage to the collection, or use the Storage as a brand new Appendstore,
//this means we don't need to do the whole Address/1, Address/2, etc. 
pub fn append_file<S: Storage> (
    store: &mut S,
    file: &File,
    for_address: &HumanAddr, //this is the name space 
) -> StdResult<()>{
    let mut store = PrefixedStorage::multilevel(&[PREFIX_MSGS, for_address.0.as_bytes()], store);
    let mut store = AppendStoreMut::attach_or_create(&mut store)?; //this is different from store above, their assignment is different. We borrow as mutable store?  
    store.push(file)
}

pub fn create_empty_collection<S: Storage> (
    store: &mut S,
    for_address: &HumanAddr,
) -> StdResult<HandleResponse>{

    let mut store = PrefixedStorage::multilevel(
        &[PREFIX_MSGS, for_address.0.as_bytes()],
        store
    );
    /// Try to use the provided storage as an AppendStore. If it doesn't seem to be one, then
    /// initialize it as one. Returns Err if the contents of the storage can not be parsed.
    /// below used to be: let mut store, but warning during test told me to get rid of "mut"
    let store = AppendStoreMut::<File, _, _>::attach_or_create(&mut store)?;
    Ok(HandleResponse::default())
}









