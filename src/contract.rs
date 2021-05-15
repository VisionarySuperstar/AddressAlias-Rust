use crate::msg::AliasAttributes;
use crate::msg::{HandleMsg, QueryMsg, SearchResponse};
use crate::state::{
    AddressesAliasesReadonlyStorage, Alias, AliasesReadonlyStorage, AliasesStorage,
};
use cosmwasm_std::{
    to_binary, Api, Env, Extern, HandleResponse, InitResponse, Querier, QueryResult, StdError,
    StdResult, Storage,
};

// === INIT ===
pub fn init<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
) -> StdResult<InitResponse> {
    Ok(InitResponse::default())
}

// === HANDLE ===
pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    let response = match msg {
        HandleMsg::Create { alias, avatar_url } => try_create(deps, env, alias, avatar_url),
        HandleMsg::Destroy { alias } => try_destroy(deps, env, alias),
    };
    // No need to pad response as all info is public
    response
}

fn try_create<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    alias_string: String,
    avatar_url: Option<String>,
) -> StdResult<HandleResponse> {
    let alias_string = alias_string.trim().to_string();
    let alias_string_byte_slice: &[u8] = alias_string.as_bytes();
    if alias_string_byte_slice.len() > u16::MAX.into() {
        return Err(StdError::generic_err("Alias is too long."));
    }
    let mut alias_storage = AliasesStorage::from_storage(&mut deps.storage);
    let alias_object: Option<Alias> = alias_storage.get_alias(&alias_string);
    if alias_object.is_none() {
        let sender_human_address = env.clone().message.sender;
        let new_alias = Alias {
            avatar_url: avatar_url,
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
    let mut alias_storage = AliasesStorage::from_storage(&mut deps.storage);
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
        QueryMsg::Search {
            search_type,
            search_value,
        } => {
            let alias_object: Option<Alias>;
            let alias_attributes: AliasAttributes;
            if search_type == "address" {
                let addresses_aliases_storage =
                    AddressesAliasesReadonlyStorage::from_storage(&deps.storage);
                let alias_key = addresses_aliases_storage.get_alias(&search_value);
                if alias_key.is_none() {
                    return Err(StdError::generic_err("Alias does not exist."));
                }
                let alias_storage = AliasesReadonlyStorage::from_storage(&deps.storage);
                alias_object = alias_storage.get_alias(&search_value);
                if alias_object.is_none() {
                    return Err(StdError::generic_err("Alias does not exist."));
                }
                alias_attributes = AliasAttributes {
                    alias: search_value,
                    avatar_url: alias_object.clone().unwrap().avatar_url,
                    address: alias_object.unwrap().human_address,
                };
            } else if search_type == "alias" {
                let alias_storage = AliasesReadonlyStorage::from_storage(&deps.storage);
                alias_object = alias_storage.get_alias(&search_value);
                if alias_object.is_none() {
                    return Err(StdError::generic_err("Alias does not exist."));
                }
                alias_attributes = AliasAttributes {
                    alias: search_value,
                    avatar_url: alias_object.clone().unwrap().avatar_url,
                    address: alias_object.unwrap().human_address,
                };
            } else {
                return Err(StdError::parse_err(
                    "search_type",
                    "must be address or alias.",
                ));
            }

            return Ok(to_binary(&SearchResponse {
                r#type: "aliases".to_string(),
                attributes: alias_attributes,
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
        let env = mock_env("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = init(&mut deps, env).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn test_try_destroy() {
        let alias: &str = "nailbiter";
        let human_address = "why";
        let mut deps = mock_dependencies(20, &coins(2, "token"));
        let env = mock_env(human_address, &coins(2, "token"));
        let env_two = mock_env("user2", &coins(2, "token"));

        // Initialize contract instance
        init(&mut deps, env.clone()).unwrap();
        // Create alias
        let create_alias_message = HandleMsg::Create {
            alias: alias.to_string(),
            avatar_url: None,
        };
        handle(&mut deps, env.clone(), create_alias_message).unwrap();
        // Query alias
        let search_response = query(
            &mut deps,
            QueryMsg::Search {
                search_type: "alias".to_string(),
                search_value: alias.to_string(),
            },
        )
        .unwrap();
        let val: SearchResponse = from_binary(&search_response).unwrap();
        assert_eq!(
            human_address.to_string(),
            val.attributes.address.to_string()
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
            QueryMsg::Search {
                search_type: "alias".to_string(),
                search_value: alias.to_string(),
            },
        );
        assert_eq!(query_response.is_err(), true);
    }

    #[test]
    fn test_try_create() {
        let alias = "   nail biter    ";
        let avatar_url = "https://www.btn.group";
        let mut deps = mock_dependencies(20, &coins(2, "token"));
        let human_address = "huma";
        let env = mock_env(human_address, &coins(2, "token"));

        // Initialize contract instance
        init(&mut deps, env.clone()).unwrap();

        // Create alias
        let create_alias_message = HandleMsg::Create {
            alias: alias.to_string(),
            avatar_url: Some(avatar_url.to_string()),
        };
        handle(&mut deps, env.clone(), create_alias_message).unwrap();

        // Query alias but with trailing and leading whitespaces
        let search_response = query(
            &mut deps,
            QueryMsg::Search {
                search_type: "alias".to_string(),
                search_value: "nail biter".to_string(),
            },
        )
        .unwrap();
        let val: SearchResponse = from_binary(&search_response).unwrap();
        assert_eq!(human_address, val.attributes.clone().address.to_string());
        assert_eq!(
            avatar_url,
            val.attributes.clone().avatar_url.unwrap().to_string()
        );

        // Create same alias
        let create_alias_message = HandleMsg::Create {
            alias: alias.to_string(),
            avatar_url: None,
        };
        assert_eq!(
            handle(&mut deps, env.clone(), create_alias_message).is_err(),
            true
        );

        // Create alias that is too long
        let alias = "Epstein didn't kill himself".repeat(10000);
        let create_alias_message = HandleMsg::Create {
            alias: alias.to_string(),
            avatar_url: None,
        };
        assert_eq!(
            handle(&mut deps, env.clone(), create_alias_message).is_err(),
            true
        );
    }
}
