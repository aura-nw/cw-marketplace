use std::vec;

use crate::msg::MintableResponse;
use crate::state::{LaunchpadInfo, PhaseConfigResponse, LAUNCHPAD_INFO, PHASE_CONFIGS, WHITELIST};
use cosmwasm_std::{Addr, Deps, StdResult};

pub fn query_launchpad_info(deps: Deps) -> StdResult<LaunchpadInfo> {
    let launchpad_info = LAUNCHPAD_INFO.load(deps.storage)?;
    Ok(launchpad_info)
}

pub fn query_all_phase_configs(deps: Deps) -> StdResult<Vec<PhaseConfigResponse>> {
    // load the last_phase_id
    let last_phase_id = LAUNCHPAD_INFO.load(deps.storage).unwrap().last_phase_id;

    let mut phase_id = 0;

    // get the dummy phase config
    let mut phase_config = PHASE_CONFIGS.load(deps.storage, phase_id).unwrap();

    // create an empty PHASE_CONFIGS_RESPONSE
    let mut phase_configs_response: Vec<PhaseConfigResponse> = vec![];

    // begin from phase_id 0, loop through all the phase_configs,
    // until the phase_id is different from the last_phase_id
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

    Ok(phase_configs_response)
}

pub fn query_mintable(deps: Deps, user: Addr) -> StdResult<Vec<MintableResponse>> {
    // load the last_phase_id
    let last_phase_id = LAUNCHPAD_INFO.load(deps.storage).unwrap().last_phase_id;

    let mut phase_id = 0;

    // get the dummy phase config
    let mut phase_config = PHASE_CONFIGS.load(deps.storage, phase_id).unwrap();

    // create an empty mintable response vector
    let mut mintable_response: Vec<MintableResponse> = vec![];

    // begin from phase_id 0, loop through all the phase_configs,
    // until the phase_id is different from the last_phase_id
    while phase_id != last_phase_id {
        // get the next phase id
        phase_id = phase_config.next_phase_id.unwrap();

        // get the next phase config
        phase_config = PHASE_CONFIGS.load(deps.storage, phase_id).unwrap();

        // load the number of minted nfts of the user from WHITELIST base of the phase_id and the address of user
        let minted_nfts = if WHITELIST.has(deps.storage, (phase_id, user.clone())) {
            WHITELIST
                .load(deps.storage, (phase_id, user.clone()))
                .unwrap()
        } else if phase_config.is_public {
            0
        } else {
            phase_config.max_nfts_per_address
        };

        let mintable = phase_config.max_nfts_per_address - minted_nfts;

        mintable_response.push(MintableResponse {
            phase_id,
            remaining_nfts: mintable,
        });
    }

    Ok(mintable_response)
}
