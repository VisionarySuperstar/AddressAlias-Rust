use cosmwasm_std::{HumanAddr, StdError, StdResult};

pub fn authorize(expected: HumanAddr, received: HumanAddr) -> StdResult<()> {
    if expected != received {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    Ok(())
}
