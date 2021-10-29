use crate::msg::ResponseStatus::Success;
use crate::msg::{
    AliasAttributes, HandleAnswer, HandleMsg, InitMsg, QueryAnswer, QueryMsg, SearchResponse,
};
use crate::state::{
    AddressesAliasesReadonlyStorage, AddressesAliasesStorage, Alias, AliasesReadonlyStorage,
    AliasesStorage, Config,
};
use cosmwasm_std::{
    to_binary, Api, Env, Extern, HandleResponse, InitResponse, Querier, QueryResult, StdError,
    StdResult, Storage,
};
use secret_toolkit::snip20;
use secret_toolkit::storage::{TypedStore, TypedStoreMut};

pub const BLOCK_SIZE: usize = 1;
pub const CONFIG_KEY: &[u8] = b"config";

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let mut config_store = TypedStoreMut::attach(&mut deps.storage);
    let config: Config = Config {
        buttcoin: msg.buttcoin.clone(),
    };
    config_store.store(CONFIG_KEY, &config)?;

    Ok(InitResponse {
        messages: vec![snip20::register_receive_msg(
            env.contract_code_hash.clone(),
            None,
            BLOCK_SIZE,
            config.buttcoin.contract_hash,
            config.buttcoin.address,
        )?],
        log: vec![],
    })
}

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
    let alias_string = alias_string.trim();
    let alias_string_formatted = alias_string.to_lowercase().to_string();
    let alias_string_byte_slice: &[u8] = alias_string_formatted.as_bytes();
    // Check alias size
    if alias_string_byte_slice.len() > u8::MAX.into() {
        return Err(StdError::generic_err("Alias is too long"));
    }
    // Check that Alias doesn't already exist
    let mut alias_storage = AliasesStorage::from_storage(&mut deps.storage);
    let alias_object: Option<Alias> = alias_storage.get_alias(alias_string_byte_slice);
    if alias_object.is_none() {
        let sender_human_address = env.clone().message.sender;
        let new_alias = Alias {
            avatar_url: avatar_url.clone(),
            human_address: sender_human_address.clone(),
        };
        alias_storage.set_alias(alias_string_byte_slice, new_alias);
        // Check that the user doesn't already have an alias
        let mut addresses_aliases_storage =
            AddressesAliasesStorage::from_storage(&mut deps.storage);
        let alias_key: Option<Vec<u8>> =
            addresses_aliases_storage.get_alias(&sender_human_address.to_string());
        if alias_key.is_none() {
            addresses_aliases_storage
                .set_alias(sender_human_address.0.as_bytes(), &alias_string_formatted)
        } else {
            return Err(StdError::generic_err("Address already has an alias"));
        }
    } else {
        return Err(StdError::generic_err("Alias has already been taken"));
    }

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Create {
            alias: AliasAttributes {
                alias: alias_string.to_string(),
                avatar_url: avatar_url,
                address: env.message.sender,
            },
        })?),
    })
}

fn try_destroy<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    alias_string: String,
) -> StdResult<HandleResponse> {
    let alias_string = alias_string.trim().to_lowercase();
    let alias_string_byte_slice: &[u8] = alias_string.as_bytes();
    let mut alias_storage = AliasesStorage::from_storage(&mut deps.storage);
    let alias_object: Option<Alias> = alias_storage.get_alias(alias_string_byte_slice);
    let sender_human_address = env.clone().message.sender;

    if alias_object.is_none() {
        return Err(StdError::not_found("Alias"));
    }
    let alias_object: Alias = alias_object.unwrap();
    if sender_human_address != alias_object.human_address {
        return Err(StdError::Unauthorized { backtrace: None });
    } else {
        alias_storage.remove_alias(alias_string_byte_slice);
        let mut addresses_aliases_storage =
            AddressesAliasesStorage::from_storage(&mut deps.storage);
        addresses_aliases_storage.remove_alias(sender_human_address.0.as_bytes());
    }

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Destroy { status: Success })?),
    })
}

pub fn query<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, msg: QueryMsg) -> QueryResult {
    match msg {
        QueryMsg::Config {} => query_config(deps),
        QueryMsg::Search {
            search_type,
            mut search_value,
        } => {
            let alias_object: Option<Alias>;
            let alias_attributes: AliasAttributes;
            if search_type == "address" {
                let addresses_aliases_storage =
                    AddressesAliasesReadonlyStorage::from_storage(&deps.storage);
                let alias_key = addresses_aliases_storage.get_alias(&search_value);
                if alias_key.is_none() {
                    return Err(StdError::not_found("Alias"));
                }
                search_value =
                    String::from_utf8(alias_key.clone().unwrap()).expect("Found invalid UTF-8");
            } else if search_type == "alias" {
                search_value = search_value.trim().to_lowercase();
            } else {
                return Err(StdError::parse_err(
                    "search_type",
                    "must be address or alias.",
                ));
            }

            let alias_storage = AliasesReadonlyStorage::from_storage(&deps.storage);
            alias_object = alias_storage.get_alias(search_value.as_bytes());
            if alias_object.is_none() {
                return Err(StdError::not_found("Alias"));
            }
            alias_attributes = AliasAttributes {
                alias: search_value,
                avatar_url: alias_object.clone().unwrap().avatar_url,
                address: alias_object.unwrap().human_address,
            };

            return Ok(to_binary(&SearchResponse {
                r#type: "aliases".to_string(),
                attributes: alias_attributes,
            })?);
        }
    }
}

fn query_config<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> QueryResult {
    let config: Config = TypedStore::attach(&deps.storage).load(CONFIG_KEY)?;

    to_binary(&QueryAnswer::Config {
        buttcoin: config.buttcoin,
    })
}

// === TESTS ===
#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::SecretContract;
    use cosmwasm_std::testing::*;
    use cosmwasm_std::HumanAddr;
    use cosmwasm_std::{coins, from_binary};
    use std::any::Any;

    fn extract_error_msg<T: Any>(error: StdResult<T>) -> String {
        match error {
            Ok(_response) => {
                panic!("Unexpected query answer")
            }
            Err(err) => match err {
                StdError::GenericErr { msg, .. } => msg,
                StdError::NotFound { kind, .. } => format!("{} not found", kind),
                StdError::Unauthorized { .. } => "Unauthorized".to_string(),
                _ => panic!("Unexpected result from init"),
            },
        }
    }

    //=== HELPER FUNCTIONS ===
    fn init_helper() -> (
        StdResult<InitResponse>,
        Extern<MockStorage, MockApi, MockQuerier>,
    ) {
        let mut deps = mock_dependencies(20, &[]);
        let env = mock_env(mock_user_address(), &[]);

        let init_msg = InitMsg {
            buttcoin: mock_buttcoin(),
        };

        (init(&mut deps, env, init_msg), deps)
    }

    fn mock_buttcoin() -> SecretContract {
        SecretContract {
            address: HumanAddr("buttcoin-address".to_string()),
            contract_hash: "buttcoin-contract-hash".to_string(),
        }
    }

    fn mock_user_address() -> HumanAddr {
        HumanAddr::from("some-geezer")
    }

    // === TESTS ===
    #[test]
    fn test_try_destroy() {
        let alias: &str = "nailbiter";
        let human_address = "why";
        let env = mock_env(human_address, &coins(2, "token"));
        let env_two = mock_env("user2", &coins(2, "token"));

        // Initialize
        let (_init_result, mut deps) = init_helper();
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
        let error = extract_error_msg(res);
        assert_eq!(error, "Alias not found");
        // Try deleting an alias with a different user
        let destroy_alias_message = HandleMsg::Destroy {
            alias: alias.to_string(),
        };
        let res = handle(&mut deps, env_two, destroy_alias_message);
        let error = extract_error_msg(res);
        assert_eq!(error, "Unauthorized");
        // Destroy alias
        let destroy_alias_message = HandleMsg::Destroy {
            alias: alias.to_string(),
        };
        handle(&mut deps, env.clone(), destroy_alias_message).unwrap();
        // Query destroyed alias via alias
        let query_response = query(
            &mut deps,
            QueryMsg::Search {
                search_type: "alias".to_string(),
                search_value: alias.to_string(),
            },
        );
        let error = extract_error_msg(query_response);
        assert_eq!(error, "Alias not found");
        // Query destroyed alias via address
        let query_response = query(
            &mut deps,
            QueryMsg::Search {
                search_type: "address".to_string(),
                search_value: human_address.to_string(),
            },
        );
        let error = extract_error_msg(query_response);
        assert_eq!(error, "Alias not found");
    }

    #[test]
    fn test_try_create() {
        let alias = "   nail biter    ";
        let avatar_url = "https://www.btn.group";
        let human_address = "secret34aergaerg3a4fa34g";
        let env = mock_env(human_address, &coins(2, "token"));

        // Initialize
        let (_init_result, mut deps) = init_helper();

        // Create alias
        let create_alias_message = HandleMsg::Create {
            alias: alias.to_string(),
            avatar_url: Some(avatar_url.to_string()),
        };
        handle(&mut deps, env.clone(), create_alias_message).unwrap();

        // Query alias with alias without trailing and leading whitespaces
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

        // Query alias with address
        let search_response = query(
            &mut deps,
            QueryMsg::Search {
                search_type: "address".to_string(),
                search_value: human_address.to_string(),
            },
        )
        .unwrap();
        let val: SearchResponse = from_binary(&search_response).unwrap();
        assert_eq!("nail biter", val.attributes.clone().alias.to_string());
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
        let response = handle(&mut deps, env.clone(), create_alias_message);
        let error = extract_error_msg(response);
        assert_eq!(error, "Alias has already been taken");

        // Create same alias with capitals
        let create_alias_message = HandleMsg::Create {
            alias: alias.to_uppercase().to_string(),
            avatar_url: None,
        };
        let response = handle(&mut deps, env.clone(), create_alias_message);
        let error = extract_error_msg(response);
        assert_eq!(error, "Alias has already been taken");

        // Create alias that is too long
        let alias = "Epstein didn't kill himself".repeat(20);
        let create_alias_message = HandleMsg::Create {
            alias: alias.to_string(),
            avatar_url: None,
        };
        let response = handle(&mut deps, env.clone(), create_alias_message);
        let error = extract_error_msg(response);
        assert_eq!(error, "Alias is too long");

        // Create another alias for the same user
        let alias = "PNG";
        let create_alias_message = HandleMsg::Create {
            alias: alias.to_string(),
            avatar_url: None,
        };
        let response = handle(&mut deps, env.clone(), create_alias_message);
        let error = extract_error_msg(response);
        assert_eq!(error, "Address already has an alias");
    }

    // === QUERY TESTS ===

    #[test]
    fn test_query_config() {
        let (_init_result, deps) = init_helper();
        let config: Config = TypedStore::attach(&deps.storage).load(CONFIG_KEY).unwrap();
        let query_result = query(&deps, QueryMsg::Config {}).unwrap();
        let query_answer: QueryAnswer = from_binary(&query_result).unwrap();
        match query_answer {
            QueryAnswer::Config { buttcoin } => {
                assert_eq!(buttcoin, config.buttcoin);
            }
        }
    }
}
