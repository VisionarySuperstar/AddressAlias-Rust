use crate::msg::{HandleMsg, InitMsg, QueryMsg, ShowResponse};
use crate::state::{load, save, Alias, AliasReadOnlyStorage, AliasStorage, Config, CONFIG_KEY};
use cosmwasm_std::{
    to_binary, Api, Env, Extern, HandleResponse, InitResponse, Querier, QueryResult, StdError,
    StdResult, Storage,
};
use secret_toolkit::utils::pad_handle_result;
use std::convert::TryFrom;

// === CONSTANTS ===
// pad handle responses and log attributes to blocks of 256 bytes to prevent
// leaking info based on response size
pub const BLOCK_SIZE: usize = 256;

// === INIT ===

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let max_alias_size = match valid_alias_size(msg.max_alias_size) {
        Some(v) => v,
        None => return Err(StdError::generic_err("Invalid max_alias_size")),
    };

    let config = Config { max_alias_size };
    save(&mut deps.storage, CONFIG_KEY, &config)?;
    Ok(InitResponse::default())
}

fn valid_alias_size(val: i32) -> Option<u16> {
    if val < 1 {
        None
    } else {
        u16::try_from(val).ok()
    }
}

// === HANDLE ===

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    let response = match msg {
        HandleMsg::Create { alias } => try_create(deps, env, alias),
        HandleMsg::Destroy { alias } => try_destroy(deps, env, alias),
    };
    pad_handle_result(response, BLOCK_SIZE)
}

fn try_create<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    alias_string: String,
) -> StdResult<HandleResponse> {
    let config: Config = load(&mut deps.storage, CONFIG_KEY)?;
    let alias_string_byte_slice: &[u8] = alias_string.as_bytes();
    if alias_string_byte_slice.len() > config.max_alias_size.into() {
        return Err(StdError::generic_err("Alias is too long."));
    }
    let mut alias_storage = AliasStorage::from_storage(&mut deps.storage);
    let alias_object: Option<Alias> = alias_storage.get_alias(&alias_string);
    if alias_object.is_none() {
        let sender_human_address = env.clone().message.sender;
        let new_alias = Alias {
            human_address: sender_human_address,
        };
        alias_storage.set_alias(alias_string_byte_slice, new_alias);
    } else {
        return Err(StdError::generic_err("Alias already exists."));
    }

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: None,
    })
}

fn try_destroy<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    alias_string: String,
) -> StdResult<HandleResponse> {
    let mut alias_storage = AliasStorage::from_storage(&mut deps.storage);
    let alias_object: Option<Alias> = alias_storage.get_alias(&alias_string);
    let alias_string_byte_slice: &[u8] = alias_string.as_bytes();
    let sender_human_address = env.clone().message.sender;

    if alias_object.is_none() {
        return Err(StdError::generic_err("Alias does not exist."));
    }
    let alias_object: Alias = alias_object.unwrap();
    if sender_human_address != alias_object.human_address {
        return Err(StdError::Unauthorized { backtrace: None });
    } else {
        alias_storage.remove_alias(alias_string_byte_slice);
    }

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: None,
    })
}

// === QUERY ===

pub fn query<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, msg: QueryMsg) -> QueryResult {
    match msg {
        QueryMsg::Show { alias } => {
            let alias_storage = AliasReadOnlyStorage::from_storage(&deps.storage);

            let alias_object: Option<Alias> = alias_storage.get_alias(&alias);
            if alias_object.is_none() {
                return Err(StdError::generic_err("Alias does not exist."));
            }
            return Ok(to_binary(&ShowResponse {
                alias: alias_object,
            })?);
        }
    }
}

///////////////////////////////////////////////////////////////////////
//////////////////////////////// Tests ////////////////////////////////
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(20, &[]);
        let msg = InitMsg {
            max_alias_size: 3333,
        };
        let env = mock_env("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = init(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn test_try_destroy() {
        let alias: &str = "nailbiter";
        let human_address = "why";
        let mut deps = mock_dependencies(20, &coins(2, "token"));
        let msg = InitMsg {
            max_alias_size: 3333,
        };
        let env = mock_env(human_address, &coins(2, "token"));
        let env_two = mock_env("user2", &coins(2, "token"));

        // Initialize contract instance
        init(&mut deps, env.clone(), msg).unwrap();
        // Create alias
        let create_alias_message = HandleMsg::Create {
            alias: alias.to_string(),
        };
        handle(&mut deps, env.clone(), create_alias_message).unwrap();
        // Query alias
        let show_response = query(
            &mut deps,
            QueryMsg::Show {
                alias: alias.to_string(),
            },
        )
        .unwrap();
        let val: ShowResponse = from_binary(&show_response).unwrap();
        assert_eq!(
            human_address.to_string(),
            val.alias.unwrap().human_address.to_string()
        );
        // Try deleting an alias that does not exist
        let destroy_alias_message = HandleMsg::Destroy {
            alias: "idonotexist".to_string(),
        };
        let res = handle(&mut deps, env.clone(), destroy_alias_message);
        assert_eq!(res.is_err(), true);
        // Try deleting an alias with a different user
        let destroy_alias_message = HandleMsg::Destroy {
            alias: alias.to_string(),
        };
        let res = handle(&mut deps, env_two, destroy_alias_message);
        assert_eq!(res.is_err(), true);
        // Destroy alias
        let destroy_alias_message = HandleMsg::Destroy {
            alias: alias.to_string(),
        };
        handle(&mut deps, env.clone(), destroy_alias_message).unwrap();
        // Query destroyed alias
        let query_response = query(
            &mut deps,
            QueryMsg::Show {
                alias: alias.to_string(),
            },
        );
        assert_eq!(query_response.is_err(), true);
    }

    #[test]
    fn test_try_create() {
        let alias: &str = "nailbiter";
        let mut deps = mock_dependencies(20, &coins(2, "token"));
        let human_address = "huma";
        let env = mock_env(human_address, &coins(2, "token"));
        let msg = InitMsg {
            max_alias_size: 3333,
        };

        // Initialize contract instance
        init(&mut deps, env.clone(), msg).unwrap();
        // Create alias
        let create_alias_message = HandleMsg::Create {
            alias: alias.to_string(),
        };
        handle(&mut deps, env.clone(), create_alias_message).unwrap();
        // Query alias
        let show_response = query(
            &mut deps,
            QueryMsg::Show {
                alias: alias.to_string(),
            },
        )
        .unwrap();
        let val: ShowResponse = from_binary(&show_response).unwrap();
        assert_eq!(human_address, val.alias.unwrap().human_address.to_string());
        // Create same alias
        let create_alias_message = HandleMsg::Create {
            alias: alias.to_string(),
        };
        assert_eq!(
            handle(&mut deps, env.clone(), create_alias_message).is_err(),
            true
        )
    }
}
