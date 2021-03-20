use cosmwasm_std::{
    Api, Env, Extern, InitResponse, Querier, StdError,
    StdResult, Storage,
};

use crate::msg::{InitMsg};
// use crate::msg::ResponseStatus::{Success, Failure};
use crate::state::{save, Config, CONFIG_KEY};
use std::convert::TryFrom;

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

// === PRIVATE ===
fn valid_alias_size(val: i32) -> Option<u16> {
  if val < 1 {
    None
  } else {
    u16::try_from(val).ok()
  }
}

// pub fn handle<S: Storage, A: Api, Q: Querier>(
//     deps: &mut Extern<S, A, Q>,
//     env: Env,
//     msg: HandleMsg,
// ) -> StdResult<HandleResponse> {
//     match msg {
//         HandleMsg::Increment {} => try_increment(deps, env),
//         HandleMsg::Reset { count } => try_reset(deps, env, count),
//     }
// }

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

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use cosmwasm_std::testing::{mock_dependencies, mock_env};
//     use cosmwasm_std::{coins, from_binary, StdError};

//     #[test]
//     fn proper_initialization() {
//         let mut deps = mock_dependencies(20, &[]);

//         let msg = InitMsg { count: 17 };
//         let env = mock_env("creator", &coins(1000, "earth"));

//         // we can just call .unwrap() to assert this was a success
//         let res = init(&mut deps, env, msg).unwrap();
//         assert_eq!(0, res.messages.len());

//         // it worked, let's query the state
//         let res = query(&deps, QueryMsg::GetCount {}).unwrap();
//         let value: CountResponse = from_binary(&res).unwrap();
//         assert_eq!(17, value.count);
//     }

//     #[test]
//     fn increment() {
//         let mut deps = mock_dependencies(20, &coins(2, "token"));

//         let msg = InitMsg { count: 17 };
//         let env = mock_env("creator", &coins(2, "token"));
//         let _res = init(&mut deps, env, msg).unwrap();

//         // anyone can increment
//         let env = mock_env("anyone", &coins(2, "token"));
//         let msg = HandleMsg::Increment {};
//         let _res = handle(&mut deps, env, msg).unwrap();

//         // should increase counter by 1
//         let res = query(&deps, QueryMsg::GetCount {}).unwrap();
//         let value: CountResponse = from_binary(&res).unwrap();
//         assert_eq!(18, value.count);
//     }

//     #[test]
//     fn reset() {
//         let mut deps = mock_dependencies(20, &coins(2, "token"));

//         let msg = InitMsg { count: 17 };
//         let env = mock_env("creator", &coins(2, "token"));
//         let _res = init(&mut deps, env, msg).unwrap();

//         // not anyone can reset
//         let unauth_env = mock_env("anyone", &coins(2, "token"));
//         let msg = HandleMsg::Reset { count: 5 };
//         let res = handle(&mut deps, unauth_env, msg);
//         match res {
//             Err(StdError::Unauthorized { .. }) => {}
//             _ => panic!("Must return unauthorized error"),
//         }

//         // only the original creator can reset the counter
//         let auth_env = mock_env("creator", &coins(2, "token"));
//         let msg = HandleMsg::Reset { count: 5 };
//         let _res = handle(&mut deps, auth_env, msg).unwrap();

//         // should now be 5
//         let res = query(&deps, QueryMsg::GetCount {}).unwrap();
//         let value: CountResponse = from_binary(&res).unwrap();
//         assert_eq!(5, value.count);
//     }

