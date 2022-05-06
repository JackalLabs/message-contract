use crate::msg::{HandleMsg, InitMsg, MsgsResponse, QueryMsg, ViewingPermissions};
use crate::state::{config, Message, File, State, PERFIX_PERMITS, PREFIX_MSGS};
use cosmwasm_std::{
    debug_print, to_binary, Api, Binary, Env, Extern, HandleResponse, HumanAddr, InitResponse,
    Querier, StdError, StdResult, Storage, ReadonlyStorage,
};
use cosmwasm_storage::ReadonlyPrefixedStorage;
use secret_toolkit::storage::AppendStore;
pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    _msg: InitMsg,
) -> StdResult<InitResponse> {
    let state = State {
        owner: deps.api.canonical_address(&env.message.sender)?,
        contract: env.contract.address,
    };

    config(&mut deps.storage).save(&state)?;

    debug_print!("Contract was initialized by {}", env.message.sender);

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::SendFile { to, contents } => send_file(deps, env, to, contents),
        HandleMsg::SetViewingKey { key, padding } => todo!(),
        // HandleMsg::SetViewingKey { key, .. } => create_key(deps, env, key),
    }
}

// pub fn create_key<S: Storage, A: Api, Q: Querier> - going to put back later

pub fn send_file<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    to: HumanAddr,
    contents: String,
) -> StdResult<HandleResponse> {
    let file = File::new(contents, env.message.sender.to_string(), false);
    file.store_file(&mut deps.storage, &to)?;

    debug_print(format!("file stored successfully to {}", to));
    Ok(HandleResponse::default())
}

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
/* 
pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetMemo {
            address,
            //auth, not sure this is needed 
            page,
            page_size,
        } => to_binary(&query_memo(deps, address, page, page_size)?),
    }
} */

/* this version is very different from CashManey/Memo
fn query_memo<S: Storage, A: Api, Q: Querier>(
        deps: &Extern<S, A, Q>,
        address: HumanAddr,
        //auth: ViewingPermissions, not sure this is needed 
        page: Option<u32>,
        page_size: Option<u32>,
    ) -> StdResult<MsgsResponse> {
            
        let msgs = Message::get_messages(
                &deps.storage,
                &address,
                page.unwrap_or(0),
                page_size.unwrap_or(10),
            )?
            .0;
    
        let length = Message::len(&deps.storage, &address);
    
        Ok(MsgsResponse { msgs, length })
    }

*/
#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, MockStorage};
    use cosmwasm_std::{coins, from_binary, ReadonlyStorage};
    use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};

    #[test]
    fn contract_init() {
        let mut deps = mock_dependencies(20, &[]);

        let msg = InitMsg { owner: None };
        let env = mock_env("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = init(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn send_file() {
        let mut deps = mock_dependencies(20, &coins(2, "token"));
        
        let msg = InitMsg { owner: None };
        let env = mock_env("creator", &coins(2, "token"));
        let _res = init(&mut deps, env, msg).unwrap();
        
        //sending a file to contract creator's address 
        let env = mock_env("Bi", &coins(2, "token"));
        let msg = HandleMsg::SendFile {
            to: HumanAddr("creator".to_string()),
            contents: "pepe.jpg".to_string(),
        };
        let res = handle(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());

        //sending a different file to another user's address 
        let env = mock_env("Bi", &coins(2, "token"));
        let msg = HandleMsg::SendFile {
            to: HumanAddr("Alice".to_string()),
            contents: "Hasbullah.pdf".to_string(),
        };
        let res = handle(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());

        //sending a different file to another user's address 
        let env = mock_env("Bi", &coins(2, "token"));
        let msg = HandleMsg::SendFile {
            to: HumanAddr("Nuggie".to_string()),
            contents: "King_Pepe.pdf".to_string(),
        };
        let res = handle(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());

    }

    #[test]
    fn append_store_works() {
        //append.store.rs is a storage rapper that creates collections of items
        //When we call store_file and pass in a recipient address, append.store.rs will use this address as a namespace for the 
        //collection. Everytime you store a file using the same address, the file is added to that address's collection because
        //it corresponds to the same namespace.
        //calling store_file for a different address will initiate a new collection for a different namespace
        //appendstore supports iterating, pushing, and popping, retrieving files at specific indexes is possible 
        let mut deps = mock_dependencies(20, &coins(2, "token"));
            
        let msg = InitMsg { owner: None };
        let env = mock_env("creator", &coins(2, "token"));
        let _res = init(&mut deps, env, msg).unwrap();
    
        //
        let env = mock_env("Bi", &coins(2, "token"));


        //saving a file for address_A
        let file1 = File::new("King_Pepe.jpg".to_string(), env.message.sender.to_string(), false);
        file1.store_file(&mut deps.storage, &HumanAddr("address_A".to_string()));
        let file1copy = File::get_file(&mut deps.storage,&HumanAddr("address_A".to_string()), 0 ).unwrap();
        assert_eq!(file1, file1copy);

        //trying to get a file at position 2 of address_A's collection. No file exists, so calling this will cause
        //test to fail and return out of bounds error. Uncommon to see error message 

        //let ghostfile = File::get_file(&mut deps.storage,&HumanAddr("address_A".to_string()), 1 ).unwrap();
        
        //saving a file for address_B
        let file2 = File::new("Queen_Pepe.jpg".to_string(), env.message.sender.to_string(), false);
        file2.store_file(&mut deps.storage, &HumanAddr("address_B".to_string()));
        let file2copy = File::get_file(&mut deps.storage,&HumanAddr("address_B".to_string()), 0 ).unwrap();
        assert_eq!(file2, file2copy);
        //check to make sure that file1 != file2
        assert_ne!(file1, file2);

        //add another File to address_B's collection and make sure that is is in fact bigger than address_A's collection
        let file3 = File::new("Prince_Pepe.jpg".to_string(), env.message.sender.to_string(), false);
        file3.store_file(&mut deps.storage, &HumanAddr("address_B".to_string()));

        let address_A_Collection_Length = File::len(&mut deps.storage,&HumanAddr("address_A".to_string()));
        print!("size of address_A's collection is {}\n", address_A_Collection_Length);

        let address_B_Collection_Length = File::len(&mut deps.storage,&HumanAddr("address_B".to_string()));
        print!("size of address_B's collection is{}", address_B_Collection_Length)





        

        
        //let res = handle(&mut deps, env, msg).unwrap();
        //assert_eq!(0, res.messages.len());
    
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

    /*/
    #[test]
    fn send_message() {
        let mut deps = mock_dependencies(20, &coins(2, "token"));
        
        let msg = InitMsg { owner: None };
        let env = mock_env("creator", &coins(2, "token"));
        let _res = init(&mut deps, env, msg).unwrap();
        
        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg::SendMemo {
            to: HumanAddr("creator".to_string()),
            message: "hello world".to_string(),
        };
        let res = handle(&mut deps, env, msg).unwrap();
        
        assert_eq!(0, res.messages.len());
    }*/
    /*
    #[test]
    fn read_message() {
        let mut deps = mock_dependencies(20, &coins(2, "token"));

        let msg = InitMsg { owner: None };
        let env = mock_env("creator", &coins(2, "token"));
        let _res = init(&mut deps, env, msg).unwrap();

        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg::SendMemo {
            to: HumanAddr("creator".to_string()),
            message: "hello world".to_string(),
        };
        let res = handle(&mut deps, env, msg).unwrap();

        assert_eq!(0, res.messages.len());

        let res = query(
            &deps,
            QueryMsg::GetMemo {
                address: HumanAddr("creator".to_string()),
                page: None,
                page_size: None,
            },
        )
        .unwrap();
        let value: MsgsResponse = from_binary(&res).unwrap();
        println!("{:?}", &value.msgs[0]);
        assert_eq!(value.msgs.len(), 1);
        assert_eq!(value.msgs[0].message, "hello world".to_string());
    }*/
 
    // #[test]
    // fn read_message_fail() {
    //     let mut deps = mock_dependencies(20, &coins(2, "token"));

    //     let msg = InitMsg { owner: None };
    //     let env = mock_env("creator", &coins(2, "token"));
    //     let _res = init(&mut deps, env, msg).unwrap();

    //     // anyone can increment
    //     let env = mock_env("anyone", &coins(2, "token"));
    //     let msg = HandleMsg::SendMemo {
    //         to: HumanAddr("creator".to_string()),
    //         message: "hello world".to_string(),
    //     };
    //     let _res = handle(&mut deps, env, msg).unwrap();

    //     let res = query(
    //         &deps,
    //         QueryMsg::GetMemo {
    //             address: HumanAddr("creator".to_string()),
    //             auth: ViewingPermissions {
    //                 permit: None,
    //                 key: Some("yoyo".to_string()),
    //             },
    //             page: None,
    //             page_size: None,
    //         },
    //     );
    //     // let value: StdResult<MsgsResponse> = from_binary(&res);
    //     assert_eq!(res.is_err(), true);
    // }



// fn query_memo<S: Storage, A: Api, Q: Querier>(
//     deps: &Extern<S, A, Q>,
//     address: HumanAddr,
//     auth: ViewingPermissions,
//     page: Option<u32>,
//     page_size: Option<u32>,
// ) -> StdResult<MsgsResponse> {
//     let contract_address = config_read(&deps.storage).load()?.contract;

//     let hrp: String = bech32::decode(address.as_str())
//         .map_err(|_| StdError::generic_err("Permit not signed for this contract"))?
//         .0;

//     let mut msgs = vec![];

//     if let Some(key) = auth.key {
//         if validate_key(&deps.storage, key, &address) {
//             msgs = Message::get_messages(
//                 &deps.storage,
//                 &address,
//                 page.unwrap_or(0),
//                 page_size.unwrap_or(10),
//             )?
//             .0;
//         } else {
//             return Err(StdError::unauthorized());
//         }
//     } else if let Some(permit) = auth.permit {
//         if !permit.check_token(&contract_address) {
//             return Err(StdError::generic_err("Permit not signed for this contract"));
//         }

//         if !permit.check_permission(&Permission::History)
//             && !permit.check_permission(&Permission::Owner)
//         {
//             return Err(StdError::generic_err(
//                 "Permit does not have correct permissions",
//             ));
//         }

//         if validate(deps, PERFIX_PERMITS, &permit, contract_address, Some(&hrp))? != address.0 {
//             return Err(StdError::generic_err("Permit invalid"));
//         }

//         msgs = Message::get_messages(
//             &deps.storage,
//             &address,
//             page.unwrap_or(0),
//             page_size.unwrap_or(10),
//         )?
//         .0;
//     }

//     let length = Message::len(&deps.storage, &address);

//     Ok(MsgsResponse { msgs, length })
// }

// #[test]
// fn multi_level() {
//     let mut storage = MockStorage::new();

//     // set with nested
//     let mut foo = PrefixedStorage::new(b"foo", &mut storage);
//     let mut bar = PrefixedStorage::new(b"bar", &mut foo);
//     bar.set(b"baz", b"winner");

//     // we can nest them the same encoding with one operation
//     let loader = ReadonlyPrefixedStorage::multilevel(&[b"foo", b"bar"], &storage);
//     assert_eq!(loader.get(b"baz"), Some(b"winner".to_vec()));
//     println!("testing multilevel");

//     // set with multilevel
//     let mut foobar = PrefixedStorage::multilevel(&[b"foo", b"bar"], &mut storage);
//     foobar.set(b"second", b"time");

//     let a = ReadonlyPrefixedStorage::new(b"foo", &storage);
//     let b = ReadonlyPrefixedStorage::new(b"bar", &a);
//     assert_eq!(b.get(b"second"), Some(b"time".to_vec()));


// 