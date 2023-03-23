pub mod execute;
pub mod msg;
pub mod query;

#[cfg(test)]
pub mod test;

use execute::distribute_nfts;
use msg::Cw2981ExecuteMsg;
pub use query::{all_nft_info, check_royalties, nft_info, query_royalties_info};

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{to_binary, Empty, StdError};
use cw2::set_contract_version;
use cw721_base::Cw721Contract;
pub use cw721_base::{
    ContractError, InstantiateMsg as Cw721InstantiateMsg, MintMsg, MinterResponse,
};
use cw_storage_plus::Item;

use crate::msg::{Cw2981QueryMsg, InstantiateMsg};

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

// Version info for migration
const CONTRACT_NAME: &str = "crates.io:cw2981-royalties";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cw_serde]
pub struct Trait {
    pub display_type: Option<String>,
    pub trait_type: String,
    pub value: String,
}

// see: https://docs.opensea.io/docs/metadata-standards
#[cw_serde]
#[derive(Default)]
pub struct Metadata {
    pub image: Option<String>,
    pub image_data: Option<String>,
    pub external_url: Option<String>,
    pub description: Option<String>,
    pub name: Option<String>,
    pub attributes: Option<Vec<Trait>>,
    pub background_color: Option<String>,
    pub animation_url: Option<String>,
    pub youtube_url: Option<String>,
    /// This is how much the minter takes as a cut when sold
    /// royalties are owed on this token if it is Some
    pub royalty_percentage: Option<u64>,
    /// The payment address, may be different to or the same
    /// as the minter addr
    /// question: how do we validate this?
    pub royalty_payment_address: Option<String>,
}

#[cw_serde]
pub struct ProvenanceInfo {
    pub final_proof: String,
    pub elements_proof: String,
    pub token_uri_anchor: u32,
    pub distinct_elements_number: u32,
}

#[cw_serde]
#[derive(Default)]
pub struct Config {
    pub royalty_percentage: Option<u64>,
    pub royalty_payment_address: Option<String>,
    pub provenance: Option<ProvenanceInfo>, // we add some extra fields here for the proofs of provenance minting
}

pub const CONFIG: Item<Config> = Item::new("config");

pub type Extension = Option<Metadata>;

pub type MintExtension = Option<Extension>;

pub type Cw2981Contract<'a> = Cw721Contract<'a, Extension, Empty, Cw2981ExecuteMsg, Cw2981QueryMsg>;
pub type ExecuteMsg = cw721_base::ExecuteMsg<Extension, Cw2981ExecuteMsg>;
pub type QueryMsg = cw721_base::QueryMsg<Cw2981QueryMsg>;

use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // create InstantiateMsg for cw721-base
    let msg_721 = Cw721InstantiateMsg {
        name: msg.name,
        symbol: msg.symbol,
        minter: msg.minter,
    };
    let res = Cw2981Contract::default().instantiate(deps.branch(), env, info, msg_721)?;
    // Explicitly set contract name and version, otherwise set to cw721-base info
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)
        .map_err(ContractError::Std)?;

    // validate royalty_percentage to be between 0 and 100
    if let Some(royalty_percentage) = msg.royalty_percentage {
        if royalty_percentage > 100 {
            return Err(ContractError::Std(StdError::generic_err(
                "Royalty percentage cannot be greater than 100",
            )));
        }
    }

    let provenance = msg.final_proof.map(|provenance_info| ProvenanceInfo {
        final_proof: provenance_info,
        elements_proof: "".to_string(), // the proof of all elements will be provided later when distributing the NFTs
        token_uri_anchor: 0,
        distinct_elements_number: 0, // the anchor will be provided later when distributing the NFTs
    });

    // set royalty_percentage and royalty_payment_address
    CONFIG.save(
        deps.storage,
        &Config {
            royalty_percentage: msg.royalty_percentage,
            royalty_payment_address: msg.royalty_payment_address,
            provenance,
        },
    )?;

    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    // match msg if it is mint message
    match msg {
        ExecuteMsg::Mint(msg) => {
            let mut extension = msg.extension.clone().unwrap_or_default();

            // return error if royalty is set
            if extension.royalty_percentage.is_some() || extension.royalty_payment_address.is_some()
            {
                return Err(ContractError::Std(StdError::generic_err(
                    "Cannot set royalty information in mint message",
                )));
            }

            let config = CONFIG.load(deps.storage)?;

            extension.royalty_percentage = config.royalty_percentage;
            extension.royalty_payment_address = config.royalty_payment_address;

            let msg_with_royalty = MintMsg {
                extension: Some(extension),
                ..msg
            };

            Cw2981Contract::default().execute(
                deps,
                env,
                info,
                cw721_base::ExecuteMsg::Mint(msg_with_royalty),
            )
        }
        ExecuteMsg::Extension { msg } => match msg {
            Cw2981ExecuteMsg::DistributeNfts {
                elements_proof,
                token_uri_anchor,
                distinct_elements_number,
            } => distribute_nfts(
                deps,
                env,
                info,
                elements_proof,
                token_uri_anchor,
                distinct_elements_number,
            ),
        },
        _ => Cw2981Contract::default().execute(deps, env, info, msg),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Extension { msg } => match msg {
            Cw2981QueryMsg::RoyaltyInfo {
                token_id,
                sale_price,
            } => to_binary(&query_royalties_info(deps, token_id, sale_price)?),
            Cw2981QueryMsg::CheckRoyalties {} => to_binary(&check_royalties(deps)?),
        },
        // we will override the default query token info to include the token_uri_anchor
        QueryMsg::NftInfo { token_id } => to_binary(&nft_info(deps, token_id)?),
        QueryMsg::AllNftInfo {
            token_id,
            include_expired,
        } => to_binary(&all_nft_info(
            deps,
            env,
            token_id,
            include_expired.unwrap_or(false),
        )?),
        _ => Cw2981Contract::default().query(deps, env, msg),
    }
}
