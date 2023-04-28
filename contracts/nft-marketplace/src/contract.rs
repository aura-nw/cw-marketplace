#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{contract, Config};

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
    let conf = Config {
        owner: msg.owner,
        vaura_address: Addr::unchecked("aura0"),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    contract().config.save(deps.storage, &conf)?;

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
    let api = deps.api;
    match msg {
        ExecuteMsg::ListNft {
            contract_address,
            token_id,
            auction_config,
        } => contract().execute_list_nft(
            deps,
            _env,
            info,
            api.addr_validate(&contract_address)?,
            token_id,
            auction_config,
        ),
        ExecuteMsg::Buy {
            contract_address,
            token_id,
        } => contract().execute_buy(
            deps,
            _env,
            info,
            api.addr_validate(&contract_address)?,
            token_id,
        ),
        ExecuteMsg::Cancel {
            contract_address,
            token_id,
        } => contract().execute_cancel(
            deps,
            _env,
            info,
            api.addr_validate(&contract_address)?,
            token_id,
        ),
        ExecuteMsg::OfferNft {
            nft,
            funds_amount,
            end_time,
        } => contract().execute_offer_nft(deps, _env, info, nft, funds_amount, end_time),
        ExecuteMsg::AcceptNftOffer {
            offerer,
            nft,
            funds_amount,
        } => contract().execute_accept_nft_offer(
            deps,
            _env,
            info,
            api.addr_validate(&offerer)?,
            nft,
            funds_amount,
        ),
        ExecuteMsg::CancelOffer { nfts } => contract().execute_cancel_offer(deps, _env, info, nfts),
        ExecuteMsg::EditVauraToken { token_address } => {
            contract().execute_edit_vaura_token(deps, _env, info, token_address)
        }
        ExecuteMsg::AuctionNft {
            nft,
            auction_config,
        } => contract().execute_auction_nft(deps, _env, info, nft, auction_config),
        ExecuteMsg::BidNft {
            offerer,
            nft,
            bid_price,
        } => contract().execute_bid_nft(
            deps,
            _env,
            info,
            api.addr_validate(&offerer)?,
            nft,
            bid_price,
        ),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let api = deps.api;
    match msg {
        // get config
        QueryMsg::Config {} => to_binary(&contract().config.load(deps.storage)?),
        QueryMsg::ListingsByContractAddress {
            contract_address,
            start_after,
            limit,
        } => to_binary(&contract().query_listings_by_contract_address(
            deps,
            api.addr_validate(&contract_address)?,
            start_after,
            limit,
        )?),
        QueryMsg::Listing {
            contract_address,
            token_id,
        } => to_binary(&contract().query_listing(
            deps,
            api.addr_validate(&contract_address)?,
            token_id,
        )?),
        QueryMsg::Offer {
            contract_address,
            token_id,
            offerer,
        } => to_binary(&contract().query_offer(
            deps,
            api.addr_validate(&contract_address)?,
            token_id,
            api.addr_validate(&offerer)?,
        )?),
        QueryMsg::NftOffers {
            contract_address,
            token_id,
            start_after_offerer,
            limit,
        } => to_binary(&contract().query_nft_offers(
            deps,
            Addr::unchecked(contract_address),
            token_id,
            start_after_offerer,
            limit,
        )?),
        QueryMsg::UserOffers {
            offerer,
            start_after_nft,
            limit,
        } => to_binary(&contract().query_user_offers(
            deps,
            api.addr_validate(&offerer)?,
            start_after_nft,
            limit,
        )?),
    }
}
