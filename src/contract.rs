use crate::msg::{HandleMsg, InitMsg, MsgsResponse, QueryMsg, ViewingPermissions};
use crate::state::{config, Message, State, PERFIX_PERMITS};
use cosmwasm_std::{
    debug_print, to_binary, Api, Binary, Env, Extern, HandleResponse, HumanAddr, InitResponse,
    Querier, StdError, StdResult, Storage,
};
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
        HandleMsg::SendMemo { to, message } => send_memo(deps, env, to, message),
        HandleMsg::SetViewingKey { key, padding } => todo!(),
        // HandleMsg::SetViewingKey { key, .. } => create_key(deps, env, key),
    }
}

// pub fn create_key<S: Storage, A: Api, Q: Querier>(
//     deps: &mut Extern<S, A, Q>,
//     env: Env,
//     key: String,
// ) -> StdResult<HandleResponse> {
//     store_key(&mut deps.storage, key, &env.message.sender);

//     debug_print(format!(
//         "key stored successfully for {}",
//         env.message.sender
//     ));
//     Ok(HandleResponse::default())
// }

pub fn send_memo<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    to: HumanAddr,
    message: String,
) -> StdResult<HandleResponse> {
    let msg = Message::new(env.message.sender.clone(), message, env.block.time);
    msg.store_message(&mut deps.storage, &to)?;

    debug_print(format!("message stored successfully to {}", to));
    Ok(HandleResponse::default())
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetMemo {
            address,
            page,
            page_size,
        } => to_binary(&query_memo(deps, address, page, page_size)?),
    }
}

fn query_memo<S: Storage, A: Api, Q: Querier>(
        deps: &Extern<S, A, Q>,
        address: HumanAddr,
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

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use cosmwasm_std::{coins, from_binary};

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
    }
    
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
    }

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
}
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