use cosmwasm_std::{to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier, StdError,
    StdResult, Storage};

use crate::msg::{HandleAnswer, HandleMsg, InitMsg, QueryMsg, ResponseStatus, ShowResponse};
use crate::msg::ResponseStatus::{Success, Failure};
use crate::state::{load, save, Alias, AliasStorage, Config, CONFIG_KEY};
use std::convert::TryFrom;
use secret_toolkit::utils::{pad_handle_result};

// === CONSTANTS ===
// pad handle responses and log attributes to blocks of 256 bytes to prevent
// leaking info based on response size
pub const BLOCK_SIZE: usize = 256;

// === FUNCTIONS ===
pub fn init<S: Storage, A: Api, Q: Querier>(
  deps: &mut Extern<S, A, Q>,
  _env: Env,
  msg: InitMsg,
) -> StdResult<InitResponse> {
  let max_alias_size = match valid_alias_size(msg.max_alias_size) {
    Some(v) => v,
    None => return Err(StdError::generic_err("Invalid max_alias_size"))
  };

  let config = Config {
    max_alias_size
  };
  save(&mut deps.storage, CONFIG_KEY, &config)?;
  Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    // match msg {
    //     HandleMsg::Increment {} => try_increment(deps, env),
    //     HandleMsg::Reset { count } => try_reset(deps, env, count),
    // }
    let response = match msg {
      HandleMsg::Create { alias_string } => try_create(deps, env, alias_string)
    };
    pad_handle_result(response, BLOCK_SIZE)
}

pub fn query<S: Storage, A: Api, Q: Querier>(
  deps: &mut Extern<S, A, Q>,
  msg: QueryMsg,
) -> StdResult<Binary> {
  match msg {
    QueryMsg::Show { alias_string } => to_binary(&query_alias(deps, alias_string)?),
  }
}

// pub fn query<S: Storage, A: Api, Q: Querier>(
//     deps: &Extern<S, A, Q>,
//     msg: QueryMsg,
// ) -> StdResult<Binary> {
//     match msg {
//         QueryMsg::GetCount {} => to_binary(&query_count(deps)?),
//     }
// }

// fn query_count<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<CountResponse> {
//     let state = config_read(&deps.storage).load()?;
//     Ok(CountResponse { count: state.count })
// }

// === PRIVATE ===
fn query_alias<S: Storage, A: Api, Q: Querier>(deps: &mut Extern<S, A, Q>, alias_string: String) -> StdResult<ShowResponse> {
  let mut alias_storage = AliasStorage::from_storage(&mut deps.storage);
  let alias_object: Option<Alias> = alias_storage.get_alias(&alias_string);
  Ok(ShowResponse { alias: alias_object })
}

fn try_create<S: Storage, A: Api, Q: Querier>(deps: &mut Extern<S, A, Q>, env: Env, alias_string: String) -> StdResult<HandleResponse> {
  let status: ResponseStatus;
  let mut response_message = String::new();
  let config: Config = load(&mut deps.storage, CONFIG_KEY)?;
  let alias_string_byte_slice: &[u8] = alias_string.as_bytes();
  
  if alias_string_byte_slice.len() > config.max_alias_size.into() {
    status = Failure;
    response_message.push_str(&format!("Message is too long."));
  } else {
    let sender_human_address = env.message.sender;
    let mut alias_storage = AliasStorage::from_storage(&mut deps.storage);
    let new_alias = Alias {
      owner: sender_human_address,
    };
    alias_storage.set_alias(alias_string_byte_slice, new_alias);
    status = Success;
    response_message.push_str(&format!("Alias created"));
  }

  Ok(HandleResponse {
    messages: vec![],
    log: vec![],
    data: Some(to_binary(&HandleAnswer::Create {
      status,
      message: response_message,
    })?),
  })
}

fn valid_alias_size(val: i32) -> Option<u16> {
  if val < 1 {
    None
  } else {
    u16::try_from(val).ok()
  }
}

// pub fn try_reset<S: Storage, A: Api, Q: Querier>(
//     deps: &mut Extern<S, A, Q>,
//     env: Env,
//     count: i32,
// ) -> StdResult<HandleResponse> {
//     let sender_address_raw = deps.api.canonical_address(&env.message.sender)?;
//     config(&mut deps.storage).update(|mut state| {
//         if sender_address_raw != state.owner {
//             return Err(StdError::Unauthorized { backtrace: None });
//         }
//         state.count = count;
//         Ok(state)
//     })?;
//     Ok(HandleResponse::default())
// }

#[cfg(test)]
mod tests {
  use super::*;
  use cosmwasm_std::testing::{mock_dependencies, mock_env};
  use cosmwasm_std::{coins, from_binary};

  #[test]
  fn proper_initialization() {
    let mut deps = mock_dependencies(20, &[]);
    let msg = InitMsg { max_alias_size: 3333 };
    let env = mock_env("creator", &coins(1000, "earth"));
    // we can just call .unwrap() to assert this was a success
    let res = init(&mut deps, env, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    // let res = query(&deps, QueryMsg::GetCount {}).unwrap();
    // let value: CountResponse = from_binary(&res).unwrap();
    // assert_eq!(17, value.count);
  }

    #[test]
    fn create() {
      let mut deps = mock_dependencies(20, &coins(2, "token"));

      let msg = InitMsg { max_alias_size: 3333 };
      let env = mock_env("creator", &coins(2, "token"));
      let _res = init(&mut deps, env, msg).unwrap();
      let _msg = HandleMsg::Create { alias_string: "alex".to_string() };
      let unauth_env = mock_env("anyone", &coins(2, "token"));
      handle(&mut deps, unauth_env, _msg).unwrap();
      let query_response = query(&mut deps, QueryMsg::Show { alias_string: "alex".to_string() }).unwrap();
      let val: ShowResponse = from_binary(&query_response).unwrap();
      assert_eq!("anyone".to_string(), val.alias.unwrap().owner.to_string());
    }

    // #[test]
    // fn reset() {
    //     let mut deps = mock_dependencies(20, &coins(2, "token"));

    //     let msg = InitMsg { count: 17 };
    //     let env = mock_env("creator", &coins(2, "token"));
    //     let _res = init(&mut deps, env, msg).unwrap();

    //     // not anyone can reset
    //     let unauth_env = mock_env("anyone", &coins(2, "token"));
    //     let msg = HandleMsg::Reset { count: 5 };
    //     let res = handle(&mut deps, unauth_env, msg);
    //     match res {
    //         Err(StdError::Unauthorized { .. }) => {}
    //         _ => panic!("Must return unauthorized error"),
    //     }

    //     // only the original creator can reset the counter
    //     let auth_env = mock_env("creator", &coins(2, "token"));
    //     let msg = HandleMsg::Reset { count: 5 };
    //     let _res = handle(&mut deps, auth_env, msg).unwrap();

    //     // should now be 5
    //     let res = query(&deps, QueryMsg::GetCount {}).unwrap();
    //     let value: CountResponse = from_binary(&res).unwrap();
    //     assert_eq!(5, value.count);
}

