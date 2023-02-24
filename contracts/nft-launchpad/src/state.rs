use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub owner: Addr,
}

#[cw_serde]
pub struct PhaseData {
    // user must specify phase_id when adding a new phase,
    // this parameter is used to identify the phase in mapping
    pub start_time: Timestamp,
    pub end_time: Timestamp,
    pub max_supply: Option<u64>,
    pub max_nfts_per_address: u64,
    pub price: u128,
    pub is_public: bool,
}

#[cw_serde]
pub struct PhaseConfig {
    pub previous_phase_id: Option<u64>,
    pub next_phase_id: Option<u64>,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
    pub max_supply: Option<u64>,
    pub total_supply: u64,
    pub max_nfts_per_address: u64,
    pub price: u128,
    pub is_public: bool,
}

#[cw_serde]
pub struct PhaseConfigResponse {
    pub phase_id: u64,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
    pub max_supply: Option<u64>,
    pub total_supply: u64,
    pub max_nfts_per_address: u64,
    pub price: u128,
}

#[cw_serde]
pub struct LaunchpadInfo {
    pub collection_address: Addr,
    pub total_supply: u64,
    pub max_supply: u64,
    pub uri_prefix: String,
    pub first_phase_id: u64,
    pub last_phase_id: u64,
    pub last_issued_id: u64, // for the unique id of phases
    pub is_active: bool,     // maybe we want to pause the launchpad for some reason
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const LAUNCHPAD_INFO: Item<LaunchpadInfo> = Item::new("launchpad_info");
pub const PHASE_CONFIGS: Map<u64, PhaseConfig> = Map::new("phase_configs");

// we will create a Item to store the list of phases for easier query
// this state will be updated when adding a new phase or removing a phase
pub const PHASE_CONFIGS_RESPONSE: Item<Vec<PhaseConfigResponse>> =
    Item::new("phase_configs_response");

// The whitelist !!! key = (phase_id, user_address), value = number of minted_nft in phase_id
pub const WHITELIST: Map<(u64, Addr), u64> = Map::new("whitelist");

// TODO: Implement this to controll random minting
// The length of the token_ids will be the same as the max_supply of the launchpad
// The remaining token_ids
pub const REMAINING_TOKEN_IDS: Item<Vec<u64>> = Item::new("remaining_token_ids");

pub const RANDOM_SEED: Item<[u8; 32]> = Item::new("random_seed");
