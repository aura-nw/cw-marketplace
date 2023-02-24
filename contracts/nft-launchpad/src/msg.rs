use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::state::{LaunchpadInfo, PhaseConfigResponse, PhaseData};

/// Message type for `instantiate` entry_point
#[cw_serde]
pub struct InstantiateMsg {
    pub random_seed: String,
    pub colection_code_id: u64,
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
    AddWhitelist {
        phase_id: u64,
        whitelist: Vec<String>,
    },
    RemoveWhitelist {
        phase_id: u64,
        whitelist: Vec<String>,
    },
    Mint {
        phase_id: u64,
    },
    DeactivateLaunchpad {},
    ActivateLaunchpad {},
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
}

#[cw_serde]
pub struct ColectionInfo {
    pub name: String,
    pub symbol: String,
    pub max_supply: u64,
    pub uri_prefix: String,
    pub royalty_percentage: Option<u64>,
    pub royalty_payment_address: Option<String>,
}
