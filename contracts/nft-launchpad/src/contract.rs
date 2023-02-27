use std::vec;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, has_coins, to_binary, Addr, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Reply, ReplyOn, Response, StdResult, Storage, SubMsg, Timestamp, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw2981_royalties::MintMsg;
use cw2981_royalties::{
    msg::InstantiateMsg as Cw2981InstantiateMsg, ExecuteMsg as Cw2981ExecuteMsg,
};
use cw_utils::parse_reply_instantiate_data;
use nois::{ints_in_range, randomness_from_str, sub_randomness_with_key};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, MintableResponse, QueryMsg};
use crate::state::{
    Config, LaunchpadInfo, PhaseConfig, PhaseConfigResponse, PhaseData, CONFIG, LAUNCHPAD_INFO,
    PHASE_CONFIGS, PHASE_CONFIGS_RESPONSE, RANDOM_SEED, REMAINING_TOKEN_IDS, WHITELIST,
};

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
        owner: info.sender.clone(),
    };
    CONFIG.save(deps.storage, &config)?;

    // store the address of the cw2981 collection contract
    LAUNCHPAD_INFO.save(
        deps.storage,
        &LaunchpadInfo {
            collection_address: Addr::unchecked("".to_string()),
            first_phase_id: 0,
            last_phase_id: 0,
            last_issued_id: 0,
            total_supply: 0,
            uri_prefix: msg.collection_info.uri_prefix,
            max_supply: msg.collection_info.max_supply,
            // set is_active to false by default, admin must activate it manually.
            // After ativation, cannot add, update or remove phase.
            // After ativation, cannot add or remove whitelist, too.
            is_active: false,
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
    }
}

fn add_mint_phase(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    after_phase_id: Option<u64>,
    phase_data: PhaseData,
) -> Result<Response, ContractError> {
    // check if the launchpad started, then return error
    if is_launchpad_started(deps.storage) {
        return Err(ContractError::LaunchpadStarted {});
    }

    // check if the sender is not the owner, then return error
    let config: Config = CONFIG.load(deps.storage)?;
    if config.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    //load the launchpad_info
    let mut launchpad_info = LAUNCHPAD_INFO.load(deps.storage)?;

    // the valid_phase_id is equal to the last_issed_phase_id + 1
    let valid_phase_id = launchpad_info.last_issued_id + 1;

    // save the last_issued_phase_id
    launchpad_info.last_issued_id = valid_phase_id;
    LAUNCHPAD_INFO.save(deps.storage, &launchpad_info)?;

    // match the after_phase_id none or not
    match after_phase_id {
        // if the after_phase_id is not none, then this phase should be added to the middle of the phase_configs
        Some(after_phase_id) => {
            // get the previous_phase_config of new phase
            let mut previous_phase_config = PHASE_CONFIGS.load(deps.storage, after_phase_id)?;

            // get the next_phase_id of the previous_phase_config
            let next_phase_id = previous_phase_config.next_phase_id;

            // check the time of phase_data is valid
            if !verify_phase_time(
                &deps,
                env,
                Some(after_phase_id),
                next_phase_id,
                phase_data.start_time,
                phase_data.end_time,
            ) {
                return Err(ContractError::InvalidPhaseTime {});
            }

            // the new phase_config is constructed from the phase_data,
            // its previous_phase_id is after_phase_id, next_phase_id is next_phase_id the of previous phase
            // and the key of item is the valid_phase_id
            let phase_config_data = PhaseConfig {
                previous_phase_id: Some(after_phase_id),
                next_phase_id: previous_phase_config.next_phase_id,
                start_time: phase_data.start_time,
                end_time: phase_data.end_time,
                max_supply: phase_data.max_supply,
                total_supply: 0,
                max_nfts_per_address: phase_data.max_nfts_per_address,
                price: phase_data.price,
                is_public: phase_data.is_public,
            };
            PHASE_CONFIGS.save(deps.storage, valid_phase_id, &phase_config_data)?;

            // update the next_phase_id of the previous_phase_config
            previous_phase_config.next_phase_id = Some(valid_phase_id);
            PHASE_CONFIGS.save(deps.storage, after_phase_id, &previous_phase_config)?;

            // if the next_phase_id of the phase_config is none, then update the last_phase_id of the launchpad_info
            if phase_config_data.next_phase_id.is_none() {
                let mut launchpad_info = LAUNCHPAD_INFO.load(deps.storage)?;
                launchpad_info.last_phase_id = valid_phase_id;
                LAUNCHPAD_INFO.save(deps.storage, &launchpad_info)?;
            }
            // update the phase_configs_response
            update_phases_config_response(deps)?;
        }
        // if the after_phase_id is none, then add the phase_data to the last item of the phase_configs
        None => {
            // get the last_phase_id
            let last_phase_id = LAUNCHPAD_INFO.load(deps.storage)?.last_phase_id;

            // check the time of phase_data is valid
            if !verify_phase_time(
                &deps,
                env,
                after_phase_id,
                None,
                phase_data.start_time,
                phase_data.end_time,
            ) {
                return Err(ContractError::InvalidPhaseTime {});
            }

            // the phase_config should be constructed from the phase_data,
            // its previous_phase_id should be last_phase_id, next_phase_id should be None
            // and the key of item is the valid_phase_id
            let phase_config_data = PhaseConfig {
                previous_phase_id: Some(last_phase_id),
                next_phase_id: None,
                start_time: phase_data.start_time,
                end_time: phase_data.end_time,
                max_supply: phase_data.max_supply,
                total_supply: 0,
                max_nfts_per_address: phase_data.max_nfts_per_address,
                price: phase_data.price,
                is_public: phase_data.is_public,
            };
            PHASE_CONFIGS.save(deps.storage, valid_phase_id, &phase_config_data)?;

            // update the next_phase_id of the last_phase_id
            let mut last_phase_config = PHASE_CONFIGS.load(deps.storage, last_phase_id)?;
            last_phase_config.next_phase_id = Some(valid_phase_id);
            PHASE_CONFIGS.save(deps.storage, last_phase_id, &last_phase_config)?;

            // update the last_phase_id of the launchpad_info
            let mut launchpad_info = LAUNCHPAD_INFO.load(deps.storage)?;
            launchpad_info.last_phase_id = valid_phase_id;
            LAUNCHPAD_INFO.save(deps.storage, &launchpad_info)?;

            // update the phase_configs_response
            update_phases_config_response(deps)?;
        }
    }

    Ok(Response::new())
}

// update the mint phase with the phase_id
pub fn update_mint_phase(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    phase_id: u64,
    phase_data: PhaseData,
) -> Result<Response, ContractError> {
    // check if the launchpad started, then return error
    if is_launchpad_started(deps.storage) {
        return Err(ContractError::LaunchpadStarted {});
    }

    // check if the sender is not the owner, then return error
    let config: Config = CONFIG.load(deps.storage)?;
    if config.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // check if the phase_id is valid
    if !PHASE_CONFIGS.has(deps.storage, phase_id) {
        return Err(ContractError::InvalidPhaseId {});
    }

    // check the new time of the phase_data
    if !verify_phase_time(
        &deps,
        env,
        PHASE_CONFIGS
            .load(deps.storage, phase_id)?
            .previous_phase_id, // get the previous_phase_id of the phase_id
        PHASE_CONFIGS.load(deps.storage, phase_id)?.next_phase_id, // get the next_phase_id of the phase_id
        phase_data.start_time,
        phase_data.end_time,
    ) {
        return Err(ContractError::InvalidPhaseTime {});
    }

    // load the phase configs data from the storage
    let phase_config: PhaseConfig = PHASE_CONFIGS.load(deps.storage, phase_id).unwrap();

    // save the new phase data to the storage
    PHASE_CONFIGS.save(
        deps.storage,
        phase_id,
        &PhaseConfig {
            start_time: phase_data.start_time,
            end_time: phase_data.end_time,
            max_supply: phase_data.max_supply,
            max_nfts_per_address: phase_data.max_nfts_per_address,
            price: phase_data.price.clone(),
            is_public: phase_data.is_public,
            ..phase_config
        },
    )?;

    // update the phase_configs_response
    update_phases_config_response(deps)?;

    Ok(Response::new().add_attributes([
        ("action", "update_mint_phase"),
        ("phase_id", &phase_id.to_string()),
        ("start_time", &phase_data.start_time.to_string()),
        ("end_time", &phase_data.end_time.to_string()),
        // ("max_supply", Some(&phase_data.max_supply).unwrap()),
        (
            "max_nfts_per_address",
            &phase_data.max_nfts_per_address.to_string(),
        ),
        ("price_denom", &phase_data.price.denom),
    ]))
}

pub fn remove_mint_phase(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    phase_id: u64,
) -> Result<Response, ContractError> {
    // check if the launchpad started, then return error
    if is_launchpad_started(deps.storage) {
        return Err(ContractError::LaunchpadStarted {});
    }

    // check if the sender is not the owner, then return error
    let config: Config = CONFIG.load(deps.storage)?;
    if config.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // cannot remove the phase with id 0
    // the dummy phase is always there
    if phase_id == 0 {
        return Err(ContractError::InvalidPhaseId {});
    }

    // check if the phase_id is valid
    if !PHASE_CONFIGS.has(deps.storage, phase_id) {
        return Err(ContractError::InvalidPhaseId {});
    }

    // load the phase configs data from the storage
    let phase_config: PhaseConfig = PHASE_CONFIGS.load(deps.storage, phase_id).unwrap();

    let launchpad_info = LAUNCHPAD_INFO.load(deps.storage)?;
    // if the phase_id is the last_phase_id, then update the last_phase_id of the launchpad_info
    if launchpad_info.last_phase_id == phase_id {
        // change the last_phase_id of the launchpad_info to the previous_phase_id of the phase_id
        let mut launchpad_info = LAUNCHPAD_INFO.load(deps.storage)?;
        launchpad_info.last_phase_id = phase_config.previous_phase_id.unwrap();
        LAUNCHPAD_INFO.save(deps.storage, &launchpad_info)?;

        // remove the next_phase_id of the previous_phase_id
        let mut previous_phase_config: PhaseConfig = PHASE_CONFIGS
            .load(deps.storage, phase_config.previous_phase_id.unwrap())
            .unwrap();
        previous_phase_config.next_phase_id = None;
        PHASE_CONFIGS.save(
            deps.storage,
            phase_config.previous_phase_id.unwrap(),
            &previous_phase_config,
        )?;

        // remove the phase_id from the storage
        PHASE_CONFIGS.remove(deps.storage, phase_id);

        // update the phase_configs_response
        update_phases_config_response(deps)?;

        Ok(Response::new().add_attributes([
            ("action", "remove_mint_phase"),
            ("phase_id", &phase_id.to_string()),
        ]))
    }
    // else the launchpad is at the middle of phase_configs, then update the next_phase_id and previous_phase_id of the phase_configs
    else {
        // load the previous_phase_id of the phase_id
        let previous_phase_id = phase_config.previous_phase_id.unwrap();
        // load the next_phase_id of the phase_id
        let next_phase_id = phase_config.next_phase_id.unwrap();

        // update the next_phase_id of the previous_phase_id
        let mut previous_phase_config: PhaseConfig =
            PHASE_CONFIGS.load(deps.storage, previous_phase_id).unwrap();
        previous_phase_config.next_phase_id = Some(next_phase_id);
        PHASE_CONFIGS.save(deps.storage, previous_phase_id, &previous_phase_config)?;

        // update the previous_phase_id of the next_phase_id
        let mut next_phase_config: PhaseConfig =
            PHASE_CONFIGS.load(deps.storage, next_phase_id).unwrap();
        next_phase_config.previous_phase_id = Some(previous_phase_id);
        PHASE_CONFIGS.save(deps.storage, next_phase_id, &next_phase_config)?;

        // remove the phase_id from the storage
        PHASE_CONFIGS.remove(deps.storage, phase_id);

        // update the phase_configs_response
        update_phases_config_response(deps)?;

        // TODO: remove the phase_id from the whitelist

        Ok(Response::new().add_attributes([
            ("action", "remove_mint_phase"),
            ("phase_id", &phase_id.to_string()),
        ]))
    }
}

pub fn add_whitelist(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    phase_id: u64,
    whitelists: Vec<String>,
) -> Result<Response, ContractError> {
    // check if the launchpad started, then return error
    if is_launchpad_started(deps.storage) {
        return Err(ContractError::LaunchpadStarted {});
    }

    // check if the sender is not the owner, then return error
    let config: Config = CONFIG.load(deps.storage)?;
    if config.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // for each address in the whitelist input, add the address to the whitelist of the phase_id
    for address in whitelists {
        // if the address is not in WHITELIST, then save it to the WHITELIST
        if !WHITELIST.has(deps.storage, (phase_id, Addr::unchecked(address.clone()))) {
            WHITELIST.save(
                deps.storage,
                (phase_id, Addr::unchecked(address.clone())),
                &0,
            )?;
        }
    }

    Ok(Response::new().add_attributes([("action", "add_whitelist")]))
}

pub fn remove_whitelist(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    phase_id: u64,
    addresses: Vec<String>,
) -> Result<Response, ContractError> {
    // check if the launchpad started, then return error
    if is_launchpad_started(deps.storage) {
        return Err(ContractError::LaunchpadStarted {});
    }

    // check if the sender is not the owner, then return error
    let config: Config = CONFIG.load(deps.storage)?;
    if config.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // for each address in the whitelist input, remove the address from the whitelist of the phase_id
    for address in addresses {
        // if the address is in the WHITELIST, then remove it from the WHITELIST
        if WHITELIST.has(deps.storage, (phase_id, Addr::unchecked(address.clone()))) {
            WHITELIST.remove(deps.storage, (phase_id, Addr::unchecked(address)));
        }
    }

    Ok(Response::new().add_attributes([("action", "remove_whitelist")]))
}

pub fn mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    phase_id: u64,
    amount: Option<u64>,
) -> Result<Response, ContractError> {
    let amount_nfts = amount.unwrap_or(1);

    if amount_nfts > 10 {
        return Err(ContractError::TooManyNfts {});
    }

    // get the launchpad info
    let mut launchpad_info: LaunchpadInfo = LAUNCHPAD_INFO.load(deps.storage)?;

    // if the launchpad is deactivated, then return error
    if !launchpad_info.is_active {
        return Err(ContractError::LaunchpadIsDeactivated {});
    }

    // load the phase_config of the phase_id
    let mut phase_config: PhaseConfig = PHASE_CONFIGS.load(deps.storage, phase_id).unwrap();

    // mayload the minted_nfts of sender
    let minted_nfts_result = WHITELIST
        .may_load(deps.storage, (phase_id, info.sender.clone()))
        .unwrap();

    // check if the phase is not public and the sender is not in the whitelist of the phase_id, then return error
    if !phase_config.is_public && minted_nfts_result.is_none() {
        return Err(ContractError::Unauthorized {});
    }

    // check if the current time is not in the phase_config, then return error
    if env.block.time < phase_config.start_time || env.block.time > phase_config.end_time {
        return Err(ContractError::PhaseIsInactivated {});
    }

    // check if the total supply of the phase_id is greater than or equal to the max_supply, then return error
    if launchpad_info.total_supply + amount_nfts > launchpad_info.max_supply
        || (phase_config.max_supply.is_some()
            && phase_config.total_supply + amount_nfts > phase_config.max_supply.unwrap())
    {
        return Err(ContractError::MaxSupplyReached {});
    }
    // increase the total supply of the phase_id
    phase_config.total_supply += amount_nfts;
    PHASE_CONFIGS.save(deps.storage, phase_id, &phase_config)?;

    // check if the number of minted NFTs of the sender is greater than or equal to the max_mint of the phase_id, then return error
    let mut minted_nfts = minted_nfts_result.unwrap_or(0u64);
    if minted_nfts + amount_nfts > phase_config.max_nfts_per_address {
        return Err(ContractError::UserMintedTooMuchNfts {});
    }

    // increase the number of minted NFTs of the sender
    minted_nfts += amount_nfts;
    WHITELIST.save(deps.storage, (phase_id, info.sender.clone()), &minted_nfts)?;

    // check if the funds is not enough, then return error
    if !has_coins(
        &info.funds,
        &Coin {
            denom: phase_config.price.denom,
            amount: phase_config
                .price
                .amount
                .checked_mul(Uint128::from(amount_nfts))
                .unwrap(),
        },
    ) {
        return Err(ContractError::NotEnoughFunds {});
    }

    // get current time
    let current_time = env.block.time;

    // mint NFT(s) for the sender
    let mut res: Response = Response::new();
    for _ in 0..amount_nfts {
        // generate random token_id
        let token_id =
            generate_random_token_id(deps.storage, current_time, info.sender.to_string()).unwrap();

        // get the token_uri based on the token_id
        let token_uri = get_token_uri(&launchpad_info.uri_prefix, &token_id);

        // create mint message NFT for the sender
        let mint_msg = WasmMsg::Execute {
            contract_addr: launchpad_info.collection_address.to_string(),
            msg: to_binary(&Cw2981ExecuteMsg::Mint(MintMsg {
                token_id,
                owner: info.sender.to_string(),
                token_uri: Some(token_uri),
                extension: None,
            }))?,
            funds: vec![],
        };

        res = res.add_message(mint_msg);
    }

    // increase the total supply of the the launchpad
    launchpad_info.total_supply += amount_nfts;
    LAUNCHPAD_INFO.save(deps.storage, &launchpad_info)?;

    Ok(res.add_attributes([
        ("action", "launchpad_mint"),
        ("owner", info.sender.as_ref()),
        ("phase_id", &phase_id.to_string()),
        ("amount", &amount_nfts.to_string()),
    ]))
}

pub fn active_launchpad(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    // check if the sender is not the owner, then return error
    let config: Config = CONFIG.load(deps.storage)?;
    if config.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // get the launchpad info
    let mut launchpad_info: LaunchpadInfo = LAUNCHPAD_INFO.load(deps.storage)?;

    // check if the launchpad is already activated, then return error
    if launchpad_info.is_active {
        return Err(ContractError::LaunchpadIsActivated {});
    }

    // activate the launchpad
    launchpad_info.is_active = true;
    LAUNCHPAD_INFO.save(deps.storage, &launchpad_info)?;

    Ok(Response::new().add_attributes([("action", "active_launchpad")]))
}

pub fn deactive_launchpad(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    // check if the sender is not the owner, then return error
    let config: Config = CONFIG.load(deps.storage)?;
    if config.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // get the launchpad info
    let mut launchpad_info: LaunchpadInfo = LAUNCHPAD_INFO.load(deps.storage)?;

    // check if the launchpad is already deactivated, then return error
    if !launchpad_info.is_active {
        return Err(ContractError::LaunchpadIsDeactivated {});
    }

    // deactivate the launchpad
    launchpad_info.is_active = false;
    LAUNCHPAD_INFO.save(deps.storage, &launchpad_info)?;

    Ok(Response::new().add_attributes([("action", "deactivated_launchpad")]))
}

fn generate_random_token_id(
    storage: &mut dyn Storage,
    current_time: Timestamp,
    sender: String,
) -> StdResult<String> {
    // load RANDOM_SEED from the storage
    let random_seed = RANDOM_SEED.load(storage).unwrap();

    // init a key for the random provider from the msg.sender and current time
    let key = format!("{}{}", sender, current_time);

    // define random provider from the random_seed
    let mut provider = sub_randomness_with_key(random_seed, key);

    // get the number of remaining nfts launchpad
    let launchpad_info: LaunchpadInfo = LAUNCHPAD_INFO.load(storage).unwrap();
    let remaining_nfts = launchpad_info.max_supply - launchpad_info.total_supply;

    // random a new random_seed
    let new_random_seed = provider.provide();
    RANDOM_SEED.save(storage, &new_random_seed)?;

    // random a randomness for random tokne_id
    let randomness = provider.provide();

    // we use a variable to determine the position of the token_id in the REMAINING_TOKEN_IDS
    let mut token_id_position = 0;

    // if the number of remaining nfts is greater then 1, then we will choose a random position
    if remaining_nfts > 1 {
        // random a number from 0 to remaining_nfts-1
        let random_positions = ints_in_range(randomness, 1, 0, remaining_nfts - 1);
        token_id_position = random_positions[0];
    }

    get_token_id_from_position(storage, token_id_position)
}

fn get_token_id_from_position(storage: &mut dyn Storage, position: u64) -> StdResult<String> {
    // get the number of remaining nfts launchpad
    let launchpad_info: LaunchpadInfo = LAUNCHPAD_INFO.load(storage).unwrap();
    let remaining_nfts = launchpad_info.max_supply - launchpad_info.total_supply;

    // get the current token_id at the token_id_position
    // if the token_id at the token_id_position is equal 0, then return its position
    // else, return the token_id at the token_id_position
    let token_id = REMAINING_TOKEN_IDS
        .may_load(storage, position)
        .unwrap()
        .unwrap_or(position + 1);

    println!("remaining_nfts: {} {}", remaining_nfts, position);
    // determine the id in the last position of the remaining_token_ids
    let last_token_id = REMAINING_TOKEN_IDS
        .may_load(storage, remaining_nfts - 1)
        .unwrap()
        .unwrap_or(remaining_nfts);

    // now, swap the token_id with the last_token_id in the remaining_token_ids
    REMAINING_TOKEN_IDS.save(storage, position, &last_token_id)?;

    // remove the last item of the remaining_token_ids
    REMAINING_TOKEN_IDS.remove(storage, remaining_nfts - 1);

    // return the token_id
    Ok(token_id.to_string())
}

fn get_token_uri(uri_prefix: &str, token_id: &str) -> String {
    // TODO: maybe we need the suffix of the token_uri, too
    // the token_uri is the uri_prefix + token_id
    format!("{}{}", uri_prefix, token_id)
}

fn verify_phase_time(
    deps: &DepsMut,
    env: Env,
    previous_phase_id: Option<u64>,
    next_phase_id: Option<u64>,
    start_time: Timestamp,
    end_time: Timestamp,
) -> bool {
    // check if the start time is not less than the end time
    if start_time > end_time {
        return false;
    }

    // if the last_phase_id is 0 (there is no phase), then the start time must be greater than the current time
    let last_phase_id = LAUNCHPAD_INFO.load(deps.storage).unwrap().last_phase_id;
    if last_phase_id == 0 && start_time < env.block.time {
        return false;
    }

    // match the previous_phase_id is none or not
    match previous_phase_id {
        // if the previous_phase_id is none, then the start time must be greater than the end time of the last phase
        None => {
            // get the last phase id
            let last_phase_id = LAUNCHPAD_INFO.load(deps.storage).unwrap().last_phase_id;

            // get the last phase config
            let last_phase_config = PHASE_CONFIGS.load(deps.storage, last_phase_id).unwrap();

            // check if the start time is not less than the end time of the last phase
            if start_time < last_phase_config.end_time {
                return false;
            }
        }
        // if the previous_phase_id is NOT none,
        // then the start_time must be greater than the end_time of the previous phase
        // and the end_time must be less than the start_time of the next phase
        Some(previous_phase_id) => {
            let previous_phase_config =
                PHASE_CONFIGS.load(deps.storage, previous_phase_id).unwrap();

            // check if the start time is not less than the end time of the previous phase
            if start_time < previous_phase_config.end_time {
                return false;
            }

            // check if the end time is not greater than the start time of the next phase of the previous phase
            if let Some(next_phase_id) = next_phase_id {
                let next_phase_config = PHASE_CONFIGS.load(deps.storage, next_phase_id).unwrap();
                if end_time > next_phase_config.start_time {
                    return false;
                }
            }
        }
    }

    true
}

fn update_phases_config_response(deps: DepsMut) -> StdResult<()> {
    // load the last_phase_id
    let last_phase_id = LAUNCHPAD_INFO.load(deps.storage).unwrap().last_phase_id;

    let mut phase_id = 0;

    // get the dummy phase config
    let mut phase_config = PHASE_CONFIGS.load(deps.storage, phase_id).unwrap();

    // create an empty PHASE_CONFIGS_RESPONSE
    let mut phase_configs_response: Vec<PhaseConfigResponse> = vec![];

    // from phase_id 0, loop through all the phase_configs, while the next_phase_id is not None
    while phase_id != last_phase_id {
        // get the next phase id
        phase_id = phase_config.next_phase_id.unwrap();

        // get the next phase config
        phase_config = PHASE_CONFIGS.load(deps.storage, phase_id).unwrap();

        // add the phase_config to the PHASE_CONFIGS_RESPONSE
        phase_configs_response.push(PhaseConfigResponse {
            phase_id,
            start_time: phase_config.start_time,
            end_time: phase_config.end_time,
            max_supply: phase_config.max_supply,
            total_supply: phase_config.total_supply,
            max_nfts_per_address: phase_config.max_nfts_per_address,
            price: phase_config.price,
            is_public: phase_config.is_public,
        });
    }

    // update the PHASE_CONFIGS_RESPONSE
    PHASE_CONFIGS_RESPONSE.save(deps.storage, &phase_configs_response)?;

    Ok(())
}

// we need a function to check when the launchpad started
fn is_launchpad_started(storage: &dyn Storage) -> bool {
    // load the status of the launchpad
    let launchpad_info = LAUNCHPAD_INFO.load(storage).unwrap();

    launchpad_info.is_active
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetLaunchpadInfo {} => to_binary(&query_launchpad_info(deps)?),
        QueryMsg::GetAllPhaseConfigs {} => to_binary(&query_all_phase_configs(deps)?),
        QueryMsg::Mintable { user } => to_binary(&query_mintable(deps, Addr::unchecked(user))?),
    }
}

pub fn query_launchpad_info(deps: Deps) -> StdResult<LaunchpadInfo> {
    let launchpad_info = LAUNCHPAD_INFO.load(deps.storage)?;
    Ok(launchpad_info)
}

pub fn query_all_phase_configs(deps: Deps) -> StdResult<Vec<PhaseConfigResponse>> {
    let phase_configs_response = PHASE_CONFIGS_RESPONSE.load(deps.storage)?;
    Ok(phase_configs_response)
}

pub fn query_mintable(deps: Deps, user: Addr) -> StdResult<Vec<MintableResponse>> {
    // load the information of phase configs
    let phase_configs = PHASE_CONFIGS_RESPONSE.load(deps.storage)?;

    // create an empty mintable response vector
    let mut mintable_response: Vec<MintableResponse> = vec![];

    for phase_config in phase_configs {
        // load the number of minted nfts of the user from WHITELIST base of the phase_id and the address of user
        let minted_nfts = if WHITELIST.has(deps.storage, (phase_config.phase_id, user.clone())) {
            WHITELIST
                .load(deps.storage, (phase_config.phase_id, user.clone()))
                .unwrap()
        } else if phase_config.is_public {
            0
        } else {
            phase_config.max_nfts_per_address
        };

        let mintable = phase_config.max_nfts_per_address - minted_nfts;

        mintable_response.push(MintableResponse {
            phase_id: phase_config.phase_id,
            remaining_nfts: mintable,
        });
    }

    Ok(mintable_response)
}
