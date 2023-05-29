use cosmwasm_std::{Addr, Deps, Env, Order, StdResult};
use cw_storage_plus::Bound;

use crate::{
    msg::AuctionsResponse,
    state::{contract, order_key, OrderComponents, OrderKey, NFT},
};

// query all auctions of a specific nft
pub fn query_nft_auction(
    deps: Deps,
    env: Env,
    contract_address: Addr,
    token_id: String,
) -> StdResult<OrderComponents> {
    // create order key based on the offerer address, nft.contract_address and nft.token_id
    let order_key = order_key(&env.contract.address, &contract_address, &token_id);

    // get order
    let order = contract().auctions.load(deps.storage, order_key)?;

    // return offers
    Ok(order)
}

// query all auctions of a specific owner
pub fn query_owner_auctions(
    deps: Deps,
    env: Env,
    owner: Addr,
    start_after_nft: Option<NFT>,
    limit: Option<u32>,
) -> StdResult<AuctionsResponse> {
    let limit = limit.unwrap_or(30).min(30) as usize;

    let start: Option<Bound<OrderKey>> = start_after_nft.map(|nft| {
        let order_key = order_key(
            &env.contract.address,
            &nft.contract_address,
            &nft.token_id.unwrap(),
        );
        Bound::exclusive(order_key)
    });

    // load auctions
    let auctions = contract()
        .auctions
        .idx
        .owners
        .prefix(owner)
        .range(deps.storage, start, None, Order::Descending)
        .map(|item| item.map(|(_, auction)| auction))
        .take(limit)
        .collect::<StdResult<Vec<_>>>()?;

    // return auctions
    Ok(AuctionsResponse { auctions })
}

// query all auctions of a specific buyer
pub fn query_buyer_auctions(
    deps: Deps,
    env: Env,
    buyer: Addr,
    start_after_nft: Option<NFT>,
    limit: Option<u32>,
) -> StdResult<AuctionsResponse> {
    let limit = limit.unwrap_or(30).min(30) as usize;

    let start: Option<Bound<OrderKey>> = start_after_nft.map(|nft| {
        let order_key = order_key(
            &env.contract.address,
            &nft.contract_address,
            &nft.token_id.unwrap(),
        );
        Bound::exclusive(order_key)
    });

    // load auctions
    let auctions = contract()
        .auctions
        .idx
        .buyers
        .prefix(buyer)
        .range(deps.storage, start, None, Order::Descending)
        .map(|item| item.map(|(_, order)| order))
        .take(limit)
        .collect::<StdResult<Vec<_>>>()?;

    // return auctions
    Ok(AuctionsResponse { auctions })
}
