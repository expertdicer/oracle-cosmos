use cosmwasm_std::{Api, StdResult, HumanAddr,CanonicalAddr};

pub fn optional_addr_validate(api: &dyn Api, addr: Option<String>) -> StdResult<Option<CanonicalAddr>> {
    let addr = if let Some(addr) = addr {
        let humanAddr = HumanAddr(addr);
        Some(api.canonical_address(&humanAddr)?)
    } else {
        None
    };

    Ok(addr)
}
