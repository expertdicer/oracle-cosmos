use cosmwasm_std::{Api, StdResult, HumanAddr,CanonicalAddr};

pub fn optional_addr_validate(api: &dyn Api, addr: Option<String>) -> StdResult<Option<CanonicalAddr>> {
    let addr = if let Some(addr) = addr {
        let human_addr = HumanAddr(addr);
        Some(api.canonical_address(&human_addr)?)
    } else {
        None
    };

    Ok(addr)
}
