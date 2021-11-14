use crate::authorize::authorize;
use crate::msg::ResponseStatus::Success;
use crate::msg::{
    AliasAttributes, HandleAnswer, HandleMsg, InitMsg, QueryAnswer, QueryMsg, ReceiveAnswer,
    ReceiveMsg, SearchResponse,
};
use crate::state::{
    AddressesAliasesReadonlyStorage, AddressesAliasesStorage, Alias, AliasesReadonlyStorage,
    AliasesStorage, Config,
};
use cosmwasm_std::{
    from_binary, to_binary, Api, Binary, Env, Extern, HandleResponse, HumanAddr, InitResponse,
    Querier, QueryResult, StdError, StdResult, Storage, Uint128,
};
use secret_toolkit::snip20;
use secret_toolkit::storage::{TypedStore, TypedStoreMut};

pub const AMOUNT_FOR_TRANSACTION: u128 = 1_000_000;
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
        butt_lode: msg.butt_lode,
    };
    config_store.store(CONFIG_KEY, &config)?;
    for alias_attributes in msg.aliases {
        let mut alias_storage = AliasesStorage::from_storage(&mut deps.storage);
        let new_alias: Alias = Alias {
            avatar_url: alias_attributes.avatar_url,
            human_address: alias_attributes.address.clone(),
        };
        let alias_string_byte_slice: &[u8] = alias_attributes.alias.as_bytes();
        alias_storage.set_alias(alias_string_byte_slice, new_alias);
        // Check that the user doesn't already have an alias
        let mut addresses_aliases_storage =
            AddressesAliasesStorage::from_storage(&mut deps.storage);
        addresses_aliases_storage.set_alias(
            alias_attributes.address.0.as_bytes(),
            &alias_attributes.alias,
        )
    }

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
    match msg {
        HandleMsg::Destroy { alias } => try_destroy(deps, env, alias),
        HandleMsg::Receive {
            from, amount, msg, ..
        } => receive(deps, env, from, amount, msg),
    }
}

fn receive<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    from: HumanAddr,
    amount: Uint128,
    msg: Binary,
) -> StdResult<HandleResponse> {
    let config: Config = TypedStore::attach(&deps.storage).load(CONFIG_KEY)?;
    // Ensure that the sent tokens are Buttcoins
    authorize(config.buttcoin.address, env.message.sender)?;

    // Ensure that amount sent in is 1 Buttcoin
    if amount != Uint128(AMOUNT_FOR_TRANSACTION) {
        return Err(StdError::generic_err(format!(
            "Amount sent in: {}. Amount required {}.",
            amount,
            Uint128(AMOUNT_FOR_TRANSACTION)
        )));
    }

    let msg: ReceiveMsg = from_binary(&msg)?;
    match msg {
        ReceiveMsg::Create { alias, avatar_url } => try_create(deps, from, alias, avatar_url),
    }
}

fn try_create<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    from: HumanAddr,
    alias_string: String,
    avatar_url: Option<String>,
) -> StdResult<HandleResponse> {
    let config: Config = TypedStore::attach(&deps.storage).load(CONFIG_KEY)?;
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
        let new_alias = Alias {
            avatar_url: avatar_url,
            human_address: from.clone(),
        };
        alias_storage.set_alias(alias_string_byte_slice, new_alias);
        // Check that the user doesn't already have an alias
        let mut addresses_aliases_storage =
            AddressesAliasesStorage::from_storage(&mut deps.storage);
        let alias_key: Option<Vec<u8>> = addresses_aliases_storage.get_alias(&from.to_string());
        if alias_key.is_none() {
            addresses_aliases_storage.set_alias(from.0.as_bytes(), &alias_string_formatted)
        } else {
            return Err(StdError::generic_err("Address already has an alias"));
        }
    } else {
        return Err(StdError::generic_err("Alias has already been taken"));
    }

    Ok(HandleResponse {
        messages: vec![snip20::transfer_msg(
            config.butt_lode.address,
            Uint128(AMOUNT_FOR_TRANSACTION),
            None,
            BLOCK_SIZE,
            config.buttcoin.contract_hash,
            config.buttcoin.address,
        )?],
        log: vec![],
        data: Some(to_binary(&ReceiveAnswer::Create { status: Success })?),
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
    if alias_object.is_none() {
        return Err(StdError::not_found("Alias"));
    }
    let alias_object: Alias = alias_object.unwrap();
    authorize(env.message.sender.clone(), alias_object.human_address)?;

    alias_storage.remove_alias(alias_string_byte_slice);
    let mut addresses_aliases_storage = AddressesAliasesStorage::from_storage(&mut deps.storage);
    addresses_aliases_storage.remove_alias(env.message.sender.0.as_bytes());

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
        butt_lode: config.butt_lode,
    })
}

// === TESTS ===
#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::SecretContract;
    use cosmwasm_std::from_binary;
    use cosmwasm_std::testing::*;
    use cosmwasm_std::HumanAddr;
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
            aliases: vec![AliasAttributes {
                alias: "epstein didn't kill himself".to_string(),
                address: HumanAddr::from("frump"),
                avatar_url: None,
            }],
            buttcoin: mock_buttcoin(),
            butt_lode: mock_butt_lode(),
        };

        (init(&mut deps, env, init_msg), deps)
    }

    fn mock_buttcoin() -> SecretContract {
        SecretContract {
            address: HumanAddr("buttcoin-address".to_string()),
            contract_hash: "buttcoin-contract-hash".to_string(),
        }
    }

    fn mock_butt_lode() -> SecretContract {
        SecretContract {
            address: HumanAddr("profit-sharing-contract-address".to_string()),
            contract_hash: "profit-sharing-contract-hash".to_string(),
        }
    }

    fn mock_user_address() -> HumanAddr {
        HumanAddr::from("some-geezer")
    }

    // === TESTS ===
    #[test]
    fn test_try_destroy() {
        let alias: &str = "nailbiter";

        // Initialize
        let (_init_result, mut deps) = init_helper();

        // when alias exists
        let create_alias_message = ReceiveMsg::Create {
            alias: alias.to_string(),
            avatar_url: None,
        };
        let receive_msg = HandleMsg::Receive {
            sender: mock_user_address(),
            from: mock_user_address(),
            amount: Uint128(AMOUNT_FOR_TRANSACTION),
            msg: to_binary(&create_alias_message).unwrap(),
        };
        let handle_result = handle(
            &mut deps,
            mock_env(mock_buttcoin().address, &[]),
            receive_msg.clone(),
        );
        handle_result.unwrap();

        // = when user tries to delete an alias that does not exist
        // = * it raises an error
        let destroy_alias_message = HandleMsg::Destroy {
            alias: "idonotexist".to_string(),
        };
        let res = handle(
            &mut deps,
            mock_env(mock_user_address(), &[]),
            destroy_alias_message,
        );
        let error = extract_error_msg(res);
        assert_eq!(error, "Alias not found");

        // = when user tries to delete an alias that belongs to someone else
        // = * it raises an error
        let destroy_alias_message = HandleMsg::Destroy {
            alias: alias.to_string(),
        };
        let res = handle(
            &mut deps,
            mock_env(mock_buttcoin().address, &[]),
            destroy_alias_message,
        );
        let error = extract_error_msg(res);
        assert_eq!(error, "Unauthorized");

        // = when user destroy their own alias
        // = * alias is removed
        let destroy_alias_message = HandleMsg::Destroy {
            alias: alias.to_string(),
        };
        handle(
            &mut deps,
            mock_env(mock_user_address(), &[]),
            destroy_alias_message,
        )
        .unwrap();
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
                search_value: mock_user_address().to_string(),
            },
        );
        let error = extract_error_msg(query_response);
        assert_eq!(error, "Alias not found");
    }

    #[test]
    fn test_try_create() {
        let alias = "   nail biter    ";
        let avatar_url = "https://www.btn.group";

        // Initialize
        let (_init_result, mut deps) = init_helper();

        // when creating an alias
        let create_alias_message = ReceiveMsg::Create {
            alias: alias.to_string(),
            avatar_url: Some(avatar_url.to_string()),
        };
        let receive_msg = HandleMsg::Receive {
            sender: mock_user_address(),
            from: mock_user_address(),
            amount: Uint128(AMOUNT_FOR_TRANSACTION),
            msg: to_binary(&create_alias_message).unwrap(),
        };

        // = when user sends in a token that is not buttcoin
        // = * it raises an error
        let handle_result = handle(
            &mut deps,
            mock_env(mock_user_address(), &[]),
            receive_msg.clone(),
        );
        assert_eq!(
            handle_result.unwrap_err(),
            StdError::Unauthorized { backtrace: None }
        );

        // = when user sends in buttcoin
        let handle_result = handle(
            &mut deps,
            mock_env(mock_buttcoin().address, &[]),
            receive_msg.clone(),
        );
        let handle_result_unwrapped = handle_result.unwrap();

        // = * it sends the BUTT to the BUTT lode
        assert_eq!(
            handle_result_unwrapped.messages,
            vec![snip20::transfer_msg(
                mock_butt_lode().address,
                Uint128(AMOUNT_FOR_TRANSACTION),
                None,
                BLOCK_SIZE,
                mock_buttcoin().contract_hash,
                mock_buttcoin().address,
            )
            .unwrap()],
        );

        // = * It creates alias without trailing and leading whitespaces
        let search_response = query(
            &mut deps,
            QueryMsg::Search {
                search_type: "alias".to_string(),
                search_value: "nail biter".to_string(),
            },
        )
        .unwrap();
        let val: SearchResponse = from_binary(&search_response).unwrap();
        assert_eq!(mock_user_address(), val.attributes.clone().address);
        assert_eq!(
            avatar_url,
            val.attributes.clone().avatar_url.unwrap().to_string()
        );
        let search_response = query(
            &mut deps,
            QueryMsg::Search {
                search_type: "address".to_string(),
                search_value: mock_user_address().to_string(),
            },
        )
        .unwrap();
        let val: SearchResponse = from_binary(&search_response).unwrap();
        assert_eq!("nail biter", val.attributes.clone().alias.to_string());
        assert_eq!(mock_user_address(), val.attributes.clone().address);
        assert_eq!(
            avatar_url,
            val.attributes.clone().avatar_url.unwrap().to_string()
        );

        // = when alias already exists
        let create_alias_message_two = ReceiveMsg::Create {
            alias: "Epstein didn't kill himself".to_string(),
            avatar_url: Some(avatar_url.to_string()),
        };
        let receive_msg_two = HandleMsg::Receive {
            sender: HumanAddr::from("crump"),
            from: HumanAddr::from("crump"),
            amount: Uint128(AMOUNT_FOR_TRANSACTION),
            msg: to_binary(&create_alias_message_two).unwrap(),
        };
        let handle_result = handle(
            &mut deps,
            mock_env(mock_buttcoin().address, &[]),
            receive_msg_two,
        );

        // = * it raises an error
        let error = extract_error_msg(handle_result);
        assert_eq!(error, "Alias has already been taken");

        // = when alias exists but is a different case
        let create_alias_message = ReceiveMsg::Create {
            alias: alias.to_uppercase().to_string(),
            avatar_url: None,
        };
        let receive_msg = HandleMsg::Receive {
            sender: mock_user_address(),
            from: mock_user_address(),
            amount: Uint128(AMOUNT_FOR_TRANSACTION),
            msg: to_binary(&create_alias_message).unwrap(),
        };

        // = * it raises an error
        let response = handle(
            &mut deps,
            mock_env(mock_buttcoin().address, &[]),
            receive_msg,
        );
        let error = extract_error_msg(response);
        assert_eq!(error, "Alias has already been taken");

        // = when alias is too long
        let alias = "Epstein didn't kill himself".repeat(20);
        let create_alias_message = ReceiveMsg::Create {
            alias: alias.to_uppercase().to_string(),
            avatar_url: None,
        };
        let receive_msg = HandleMsg::Receive {
            sender: mock_user_address(),
            from: mock_user_address(),
            amount: Uint128(AMOUNT_FOR_TRANSACTION),
            msg: to_binary(&create_alias_message).unwrap(),
        };

        // = * it raises an error
        let response = handle(
            &mut deps,
            mock_env(mock_buttcoin().address, &[]),
            receive_msg,
        );
        let error = extract_error_msg(response);
        assert_eq!(error, "Alias is too long");

        // = when user already has an alias
        // = * it raises an error
        let alias = "Epstein didn't kill himself".repeat(5);
        let create_alias_message = ReceiveMsg::Create {
            alias: alias.to_string(),
            avatar_url: None,
        };
        let receive_msg = HandleMsg::Receive {
            sender: HumanAddr::from("frump"),
            from: HumanAddr::from("frump"),
            amount: Uint128(AMOUNT_FOR_TRANSACTION),
            msg: to_binary(&create_alias_message).unwrap(),
        };
        let response = handle(
            &mut deps,
            mock_env(mock_buttcoin().address, &[]),
            receive_msg,
        );
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
            QueryAnswer::Config {
                buttcoin,
                butt_lode,
            } => {
                assert_eq!(buttcoin, config.buttcoin);
                assert_eq!(butt_lode, config.butt_lode);
            }
        }
    }
}
