use std::convert::TryInto;

use crate::msg::{HandleAnswer};
use crate::state::{append_message, Message, State, /*PERFIX_PERMITS*/ PREFIX_MSGS_RECEIVED, CONFIG_KEY, load, write_viewing_key};
use crate::viewing_key::ViewingKey;
use cosmwasm_std::{ to_binary, Api, Env, Extern, HandleResponse, HumanAddr, Querier, StdError, StdResult, Storage, ReadonlyStorage,
};

use cosmwasm_storage::{ReadonlyPrefixedStorage};
use secret_toolkit::storage::{AppendStore};

// HandleMsg::InitAddress
/*append_message will first create an appendStore space for whoever called try_init. We save a dummy message at index 0 
so we can easily retrieve the owner of the space and possibly handle edge cases*/

pub fn try_init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    entropy: String,
) -> StdResult<HandleResponse> {

    let already_init = collection_exist(&mut deps.storage, &env.message.sender);

    let ha = deps.api.human_address(&deps.api.canonical_address(&env.message.sender)?)?;

    let dummy_message = Message::new(String::from("Placeholder/homefolder/dummy.jpg"), String::from(env.message.sender.as_str()));

    match already_init{
        false => {
            let _messages = append_message(&mut deps.storage, &dummy_message, &ha);

            let config: State = load(&mut deps.storage, CONFIG_KEY)?;
            let prng_seed = config.prng_seed;
            let key = ViewingKey::new(&env, &prng_seed, (&entropy).as_ref());
            let message_sender = deps.api.canonical_address(&env.message.sender)?;
            write_viewing_key(&mut deps.storage, &message_sender, &key);
            Ok(HandleResponse {
                messages: vec![],
                log: vec![],
                data: Some(to_binary(&HandleAnswer::CreateViewingKey { key })?),
            })
        }
        true => {
            let error_message = format!("user has already been initiated!");
            Err(StdError::generic_err(error_message))
        }
        }
    }
    
pub fn try_create_viewing_key<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    entropy: String,
) -> StdResult<HandleResponse> {
    let config: State = load(&mut deps.storage, CONFIG_KEY)?;
    let prng_seed = config.prng_seed;

    let key = ViewingKey::new(&env, &prng_seed, (&entropy).as_ref());

    let message_sender = deps.api.canonical_address(&env.message.sender)?;

    write_viewing_key(&mut deps.storage, &message_sender, &key);

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::CreateViewingKey { key })?),
    })
}

//using store.is_empty() was also another attempted approach - might need it in future for something else. 
pub fn collection_exist<'a, S: Storage>(
    store: &'a mut S,
    for_address: &HumanAddr,
    
) -> bool{

    let store = ReadonlyPrefixedStorage::multilevel(
        &[PREFIX_MSGS_RECEIVED, for_address.0.as_bytes()],
        store
    );

    // Try to access the storage of files for the account.
    let store = AppendStore::<Message, _, _>::attach(&store);
    let store = if let Some(result) = store {
        result
    } else {
        return false
    };

    match store {
        Ok(_v) => {return true},
        Err(_e) => return false,
    };
}

//retrieve message given position of message in collection--would have to call
//get_messages first. Could also possibly retrieve message given path but would be a bit harder. 
//does front end need this or can front end create a function that 
//retrieves a single message from the vector that's returned by get_messages? 

pub fn get_message<S: ReadonlyStorage>(
    storage: &S,
    for_address: &HumanAddr,
    position: u32
) -> StdResult<Message> {

    let store = ReadonlyPrefixedStorage::multilevel(
        &[PREFIX_MSGS_RECEIVED, for_address.0.as_bytes()],
        storage
    );

    // Try to access the storage of files for the account.
    // If it doesn't exist yet, return a Message with path called "Does Not Exist" 
    let store = AppendStore::<Message, _, _>::attach(&store);

    let store = if let Some(result) = store {
        result?
    } else {
        return Ok(Message::new(String::from("Does Not Exist/"), String::from("None")))
    };

    store.get_at(position)
} 

pub fn get_messages<S: ReadonlyStorage>(
    storage: &S,
    behalf: &HumanAddr,

) -> StdResult<Vec<Message>> {
    let store = ReadonlyPrefixedStorage::multilevel(
        &[PREFIX_MSGS_RECEIVED, behalf.0.as_bytes()],
        storage
    );

    // Try to access the collection for the account.
    // If it doesn't exist yet, return an empty collection.
    let store = AppendStore::<Message, _, _>::attach(&store);
    let store = if let Some(result) = store {
        result?
    } else {
        return Ok(vec![]);
    };
    
    let tx_iter = store
        .iter()
        .take(store.len().try_into().unwrap());

    let txs: StdResult<Vec<Message>> = tx_iter
        .map(|tx| tx)
        .collect();
        txs.map(|txs| (txs)) 
}

// Bi's notes to self: 
// Previous version of get_messages returned the vector of messages AND the length of the vector--this overcomplicates things
// and is not needed because we already have a len function built in. See code below for reference.
// let txs: StdResult<Vec<Message>> = tx_iter
// .map(|tx| tx)
// .collect();
// txs.map(|txs| (txs, store.len() as u64)) //the length of the collection of messages is also returned -- do we need it?
// }

//create_empty_collection(& mut deps.storage, &ha); - Experimented with possibility of just creating an empty collection
// let mut store = PrefixedStorage::multilevel(&[PREFIX_MSGS, ha.0.as_bytes()], &mut deps.storage);
// let store = AppendStore::<Message, _, _>::attach(&store);

//Register Wallet info - may not need to do this unless Erin needs it? 