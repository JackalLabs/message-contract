use std::convert::TryInto;

use crate::msg::{HandleMsg, InitMsg, MessageResponse, QueryMsg, ViewingPermissions};
use crate::state::{config, append_message, create_empty_collection, Message, State, /*PERFIX_PERMITS*/ PREFIX_MSGS_RECEIVED, save, CONFIG_KEY, read_viewing_key};
use crate::backend::{try_init, get_messages, try_create_viewing_key};
use crate::viewing_key::VIEWING_KEY_SIZE;

use cosmwasm_std::{
    debug_print, to_binary, Api, Binary, Env, Extern, HandleResponse, HumanAddr, InitResponse,
    Querier, StdError, StdResult, Storage, ReadonlyStorage, QueryResult,
};

use cosmwasm_storage::{ReadonlyPrefixedStorage, PrefixedStorage};
use secret_toolkit_crypto::sha_256;
use secret_toolkit::storage::{AppendStore, AppendStoreMut};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    
    let config = State {
        owner: deps.api.canonical_address(&env.message.sender)?,
        contract: env.contract.address,
        prng_seed: sha_256(base64::encode(msg.prng_seed).as_bytes()).to_vec(), 
    };

    //config(&mut deps.storage).save(&state)?;

    debug_print!("Contract was initialized by {}", env.message.sender);

    save(&mut deps.storage, CONFIG_KEY, &config)?;
    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::SendMessage { to, path } => send_message(deps, env, to, path),
        HandleMsg::CreateViewingKey { entropy, .. } => try_create_viewing_key(deps, env, entropy),
        HandleMsg::InitAddress { entropy } => try_init(deps, env, entropy),
        // HandleMsg::SetViewingKey { key, .. } => create_key(deps, env, key),
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    env: Env,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        //QueryMsg::GetPath { behalf, path, key } => todo!(), - going to be in authenticated queries
        //QueryMsg::GetWalletInfo { behalf, key } => todo!(), - don't need 
        _=> authenticated_queries(deps, env, msg),
    }
} 

fn authenticated_queries<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    env: Env,
    msg: QueryMsg,
) -> QueryResult {
    let (addresses, key) = msg.get_validation_params();

    for address in addresses {
        let canonical_addr = deps.api.canonical_address(address)?;

        let expected_key = read_viewing_key(&deps.storage, &canonical_addr);

        if expected_key.is_none() {
            // Checking the key will take significant time. We don't want to exit immediately if it isn't set
            // in a way which will allow to time the command and determine if a viewing key doesn't exist
            key.check_viewing_key(&[0u8; VIEWING_KEY_SIZE]);
        } else if key.check_viewing_key(expected_key.unwrap().as_slice()) {

            return match msg {
                QueryMsg::GetMessages { behalf, .. } => to_binary(&query_messages(deps, env, &behalf)?),
                //QueryMsg::GetWalletInfo { behalf, .. } => to_binary(&query_wallet_info(deps, &behalf)?),
                _ => panic!("How did this even get to this stage. It should have been processed.")
            };
        }
    }

    Err(StdError::unauthorized())
}

// pub fn create_key<S: Storage, A: Api, Q: Querier> - going to put back later

pub fn send_message<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    to: HumanAddr,
    path: String,
) -> StdResult<HandleResponse> {
    let file = Message::new(String::from(path), env.message.sender.to_string());
    file.store_message(&mut deps.storage, &to)?;

    debug_print(format!("file stored successfully to {}", to));
    Ok(HandleResponse::default())
}

fn query_messages<S: Storage, A: Api, Q: Querier>(
        deps: &Extern<S, A, Q>,
        env: Env,
        behalf: &HumanAddr,
    ) -> StdResult<MessageResponse> {
        

        //need to do check: 
        //(if behalf == env.message.sender) 
        //  return the vector of messages
        //      else
        //      return error message: "Sorry can only get your own messages!"                  
        let msgs = get_messages(
                &deps.storage,
                &behalf,
            )?
            .0;
    
        let len = Message::len(&deps.storage, &behalf);   
        Ok(MessageResponse {messages: msgs, length: len})
    }


#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, MockStorage};
    use cosmwasm_std::{coins, from_binary, ReadonlyStorage};
    use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
    use crate::msg::{MessageResponse, HandleAnswer, self, /*WalletInfoResponse*/};

    fn init_for_test<S: Storage, A: Api, Q: Querier> (
        deps: &mut Extern<S, A, Q>,
        address:String,
    ) /*-> ViewingKey*/ {

        // Init Contract
        let msg = InitMsg {owner: None, prng_seed: todo!() };
        let env = mock_env("creator", &[]);
        let _res = init(deps, env, msg).unwrap();

        // Init Address and Create ViewingKey
        let env = mock_env(String::from(&address), &[]);
        let msg = HandleMsg::InitAddress { entropy: String::from("Entropygoeshereboi") };
        let handle_response = handle(deps, env, msg).unwrap();
        // let vk = match from_binary(&handle_response.data.unwrap()).unwrap() {
        //     HandleAnswer::CreateViewingKey { key } => {
        //         key
        //     },
        //     _ => panic!("Unexpected result from handle"),
        // };
        // vk
    }

    //double check this one monday 
    #[test]
    fn contract_init() {
        let mut deps = mock_dependencies(20, &[]);

        let msg = InitMsg {owner: None, prng_seed:String::from("lets init bro")};
        let env = mock_env("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = init(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());
    } 

    #[test]
    fn send_file() {
        let mut deps = mock_dependencies(20, &coins(2, "token"));
        
        let msg = InitMsg { owner: None, prng_seed:String::from("lets init bro") };
        let env = mock_env("creator", &coins(2, "token"));
        let _res = init(&mut deps, env, msg).unwrap();
        
        //sending a file to contract creator's address 
        let env = mock_env("Bi", &coins(2, "token"));
        let msg = HandleMsg::SendMessage {
            to: HumanAddr("creator".to_string()),
            path: "home/pepe.jpg".to_string(),
        };
        let res = handle(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());

        //sending a file to Address_A's collection

        let env = mock_env("Address_A", &coins(2, "token"));
        try_init(&mut deps, env, String::from("Entropygoeshereboi") );

        let env = mock_env("Bi", &coins(2, "token"));
        let msg = HandleMsg::SendMessage {
            to: HumanAddr("Address_A".to_string()),
            path: "Abdul.pdf".to_string(),
        };
        let res = handle(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let env = mock_env("Bi", &coins(2, "token"));
        //sending another file to address_A's collection
        let msg = HandleMsg::SendMessage {
            to: HumanAddr("Address_A".to_string()),
            path: "home/Khabib.pdf".to_string(),
        };
        let res = handle(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let length = Message::len(&mut deps.storage, &HumanAddr::from("Address_A"));
        println!("Length of Address_A collection is {}\n", length);
        let A_allfiles = get_messages(&mut deps.storage, &HumanAddr::from("Address_A"));
        println!("{:?}", A_allfiles);

    }

    //consider implementing some send_file query tests into below test and see if the the addresses keep getting updated
    #[test]
    fn append_store_works() {
        //append.store.rs is a storage rapper that creates collections of items
        //When we call store_file and pass in a recipient address, append.store.rs will use this address as a namespace for the 
        //collection. Everytime you store a file using the same address, the file is added to that address's collection because
        //it corresponds to the same namespace.
        //calling store_file for a different address will initiate a new collection for a different namespace
        //appendstore supports iterating, pushing, and popping, retrieving files at specific indexes is possible 

        //init contract 
        let mut deps = mock_dependencies(20, &coins(2, "token"));       
        let msg = InitMsg { owner: None, prng_seed:String::from("lets init bro") };
        let env = mock_env("creator", &coins(2, "token"));
        let _res = init(&mut deps, env, msg).unwrap();
    
        //initializing Appendstore empty collection for Address_A 
        let env = mock_env("Address_A", &coins(2, "token"));
        try_init(&mut deps, env, String::from("Entropygoeshereboi") );

        //saving a file for Address_A's collection
        let file = Message::new("home/King_Pepe.jpg".to_string(), "anyone".to_string());
        file.store_message(&mut deps.storage, &HumanAddr::from("Address_A"));

        let file2 = Message::new("home/Queen_Pepe.jpg".to_string(), "anyone".to_string());
        file2.store_message(&mut deps.storage, &HumanAddr::from("Address_A"));

        //printing length of collection should display 3
        let length = Message::len(&mut deps.storage, &HumanAddr::from("Address_A"));
        println!("Length of Address_A collection is {}\n", length);
        let A_allfiles = get_messages(&mut deps.storage, &HumanAddr::from("Address_A"));
        println!("{:?}", A_allfiles);
        //let ha2 = deps.api.human_address(&deps.api.canonical_address(&env.message.sender).unwrap());
        //note: if human address is 2 characters or fewer, unwrapping fails 

        //initializing Appendstore empty collection for Address_B 
        let env = mock_env("Address_B", &coins(2, "token"));
        try_init(&mut deps, env, String::from("entropygoeshereboi"));

        //saving a file for Address_B's collection
        let file = Message::new("home/B_Coin.jpg".to_string(), "anyone".to_string());
        file.store_message(&mut deps.storage, &HumanAddr::from("Address_B"));


        let file2 = Message::new("home/C_Coin.jpg".to_string(), "anyone".to_string());
        file2.store_message(&mut deps.storage, &HumanAddr::from("Address_A"));


        //printing length of collection should display 3
        let length = Message::len(&mut deps.storage, &HumanAddr::from("Address_B"));
        println!("Length of Address_B collection is {}\n", length);
        let B_allfiles = get_messages(&mut deps.storage, &HumanAddr::from("Address_B"));
        println!("{:?}", B_allfiles);

        let updatedlength = Message::len(&mut deps.storage, &HumanAddr::from("Address_A"));
        println!("Length of Address_A collection is {}\n", updatedlength); //print 4 
        let A_allfiles = get_messages(&mut deps.storage, &HumanAddr::from("Address_A"));
        println!("{:?}", A_allfiles);

    } 

}


    // #[test]
    // fn create_key() {
    //     let mut deps = mock_dependencies(20, &coins(2, "token"));

    //     let msg = InitMsg { owner: None };
    //     let env = mock_env("creator", &coins(2, "token"));
    //     let _res = init(&mut deps, env, msg).unwrap();

    //     let env = mock_env("anyone", &coins(2, "token"));
    //     let msg = HandleMsg::SetViewingKey {
    //         key: "yoyo".to_string(),
    //         padding: None,
    //     };
    //     let res = handle(&mut deps, env, msg).unwrap();

    //     assert_eq!(0, res.messages.len());
    // }

    
    /*
    #[test]
    fn read_message() {}*/
 
    // #[test]
    // fn read_message_fail() {}
// 