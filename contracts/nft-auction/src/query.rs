use cosmwasm_std::{Addr, Decimal, Deps, Env, Order, StdResult, Uint128};
use cw_storage_plus::Bound;

use crate::{
    msg::AuctionsResponse,
    state::{
        contract, order_key, Asset, DutchAuctionMetadata, EnglishAuctionMetadata, OrderComponents,
        OrderKey, NFT,
    },
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

// query valid price for bidding of a specific nft
pub fn query_valid_price(
    deps: Deps,
    env: Env,
    contract_address: Addr,
    token_id: String,
) -> StdResult<u128> {
    // create order key based on the offerer address, nft.contract_address and nft.token_id
    let order_key = order_key(&env.contract.address, &contract_address, &token_id);

    // get order
    let order = contract().auctions.load(deps.storage, order_key)?;

    // check if auction is expired
    if order.is_expired(&env.block) {
        Ok(0u128)
    } else {
        match &order.consideration[0].item {
            Asset::Native(current_price) => {
                // if type is EnglishAuction
                if order.config.order_type == "EnglishAuction" {
                    let previous_bidder = order.consideration[0].recipient.clone();
                    if previous_bidder != order.offer[0].offerer {
                        // get metadata of auction
                        let auction_matadata =
                            EnglishAuctionMetadata::from(order.config.metadata.clone());

                        // parse the step_percentage from order.config
                        let step_percentage = auction_matadata.step_percentage;

                        // check if the bid_price is greater than the current_price + step_price
                        let step_price =
                            Uint128::from(current_price.amount) * Decimal::percent(step_percentage);

                        Ok(current_price.amount.checked_add(step_price.into()).unwrap())
                    } else {
                        Ok(current_price.amount)
                    }
                } else if order.config.order_type == "DutchAuction" {
                    // if type is DutchAuction
                    // load the metadata of dutch_auction
                    let dutch_auction_metadata =
                        DutchAuctionMetadata::from(order.config.metadata.clone());
                    let bidding_price = Uint128::from(current_price.amount)
                        .checked_sub(
                            (Uint128::from(
                                env.block.time.nanos() - dutch_auction_metadata.start_time,
                            ))
                            .checked_div(Uint128::from(60_000_000_000u128))
                            .unwrap()
                            .checked_mul(Uint128::from(dutch_auction_metadata.step_amount))
                            .unwrap(),
                        )
                        .unwrap();
                    Ok(bidding_price.into())
                } else {
                    // if type is BlindAuction, return 0
                    Ok(0u128)
                }
            }
            _ => {
                // if type of consideration invalid, return 0
                Ok(0u128)
            }
        }
    }
}
