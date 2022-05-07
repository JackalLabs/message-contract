use std::convert::TryInto;

use crate::msg::{HandleMsg, InitMsg, fileResponse, QueryMsg, ViewingPermissions};
use crate::state::{config, append_file, create_empty_collection, File, State, /*PERFIX_PERMITS*/ PREFIX_MSGS};
use cosmwasm_std::{
    debug_print, to_binary, Api, Binary, Env, Extern, HandleResponse, HumanAddr, InitResponse,
    Querier, StdError, StdResult, Storage, ReadonlyStorage,
};

use cosmwasm_storage::{ReadonlyPrefixedStorage, PrefixedStorage};
use secret_toolkit::storage::{AppendStore, AppendStoreMut};

// HandleMsg::InitAddress
pub fn try_init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
        //entropy: String,
) -> StdResult<HandleResponse> {
    let ha = deps.api.human_address(&deps.api.canonical_address(&env.message.sender)?)?;
    //let adr = String::from(ha.clone().as_str());
    let file1 = File::new("Hasbullah.jpg".to_string(), "anyone".to_string(), false);
    //creating an AppendStore collection for sender with a dummy file for testing purposes
    append_file(&mut deps.storage, &file1, &ha);

    //creating an empty Appendstore collection for sender 
    //create_empty_collection(& mut deps.storage, &ha);

    


    // let mut store = PrefixedStorage::multilevel(&[PREFIX_MSGS, ha.0.as_bytes()], &mut deps.storage);
    // let store = AppendStore::<File, _, _>::attach(&store);

    //Can register wallet info and viewing key after testing init address
    // //Register Wallet info
    // let wallet_info = WalletInfo { 
    //     init : true
    // };
    // let bucket_response = bucket(FILE_LOCATION, &mut deps.storage).save(&adr.as_bytes(), &wallet_info);
    // match bucket_response {
    //     Ok(bucket_response) => bucket_response,
    //     Err(e) => panic!("Bucket Error: {}", e)
    // }

    // // Let's create viewing key - creates a viewing key for whoever made this collection O.o
    // let config: State = load(&mut deps.storage, CONFIG_KEY)?;
    // let prng_seed = config.prng_seed;
    // let key = ViewingKey::new(&env, &prng_seed, (&entropy).as_ref());
    // let message_sender = deps.api.canonical_address(&env.message.sender)?;
    // write_viewing_key(&mut deps.storage, &message_sender, &key);

    // Ok(HandleResponse::default())
    Ok(HandleResponse::default())
}

//if you have queried to retrieve a vector of all the files, you know what position your desired file is at 
//so you can throw in the position and get the file - is this enough for front end to buy buttons to retrieve contents of a file?
//consider writing fn to get file given a contents: String instead of position? Would be a lot harder 

pub fn get_file<S: ReadonlyStorage>(
    storage: &S,
    for_address: &HumanAddr,
    position: u32
) -> StdResult<File> {

    let store = ReadonlyPrefixedStorage::multilevel(
        &[PREFIX_MSGS, for_address.0.as_bytes()],
        storage
    );

    // Try to access the storage of files for the account.
    // If it doesn't exist yet, return a file that says nothing 
    let store = AppendStore::<File, _, _>::attach(&store);
    let store = if let Some(result) = store {
        result?
    } else {
        return Ok(File::new("nothing".to_string(), "None".to_string(), false))
    };

    store.get_at(position)
} 

pub fn get_files<S: ReadonlyStorage>(
    storage: &S,
    for_address: &HumanAddr,

) -> StdResult<(Vec<File>, u64)> {
    let store = ReadonlyPrefixedStorage::multilevel(
        &[PREFIX_MSGS, for_address.0.as_bytes()],
        storage
    );

    // Try to access the collection for the account.
    // If it doesn't exist yet, return an empty collection.
    let store = AppendStore::<File, _, _>::attach(&store);
    let store = if let Some(result) = store {
        result?
    } else {
        return Ok((vec![], 0));
    };

    
    let tx_iter = store
        .iter()
        //.rev()
        .take(store.len().try_into().unwrap());

    let txs: StdResult<Vec<File>> = tx_iter
        .map(|tx| tx)
        .collect();
        txs.map(|txs| (txs, store.len() as u64))
}
