#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::execute::{execute_auction_nft, execute_bid_auction, execute_settle_auction};
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::query::{query_buyer_auctions, query_nft_auction, query_owner_auctions};
use crate::state::{Config, CONFIG};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:nft-marketplace";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // the default value of vaura_address is equal to "aura0" and MUST BE SET before offer nft
    let conf = Config { owner: msg.owner };
    CONFIG.save(deps.storage, &conf)?;

    // auctions = auctions();

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::AuctionNft {
            nft,
            auction_config,
        } => execute_auction_nft(deps, _env, info, nft, auction_config),
        ExecuteMsg::BidAuction { nft, bid_price } => {
            execute_bid_auction(deps, _env, info, nft, bid_price)
        }
        ExecuteMsg::SettleAuction { nft } => execute_settle_auction(deps, _env, info, nft),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let api = deps.api;
    match msg {
        QueryMsg::NftAuction {
            contract_address,
            token_id,
        } => to_binary(&query_nft_auction(
            deps,
            env,
            api.addr_validate(&contract_address)?,
            token_id,
        )?),
        QueryMsg::OwnerAuctions {
            owner,
            start_after_nft,
            limit,
        } => to_binary(&query_owner_auctions(
            deps,
            env,
            api.addr_validate(&owner)?,
            start_after_nft,
            limit,
        )?),
        QueryMsg::BuyerAuctions {
            buyer,
            start_after_nft,
            limit,
        } => to_binary(&query_buyer_auctions(
            deps,
            env,
            api.addr_validate(&buyer)?,
            start_after_nft,
            limit,
        )?),
    }
}
