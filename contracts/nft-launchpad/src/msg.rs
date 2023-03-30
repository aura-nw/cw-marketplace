use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::state::{LaunchpadInfo, PhaseConfigResponse, PhaseData};

/// Message type for `instantiate` entry_point
#[cw_serde]
pub struct InstantiateMsg {
    pub random_seed: String,
    pub colection_code_id: u64,
    pub launchpad_fee: u32,
    pub launchpad_collector: Option<String>,
    pub collection_info: ColectionInfo,
}

/// Message type for `execute` entry_point
#[cw_serde]
pub enum ExecuteMsg {
    AddMintPhase {
        after_phase_id: Option<u64>,
        phase_data: PhaseData,
    },
    UpdateMintPhase {
        phase_id: u64,
        phase_data: PhaseData,
    },
    RemoveMintPhase {
        phase_id: u64,
    },
    // TODO: should implement Merkle Airdop instead of whitelist
    AddWhitelist {
        phase_id: u64,
        whitelists: Vec<String>,
    },
    RemoveWhitelist {
        phase_id: u64,
        addresses: Vec<String>,
    },
    DeactivateLaunchpad {},
    ActivateLaunchpad {},
    Mint {
        phase_id: u64,
        amount: Option<u64>,
    },
    Withdraw {
        denom: String,
    },
}

/// Message type for `migrate` entry_point
#[cw_serde]
pub enum MigrateMsg {}

/// Message type for `query` entry_point
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(LaunchpadInfo)]
    GetLaunchpadInfo {},
    #[returns(Vec<PhaseConfigResponse>)]
    GetAllPhaseConfigs {},
    #[returns(Vec<MintableResponse>)]
    Mintable { user: String },
}

#[cw_serde]
pub struct MintableResponse {
    pub phase_id: u64,
    pub remaining_nfts: u64,
}

#[cw_serde]
pub struct ColectionInfo {
    pub creator: String,
    pub name: String,
    pub symbol: String,
    pub max_supply: u64,
    pub uri_prefix: String,
    pub uri_suffix: String,
    pub royalty_percentage: Option<u64>,
    pub royalty_payment_address: Option<String>,
    pub reserved_tokens: Option<Vec<u64>>,
    pub token_id_offset: Option<u64>,
}
