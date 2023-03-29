use std::vec;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Reply, ReplyOn,
    Response, StdResult, SubMsg, WasmMsg,
};
use cw2::set_contract_version;
use cw2981_royalties::msg::InstantiateMsg as Cw2981InstantiateMsg;
use cw_utils::parse_reply_instantiate_data;
use nois::randomness_from_str;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{
    Config, LaunchpadInfo, PhaseConfig, CONFIG, LAUNCHPAD_INFO, PHASE_CONFIGS, RANDOM_SEED,
};

use crate::execute::{
    active_launchpad, add_mint_phase, add_whitelist, deactive_launchpad, mint, remove_mint_phase,
    remove_whitelist, update_mint_phase, withdraw,
};

use crate::query::{query_all_phase_configs, query_launchpad_info, query_mintable};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:nft-launchpad";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Handling contract instantiation
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // save contract config
    let config = Config {
        admin: info.sender.clone(),
        launchpad_collector: deps
            .api
            .addr_validate(
                &msg.launchpad_collector
                    .unwrap_or_else(|| info.sender.to_string()),
            )
            .unwrap(),
    };
    CONFIG.save(deps.storage, &config)?;

    // store the address of the cw2981 collection contract
    LAUNCHPAD_INFO.save(
        deps.storage,
        &LaunchpadInfo {
            creator: deps
                .api
                .addr_validate(&msg.collection_info.creator)
                .unwrap(),
            collection_address: Addr::unchecked("".to_string()),
            start_phase_id: 0,
            last_phase_id: 0,
            last_issued_id: 0,
            total_supply: 0,
            uri_prefix: msg.collection_info.uri_prefix,
            uri_suffix: msg.collection_info.uri_suffix,
            max_supply: msg.collection_info.max_supply,
            is_active: false,
            launchpad_fee: if msg.launchpad_fee < 100 {
                // we will not take all the profit of creator ^^
                msg.launchpad_fee
            } else {
                return Err(ContractError::InvalidLaunchpadFee {});
            },
        },
    )?;

    // save the init RANDOM_SEED to the storage
    let randomness = randomness_from_str(msg.random_seed).unwrap();
    RANDOM_SEED.save(deps.storage, &randomness)?;

    // add an instantiate message for new cw2981 collection contract
    Ok(Response::new()
        .add_attributes(vec![
            ("action", "instantiate_launchpad"),
            ("collection_code_id", &msg.colection_code_id.to_string()),
        ])
        .add_submessage(SubMsg {
            id: 1,
            gas_limit: None,
            msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
                code_id: msg.colection_code_id,
                funds: vec![],
                admin: Some(info.sender.to_string()),
                label: "cw2981-instantiate".to_string(),
                msg: to_binary(&Cw2981InstantiateMsg {
                    name: msg.collection_info.name,
                    symbol: msg.collection_info.symbol,
                    minter: env.contract.address.to_string(),
                    royalty_percentage: msg.collection_info.royalty_percentage,
                    royalty_payment_address: msg.collection_info.royalty_payment_address,
                    creator: Some(msg.collection_info.creator),
                })?,
            }),
            reply_on: ReplyOn::Success,
        }))
}

/// This just stores the result for future query
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> StdResult<Response> {
    let reply = parse_reply_instantiate_data(msg).unwrap();

    // create a dummy phase config
    let phase_config = PhaseConfig {
        previous_phase_id: None,
        next_phase_id: None,
        start_time: env.block.time,
        end_time: env.block.time.plus_nanos(1), // start_time < end_time
        max_supply: Some(0),
        total_supply: 0,
        max_nfts_per_address: 0,
        price: coin(0, "uaura"),
        is_public: false,
    };

    // store the dummy phase config with the phase id 0
    PHASE_CONFIGS.save(deps.storage, 0, &phase_config)?;

    // load the launchpad info
    let mut launchpad_info = LAUNCHPAD_INFO.load(deps.storage).unwrap();
    launchpad_info.collection_address = deps.api.addr_validate(&reply.contract_address)?;

    // store the address of the cw2981 collection contract
    LAUNCHPAD_INFO.save(deps.storage, &launchpad_info)?;

    Ok(Response::new().add_attributes(vec![
        ("action", "instantiate_collection"),
        ("collection_address", &reply.contract_address),
    ]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    match msg {
        // Find matched incoming message variant and execute them with your custom logic.
        //
        // With `Response` type, it is possible to dispatch message to invoke external logic.
        // See: https://github.com/CosmWasm/cosmwasm/blob/main/SEMANTICS.md#dispatching-messages
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::AddMintPhase {
            after_phase_id,
            phase_data,
        } => add_mint_phase(deps, env, info, after_phase_id, phase_data),
        ExecuteMsg::UpdateMintPhase {
            phase_id,
            phase_data,
        } => update_mint_phase(deps, env, info, phase_id, phase_data),
        ExecuteMsg::RemoveMintPhase { phase_id } => remove_mint_phase(deps, env, info, phase_id),
        ExecuteMsg::AddWhitelist {
            phase_id,
            whitelists,
        } => add_whitelist(deps, env, info, phase_id, whitelists),
        ExecuteMsg::RemoveWhitelist {
            phase_id,
            addresses,
        } => remove_whitelist(deps, env, info, phase_id, addresses),
        ExecuteMsg::Mint { phase_id, amount } => mint(deps, env, info, phase_id, amount),
        ExecuteMsg::ActivateLaunchpad {} => active_launchpad(deps, info),
        ExecuteMsg::DeactivateLaunchpad {} => deactive_launchpad(deps, info),
        ExecuteMsg::Withdraw { denom } => withdraw(deps, env, info, denom),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetLaunchpadInfo {} => to_binary(&query_launchpad_info(deps)?),
        QueryMsg::GetAllPhaseConfigs {} => to_binary(&query_all_phase_configs(deps)?),
        QueryMsg::Mintable { user } => to_binary(&query_mintable(deps, Addr::unchecked(user))?),
    }
}
