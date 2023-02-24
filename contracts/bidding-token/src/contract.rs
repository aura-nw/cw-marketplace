#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    attr, to_binary, Addr, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response,
    StdError, StdResult, Uint128,
};

use cw2::set_contract_version;
use cw20::{AllowanceResponse, Expiration};
use cw20_base::allowances::query_allowance;
use cw20_base::contract::query as cw20_query;
use cw20_base::msg::{ExecuteMsg, QueryMsg};
use cw20_base::state::{MinterData, TokenInfo, BALANCES, TOKEN_INFO};
use cw20_base::ContractError;

use crate::state::{
    InstantiateMsg, MarketplaceInfo, SupportedNative, MARKETPLACE_INFO, SUPPORTED_NATIVE,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:bidding-token";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// this is the denominator of native token that is supported by this contract
pub static NATIVE_DENOM: &str = "uaura";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    // due to this contract is used for the marketplace once, so we don't need to check the validation of the message
    // // check valid token info
    // msg.validate()?;

    // this is a sanity check, to ensure that each token of this contract has garanteed by 1 native token
    if !msg.initial_balances.is_empty() {
        return Err(StdError::generic_err("Initial balances must be empty").into());
    }

    let init_supply = Uint128::zero();

    if let Some(limit) = msg.get_cap() {
        if init_supply > limit {
            return Err(StdError::generic_err("Initial supply greater than cap").into());
        }
    }

    // we force minter to be empty as anyone can mint
    let mint = match msg.mint {
        Some(m) => Some(MinterData {
            minter: Addr::unchecked(""),
            cap: m.cap,
        }),
        None => None,
    };

    // store token info
    let data = TokenInfo {
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
        total_supply: init_supply,
        mint,
    };
    TOKEN_INFO.save(deps.storage, &data)?;

    // set value for NATIVE_DENOM and marketplace contract address
    MARKETPLACE_INFO.save(
        deps.storage,
        &MarketplaceInfo {
            contract_address: msg.marketplace_address,
        },
    )?;

    SUPPORTED_NATIVE.save(
        deps.storage,
        &SupportedNative {
            denom: msg.native_denom,
        },
    )?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Burn { amount } => execute_burn(deps, env, info, amount),
        ExecuteMsg::Mint {
            recipient,
            amount: _,
        } => execute_mint(deps, env, info, recipient),
        ExecuteMsg::TransferFrom {
            owner,
            recipient,
            amount,
        } => execute_transfer_from(deps, env, info, owner, recipient, amount),
        // TODO: add message to update MarketplaceInfo here
        _ => {
            // the other messages not supported by this contract
            Err(StdError::generic_err("Unsupported message").into())
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    // TODO: add query for MarketplaceInfo here
    match msg {
        QueryMsg::Allowance { owner, spender } => {
            let marketplace_info = MARKETPLACE_INFO.load(deps.storage)?;
            if spender == marketplace_info.contract_address {
                // if spender is marketplace contract, return cap of minter
                to_binary(&marketplace_query_allowance(deps)?)
            } else {
                to_binary(&query_allowance(deps, owner, spender)?)
            }
        }
        _ => cw20_query(deps, env, msg),
    }
}

pub fn marketplace_query_allowance(deps: Deps) -> StdResult<AllowanceResponse> {
    // get cap from mint data
    let minter = TOKEN_INFO.load(deps.storage).unwrap().mint.unwrap();
    let cap = minter.cap.unwrap_or_default();

    Ok(AllowanceResponse {
        allowance: cap,
        expires: Expiration::Never {},
    })
}

fn validate_balance(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
    let native_denom = SUPPORTED_NATIVE.load(deps.storage)?.denom;
    let total_supply = TOKEN_INFO.load(deps.storage)?.total_supply;
    let balance = deps
        .querier
        .query_balance(env.contract.address, native_denom)
        .unwrap()
        .amount;

    if balance < total_supply {
        return Err(ContractError::Std(StdError::generic_err(format!(
            "Invalid balance, {}, {}",
            total_supply, balance,
        ))));
    }
    Ok(Response::new())
}

// After a user burn the token, contract will return the same amount of native token to him
// This function is taken from cw20-base with some modifications
pub fn execute_burn(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    if amount == Uint128::zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    // lower balance of sender
    BALANCES.update(
        deps.storage,
        &info.sender,
        |balance: Option<Uint128>| -> StdResult<_> {
            Ok(balance.unwrap_or_default().checked_sub(amount)?)
        },
    )?;

    // reduce total_supply
    TOKEN_INFO.update(deps.storage, |mut info| -> StdResult<_> {
        info.total_supply = info.total_supply.checked_sub(amount)?;
        Ok(info)
    })?;

    let native_denom = SUPPORTED_NATIVE.load(deps.storage)?.denom;

    // transfer native tokens to sender
    let transfer_native_msg = BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![Coin {
            denom: native_denom,
            amount,
        }],
    };

    // TODO: we would like to run validate_balance but we will need a submessage for that
    // for some reason, the submessage is not working
    // will check later
    Ok(Response::new()
        .add_message(transfer_native_msg)
        .add_attribute("action", "burn")
        .add_attribute("from", info.sender)
        .add_attribute("amount", amount))
}

// Every user send native token to this contract, and the contract will mint the same amount of token to the user.
// This function is taken from cw20-base with some modifications
pub fn execute_mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
) -> Result<Response, ContractError> {
    if info.funds[0].amount == Uint128::zero() {
        return Err(ContractError::Unauthorized {});
    }

    let mut config = TOKEN_INFO
        .may_load(deps.storage)?
        .ok_or(ContractError::Unauthorized {})?;

    // check the funds are sent with the message
    // if the denom of funds is not the same as the native denom, we reject
    let native_denom = SUPPORTED_NATIVE.load(deps.storage)?.denom;
    if info.funds.len() != 1 || info.funds[0].denom != native_denom {
        return Err(ContractError::Unauthorized {});
    }

    // update supply and enforce cap
    config.total_supply += info.funds[0].amount;
    if let Some(limit) = config.get_cap() {
        if config.total_supply > limit {
            return Err(ContractError::CannotExceedCap {});
        }
    }
    TOKEN_INFO.save(deps.storage, &config)?;

    // add amount to recipient balance
    let rcpt_addr = deps.api.addr_validate(&recipient)?;
    BALANCES.update(
        deps.storage,
        &rcpt_addr,
        |balance: Option<Uint128>| -> StdResult<_> {
            Ok(balance.unwrap_or_default() + info.funds[0].amount)
        },
    )?;

    validate_balance(deps, env).unwrap();

    Ok(Response::new()
        .add_attribute("action", "mint")
        .add_attribute("to", recipient)
        .add_attribute("amount", info.funds[0].amount))
}

pub fn execute_transfer_from(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: String,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    // this function is called by marketplace contract only
    // get marketplace address from mint data
    let marketplace = MARKETPLACE_INFO.load(deps.storage)?.contract_address;

    // check if the sender is not minter
    if marketplace != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let rcpt_addr = deps.api.addr_validate(&recipient)?;
    let owner_addr = deps.api.addr_validate(&owner)?;

    BALANCES.update(
        deps.storage,
        &owner_addr,
        |balance: Option<Uint128>| -> StdResult<_> {
            Ok(balance.unwrap_or_default().checked_sub(amount)?)
        },
    )?;

    BALANCES.update(
        deps.storage,
        &rcpt_addr,
        |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) },
    )?;

    validate_balance(deps, env).unwrap();

    Ok(Response::new().add_attributes(vec![
        attr("action", "transfer_from"),
        attr("from", owner),
        attr("to", recipient),
        attr("by", info.sender),
        attr("amount", amount),
    ]))
}
