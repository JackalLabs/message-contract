use std::convert::TryInto;

use crate::msg::{HandleMsg, InitMsg, fileResponse, QueryMsg, ViewingPermissions};
use crate::state::{config, append_file, create_empty_collection, File, State, /*PERFIX_PERMITS*/ PREFIX_MSGS};
use crate::backend::{try_init, get_files};

use cosmwasm_std::{
    debug_print, to_binary, Api, Binary, Env, Extern, HandleResponse, HumanAddr, InitResponse,
    Querier, StdError, StdResult, Storage, ReadonlyStorage,
};

use cosmwasm_storage::{ReadonlyPrefixedStorage, PrefixedStorage};
use secret_toolkit::storage::{AppendStore, AppendStoreMut};

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
        HandleMsg::InitAddress {} => try_init(deps, env)
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

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetFiles {
            address,
        } => to_binary(&query_files(deps, address)?),
    }
} 

fn query_files<S: Storage, A: Api, Q: Querier>(
        deps: &Extern<S, A, Q>,
        address: HumanAddr,
    ) -> StdResult<fileResponse> {
            
        let files = get_files(
                &deps.storage,
                &address,
            )?
            .0;
    
        let length = File::len(&deps.storage, &address);   
        Ok(fileResponse { files, length })
    }


#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, MockStorage};
    use cosmwasm_std::{coins, from_binary, ReadonlyStorage};
    use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
    use crate::msg::{fileResponse, HandleAnswer, self, /*WalletInfoResponse*/};

    fn init_for_test<S: Storage, A: Api, Q: Querier> (
        deps: &mut Extern<S, A, Q>,
        address:String,
    ) /*-> ViewingKey*/ {

        // Init Contract
        let msg = InitMsg {owner: None};
        let env = mock_env("creator", &[]);
        let _res = init(deps, env, msg).unwrap();

        // Init Address and Create ViewingKey
        let env = mock_env(String::from(&address), &[]);
        let msg = HandleMsg::InitAddress {};
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

        let msg = InitMsg {owner: None};
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

        //sending a file to Address_A's collection - not this is different testing environment so address_A here is not one and the same as in test below 
        let env = mock_env("Bi", &coins(2, "token"));
        let msg = HandleMsg::SendFile {
            to: HumanAddr("Address_A".to_string()),
            contents: "Abdul.pdf".to_string(),
        };
        let res = handle(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());

        //sending another file to address_A's collection
        let env = mock_env("Bi", &coins(2, "token"));
        let msg = HandleMsg::SendFile {
            to: HumanAddr("Address_A".to_string()),
            contents: "Khabib.pdf".to_string(),
        };
        let res = handle(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());

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
        let msg = InitMsg { owner: None };
        let env = mock_env("creator", &coins(2, "token"));
        let _res = init(&mut deps, env, msg).unwrap();
    
        //initializing Appendstore empty collection for Address_A 
        let env = mock_env("Address_A", &coins(2, "token"));
        try_init(&mut deps, env);

        //saving a file for Address_A's collection
        let file = File::new("King_Pepe.jpg".to_string(), "anyone".to_string(), false);
        file.store_file(&mut deps.storage, &HumanAddr::from("Address_A"));

        let file2 = File::new("Queen_Pepe.jpg".to_string(), "anyone".to_string(), false);
        file2.store_file(&mut deps.storage, &HumanAddr::from("Address_A"));

        //printing length of collection should display 3
        let length = File::len(&mut deps.storage, &HumanAddr::from("Address_A"));
        println!("Length of Address_A collection is {}\n", length);
        let A_allfiles = get_files(&mut deps.storage, &HumanAddr::from("Address_A"));
        println!("{:?}", A_allfiles);
        //let ha2 = deps.api.human_address(&deps.api.canonical_address(&env.message.sender).unwrap());
        //note: if human address is 2 characters or fewer, unwrapping fails 

        //initializing Appendstore empty collection for Address_B 
        let env = mock_env("Address_B", &coins(2, "token"));
        try_init(&mut deps, env);

        //saving a file for Address_B's collection
        let file = File::new("B_Coin.jpg".to_string(), "anyone".to_string(), false);
        file.store_file(&mut deps.storage, &HumanAddr::from("Address_B"));

        let file2 = File::new("C_Coin.jpg".to_string(), "anyone".to_string(), false);
        file2.store_file(&mut deps.storage, &HumanAddr::from("Address_B"));

        //printing length of collection should display 3
        let length = File::len(&mut deps.storage, &HumanAddr::from("Address_B"));
        println!("Length of Address_B collection is {}\n", length);
        let B_allfiles = get_files(&mut deps.storage, &HumanAddr::from("Address_B"));
        println!("{:?}", B_allfiles)
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