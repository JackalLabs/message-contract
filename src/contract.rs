use crate::msg::{HandleMsg, InitMsg, MessageResponse, QueryMsg};
use crate::state::{Message, State, save, CONFIG_KEY, read_viewing_key, create_empty_collection, append_message};
use crate::backend::{try_init, get_messages, try_create_viewing_key, delete_all_messages, get_collection_owner, collection_exist};
use crate::viewing_key::VIEWING_KEY_SIZE;

use cosmwasm_std::{
    debug_print, to_binary, Api, Binary, Env, Extern, HandleResponse, HumanAddr, InitResponse,
    Querier, StdError, StdResult, Storage, QueryResult,
};

use secret_toolkit_crypto::sha_256;

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

    debug_print!("Contract was initialized by {}", env.message.sender);

    save(&mut deps.storage, CONFIG_KEY, &config)?;
    //config(&mut deps.storage).save(&state)?;

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::InitAddress { entropy } => try_init(deps, env, entropy),
        HandleMsg::CreateViewingKey { entropy, .. } => try_create_viewing_key(deps, env, entropy),
        HandleMsg::SendMessage { to, contents } => send_message(deps, env, to, contents),
        HandleMsg::DeleteAllMessages {} => delete_all_messages(deps, env)
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        _=> authenticated_queries(deps, msg), //could just get rid of the "_=>" ? 
    }
} 

fn authenticated_queries<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
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
                QueryMsg::GetMessages { behalf, .. } => to_binary(&query_messages(deps, &behalf)?),
                //QueryMsg::GetWalletInfo { behalf, .. } => to_binary(&query_wallet_info(deps, &behalf)?),
                _ => panic!("How did this even get to this stage. It should have been processed.")
            };
        }
    }

    Err(StdError::unauthorized())
}

pub fn send_message<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    to: HumanAddr,
    contents: String,
) -> StdResult<HandleResponse> {

    let message = Message::new(String::from(contents), env.message.sender.to_string());

    let already_init = collection_exist(&mut deps.storage, &to);
    //if "to" does not have a collection yet, the owner of this dummy message will be to because it will be placed
    //in the collection that this function makes for them 
    let dummy_message = Message::new(String::from("Dummy_contents.jpg"), String::from(to.as_str()));

    match already_init{
        false => {
            //if recipient does not have a list, make one for them. We let them make their own viewing key. - how to notify that they need to make one? 
            let _storage_space = create_empty_collection(&mut deps.storage, &to);
            let _dummy_messages = append_message(&mut deps.storage, &dummy_message, &to);
            let _saved_message = append_message(&mut deps.storage, &message, &to);
            debug_print(format!("message stored successfully to {}", to));
            Ok(HandleResponse::default())
        }
        true => {

            message.store_message(&mut deps.storage, &to)?;
            debug_print(format!("message stored successfully to {}", to));
            Ok(HandleResponse::default())
        }
        }
}

fn query_messages<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    behalf: &HumanAddr,
) -> StdResult<MessageResponse> {
    
    let mut messages: Vec<Message> = Vec::new();

    let owner = get_collection_owner(&deps.storage, &behalf)?;
    
    if owner == behalf.to_string() {
        let msgs = get_messages(
            &deps.storage,
            &behalf,
        )?;
        messages = msgs
    } else {
        return Err(StdError::generic_err("Can only query your own messages!"));
    }

    
    Ok(MessageResponse {messages: messages})
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use cosmwasm_std::{coins, from_binary};
    use crate::msg::{MessageResponse, HandleAnswer/*WalletInfoResponse*/};
    use crate::viewing_key::ViewingKey;

    fn init_for_test<S: Storage, A: Api, Q: Querier> (
        deps: &mut Extern<S, A, Q>,
        address: String,
    ) -> ViewingKey {

        // Init Contract
        let msg = InitMsg { prng_seed:String::from("lets init bro")};
        let env = mock_env("creator", &[]);
        let _res = init(deps, env, msg).unwrap(); 

        // Init Address and Create ViewingKey
        let env = mock_env(String::from(&address), &[]);
        let msg = HandleMsg::InitAddress { entropy: String::from("Entropygoeshereboi") };
        let handle_response = handle(deps, env, msg).unwrap();

        match from_binary(&handle_response.data.unwrap()).unwrap() {
            HandleAnswer::CreateViewingKey { key } => {
                key
            },
            _=> panic!("Unexpected result from handle"),
        }
    }

    #[test]
    fn test_create_viewing_key() {
        let mut deps = mock_dependencies(20, &[]);

        // init
        let msg = InitMsg {prng_seed:String::from("lets init bro")};
        let env = mock_env("anyone", &[]);
        let _res = init(&mut deps, env, msg).unwrap();
        
        // create viewingkey
        let env = mock_env("anyone", &[]);
        let create_vk_msg = HandleMsg::CreateViewingKey {
            entropy: "supbro".to_string(),
            padding: None,
        };
        let handle_response = handle(&mut deps, env, create_vk_msg).unwrap();
        
        let _vk = match from_binary(&handle_response.data.unwrap()).unwrap() {
            HandleAnswer::CreateViewingKey { key } => {
                // println!("viewing key here: {}",key);
                key
            },
            _ => panic!("Unexpected result from handle"),
        };

    }

    #[test]
    fn send_messages_and_query() {
        let mut deps = mock_dependencies(20, &coins(2, "token"));
        let vk_anyone = init_for_test(&mut deps, String::from("anyone"));
        
        //Changing 'nuggie' to 'anyone' will cause error: "user has already been initiated!"
        let vk_nuggie = init_for_test(&mut deps, String::from("nuggie"));
        
        //sending a message to anyone's address
        let env = mock_env("sender", &[]);
        let msg = HandleMsg::SendMessage {
            to: HumanAddr("anyone".to_string()),
            contents: "Hello: sender has shared Pepe.jpg with you".to_string(),
        };
        let res = handle(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());

        //sending another message to anyone's address

        let env = mock_env("sender", &[]);
        let msg = HandleMsg::SendMessage {
            to: HumanAddr("anyone".to_string()),
            contents: "Hello: sender has shared Hasbullah.jpg with you".to_string(),
        };
        let res = handle(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());
        
        // Query Anyone's Messages
        let query_res = query(&deps, QueryMsg::GetMessages { behalf: HumanAddr("anyone".to_string()), key: vk_anyone.to_string() },).unwrap(); //changing viewing key causes error
        let value: MessageResponse = from_binary(&query_res).unwrap();
        println!("All messages --> {:#?}", value.messages);        

        let length = Message::len(&mut deps.storage, &HumanAddr::from("anyone"));
        println!("Length of anyone's collection is {}\n", length);

        //Query with a different viewing key will fail 
        let query_res = query(&deps, QueryMsg::GetMessages { behalf: HumanAddr("anyone".to_string()), key: vk_nuggie.to_string() }); //changing viewing key causes error
        assert!(query_res.is_err());

        //sending a message to nuggie's address
        let env = mock_env("sender", &[]);
        let msg = HandleMsg::SendMessage {
            to: HumanAddr("nuggie".to_string()),
            contents: "Sender/pepe.jpg".to_string(),
        };
        let res = handle(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // Query Nuggies's Messages
        let query_res = query(&deps, QueryMsg::GetMessages { behalf: HumanAddr("nuggie".to_string()), key: vk_nuggie.to_string() },).unwrap(); //changing viewing key causes error
        let value: MessageResponse = from_binary(&query_res).unwrap();
        println!("All messages --> {:#?}", value.messages);        

        let length = Message::len(&mut deps.storage, &HumanAddr::from("nuggie"));
        println!("Length of nuggie's collection is {}\n", length);

        //Using anyone's viewing key to query nuggie's messages will fail 
        let query_res = query(&deps, QueryMsg::GetMessages { behalf: HumanAddr("nuggie".to_string()), key: vk_anyone.to_string() }); //changing viewing key causes error
        assert!(query_res.is_err());

    }

    #[test]
    fn send_to_uninitiated_address() {
        let mut deps = mock_dependencies(20, &coins(2, "token"));

        // Init Contract
        let msg = InitMsg { prng_seed:String::from("lets init bro")};
        let env = mock_env("creator", &[]);
        let _res = init(&mut deps, env, msg).unwrap();
        
        //sending a message to anyone's address - anyone has NOT initituate a collection for their address
        let env = mock_env("sender", &[]);
        let msg = HandleMsg::SendMessage {
            to: HumanAddr("anyone".to_string()),
            contents: "Hello: sender has shared Pepe.jpg with you".to_string(),
        };
        let res = handle(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());

        //SendMessage above will have made a collection for anyone, and placed above message next to dummy message in this collection. 
        
        //sending another message to anyone's address
        let env = mock_env("sender", &[]);
        let msg = HandleMsg::SendMessage {
            to: HumanAddr("anyone".to_string()),
            contents: "Hello: sender has shared Hasbullah.jpg with you".to_string(),
        };
        let res = handle(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());

        //At this point, anyone has a list, so we only need to create a viewing key for anyone 
        
        // create viewingkey
        let env = mock_env("anyone", &[]);
        let create_vk_msg = HandleMsg::CreateViewingKey {
            entropy: "supbro".to_string(),
            padding: None,
        };
        let handle_response = handle(&mut deps, env, create_vk_msg).unwrap();
        
        let vk_anyone = match from_binary(&handle_response.data.unwrap()).unwrap() {
            HandleAnswer::CreateViewingKey { key } => {
                // println!("viewing key here: {}",key);
                key
            },
            _ => panic!("Unexpected result from handle"),
        };

        // Query Anyone's Messages
        let query_res = query(&deps, QueryMsg::GetMessages { behalf: HumanAddr("anyone".to_string()), key: vk_anyone.to_string() },).unwrap(); //changing viewing key causes error
        let value: MessageResponse = from_binary(&query_res).unwrap();
        println!("All messages --> {:#?}", value.messages);        

        let length = Message::len(&mut deps.storage, &HumanAddr::from("anyone"));
        println!("Length of anyone's collection is {}\n", length);

    }

    #[test]
    fn delete_all_messages() {
        let mut deps = mock_dependencies(20, &coins(2, "token"));
        let vk = init_for_test(&mut deps, String::from("anyone"));
        
        //sending a file to anyone's address
        let env = mock_env("sender", &[]);
        let msg = HandleMsg::SendMessage {
            to: HumanAddr("anyone".to_string()),
            contents: "Sender/pepe.jpg".to_string(),
        };
        let res = handle(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());
        
        // Query Messages
        let query_res = query(&deps, QueryMsg::GetMessages { behalf: HumanAddr("anyone".to_string()), key: vk.to_string() },).unwrap(); //changing viewing key causes error
        let value: MessageResponse = from_binary(&query_res).unwrap();
        println!("All messages --> {:#?}", value.messages);        

        let length = Message::len(&mut deps.storage, &HumanAddr::from("anyone"));
        println!("Length of anyone's collection is {}\n", length);

        //delete all messages
        let env = mock_env("anyone", &[]);
        let msg = HandleMsg::DeleteAllMessages {};
        let res = handle(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // Query Messages should now only display the dummy message
        let query_res = query(&deps, QueryMsg::GetMessages { behalf: HumanAddr("anyone".to_string()), key: vk.to_string() },).unwrap(); //changing viewing key causes error
        let value: MessageResponse = from_binary(&query_res).unwrap();
        println!("All messages --> {:#?}", value.messages);        

        let length = Message::len(&mut deps.storage, &HumanAddr::from("anyone"));
        println!("Length of anyone's collection is {}\n", length);

    }

    #[test]
    fn get_owner() {
        let mut deps = mock_dependencies(20, &coins(2, "token"));
        let vk = init_for_test(&mut deps, String::from("anyone"));
        
        //sending a file to anyone's address
        let env = mock_env("sender", &[]);
        let msg = HandleMsg::SendMessage {
            to: HumanAddr("anyone".to_string()),
            contents: "Sender/pepe.jpg".to_string(),
        };
        let res = handle(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let env = mock_env("anyone", &[]);
        let owner = get_collection_owner(&deps.storage, &env.message.sender).unwrap();
        println!("{}", owner);
        
    }
}   
   
 /*Bi's notes to self: 

 let ha2 = deps.api.human_address(&deps.api.canonical_address(&env.message.sender).unwrap());
 note: if human address is 2 characters or fewer, unwrapping fails 
*/


// fn query_messages<S: Storage, A: Api, Q: Querier>(
//     deps: &Extern<S, A, Q>,
//     behalf: &HumanAddr,
// ) -> StdResult<MessageResponse> {

//         let msgs = get_messages(
//             &deps.storage,
//             &behalf,
//         )?;

//     let len = Message::len(&deps.storage, &behalf);   
//     Ok(MessageResponse {messages: msgs, length: len})
// }

// #[test]
// fn cannot_send_to_uninitiated_address(){
//     //sending a file to an uninitiated address
//     //this deps will contain a storage that has not been changed into an Appendstore space 
//     let mut deps = mock_dependencies(18, &coins(4, "earth_coin"));
//     //init_for_test not called and no viewing key is made 
//     let env = mock_env("sender", &[]);
//     let msg = HandleMsg::SendMessage {
//         to: HumanAddr("Peter".to_string()),
//         contents: "Sender/Abdul.pdf".to_string(),
//     };
//     let res = handle(&mut deps, env, msg).unwrap_err();   
//     println!("{}", res);
// }

/*
    #[test]
    fn append_store_works() {

        //init contract 
        let mut deps = mock_dependencies(20, &coins(2, "token"));       
        let msg = InitMsg { prng_seed:String::from("lets init bro") };
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

        //initializing Appendstore empty collection for Address_B 
        let env = mock_env("Address_B", &coins(2, "token"));
        try_init(&mut deps, env, String::from("entropygoeshereboi"));

        //saving a file for Address_B's collection
        let file = Message::new("home/B_Coin.jpg".to_string(), "anyone".to_string());
        file.store_message(&mut deps.storage, &HumanAddr::from("Address_B"));

        //saving a file for Address_A's collection
        let file2 = Message::new("home/C_Coin.jpg".to_string(), "anyone".to_string());
        file2.store_message(&mut deps.storage, &HumanAddr::from("Address_A"));

        //printing length of Address B's collection should display 3
        let length = Message::len(&mut deps.storage, &HumanAddr::from("Address_B"));
        println!("Length of Address_B collection is {}\n", length);
        let B_allfiles = get_messages(&mut deps.storage, &HumanAddr::from("Address_B"));
        println!("{:?}", B_allfiles);
        
        //printing updated length of Address_A's collection should display 4 
        let updatedlength_A = Message::len(&mut deps.storage, &HumanAddr::from("Address_A"));
        println!("Length of Address_A collection is {}\n", updatedlength_A);  
        let A_allfiles = get_messages(&mut deps.storage, &HumanAddr::from("Address_A"));
        println!("{:?}", A_allfiles);
    } 
*/
