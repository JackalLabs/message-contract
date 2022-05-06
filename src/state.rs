use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, Storage, HumanAddr, StdResult, ReadonlyStorage, HandleResponse};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton, PrefixedStorage, ReadonlyPrefixedStorage};
use secret_toolkit::storage::{AppendStore, AppendStoreMut};


const MAX_LENGTH: u16 = 280;
pub static CONFIG_KEY: &[u8] = b"config"; //this is for initializing the contract 
pub const PREFIX_MSGS: &[u8] = b"transactions"; //A prefix to make namespace longer (this is NOT the viewing key--it's just key for each value inside of Storage)
//pub const PERFIX_PERMITS: &str = "revoked_permits"; this is for the permit system - likely will delete 

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub owner: CanonicalAddr, //owner address
    pub contract: HumanAddr //are they saving the contract address? 
}

pub fn config<S: Storage>(storage: &mut S) -> Singleton<S, State> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read<S: Storage>(storage: &S) -> ReadonlySingleton<S, State> {
    singleton_read(storage, CONFIG_KEY)
}

#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Clone, Debug)]
pub struct Message {
    pub from: HumanAddr, //sender
    pub message: String,
    pub block_time: u64
}

// HandleMsg FILE
#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Debug, Clone)]
pub struct File{
    contents: String,
    owner: String,
    public: bool,
    //allow_read_list: OrderedSet<String>, this should definitely not be here 
    //allow_write_list: OrderedSet<String> - not needed? 
}

impl File {

    pub fn new(contents: String, owner: String, public: bool) -> Self {
        Self {
            contents,
            owner,
            public,
            //can add allow write list later; however, this is just for sharing 
            //single files with other people, and we're building it so that only 
            //the owner can view their files - why would the owner want to write
            //to this File struct when this system is built only for simple viewing?
            
        }
    }

    pub fn get_contents(&self) -> &str {
        &self.contents
    }

    /** 
       Please call these before doing anything to files. If you are adding a newly 
       created file to a folder, please check that you can write to the folder. If 
       the file exists, just check the file permission since they overwrite the 
       folder. 
     */
    //leaving these function prototypes here just to remember that they exist 
    // pub fn can_read(&self, address:String) -> bool{}
    // pub fn can_write(&self, address:String) -> bool{}
    // pub fn allow_read(&mut self, address:String) -> bool {}
    // pub fn allow_write(&mut self, address:String) -> bool {}
    // pub fn disallow_read(&mut self, address:String) -> bool {}
    // pub fn disallow_write(&mut self, address:String) -> bool {}

    pub fn make_public(&mut self) -> bool {
        self.public = true;
        true
    }

    pub fn make_private(&mut self) -> bool {
        self.public = false;
        true
    }

    pub fn is_public(&self) -> bool {
        self.public
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
}


pub fn append_file<S: Storage> (
    store: &mut S,
    file: &File,
    for_address: &HumanAddr,
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

    println!("empty collection successfuly made");

    // Try to access the storage of files for the account.
    // If it doesn't exist yet, return a file that says nothing 
    let mut store = AppendStoreMut::<File, _, _>::attach_or_create(&mut store)?; 
    // let store = if let Some(result) = store {
    //     result?
    // } else {
    //     return Ok(File::new("nothing".to_string(), "None".to_string(), false))
    // };

    Ok(HandleResponse::default())
}

//going to delete Message stuff after we finish writing FILE 
impl Message {
    pub fn new(from: HumanAddr, message: String, block_time: u64) -> Self {
        Self {
            from,
            message,
            block_time
        }
    }

    pub fn validate(&self) -> bool {
        return self.message.len() <= usize::from(MAX_LENGTH); //ensure message not longer than max length 
    }

    pub fn store_message<S: Storage>(&self, store: &mut S, to: &HumanAddr) -> StdResult<()> {
        append_msg(store, &self, to) //storing a message for a specific address
    }

    pub fn len<S: ReadonlyStorage>(storage: &S,
                                   for_address: &HumanAddr) -> u32 {
        let store = ReadonlyPrefixedStorage::multilevel(
            &[PREFIX_MSGS, for_address.0.as_bytes()],
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

    pub fn get_messages<S: ReadonlyStorage>(
        storage: &S,
        for_address: &HumanAddr,
        page: u32,
        page_size: u32,
    ) -> StdResult<(Vec<Self>, u64)> {
        let store = ReadonlyPrefixedStorage::multilevel(
            &[PREFIX_MSGS, for_address.0.as_bytes()],
            storage
        );

        // Try to access the storage of txs for the account.
        // If it doesn't exist yet, return an empty list of transfers.
        let store = AppendStore::<Message, _, _>::attach(&store);
        let store = if let Some(result) = store {
            result?
        } else {
            return Ok((vec![], 0));
        };

        // Take `page_size` txs starting from the latest tx, potentially skipping `page * page_size`
        // txs from the start.
        let tx_iter = store
            .iter()
            .rev()
            .skip((page * page_size) as _)
            .take(page_size as _);

        let txs: StdResult<Vec<Message>> = tx_iter
            .map(|tx| tx)
            .collect();
        txs.map(|txs| (txs, store.len() as u64))
    }
}


fn append_msg<S: Storage>(
    store: &mut S,
    msg: &Message, //I think we can put a FILE here 
    for_address: &HumanAddr, //this is going to be used as the key, i.e., Address/1, Address/2, etc. 
) -> StdResult<()> {
    let mut store = PrefixedStorage::multilevel(&[PREFIX_MSGS, for_address.0.as_bytes()], store); //here, store is &mut S 
    //do we need to change the prefix everytime? Or will having their address just give us access to all their files? 
    //I think the collection is all connected so long as you have for_address
    //update - attach_or_create will either attach a Storage to the collection, or use the Storage as a brand new Appendstore,
    //this means we don't need to do the whole Address/1, Address/2, etc. 
    let mut store = AppendStoreMut::attach_or_create(&mut store)?; //this is different from store above, their assignment is different. We borrow as mutable store?
    
    store.push(msg)
}

