use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdError};
use cw721_base::ContractError;

use crate::{Config, Cw2981Contract, ProvenanceInfo, CONFIG};

pub fn distribute_nfts(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    elements_proof: String,
) -> Result<Response, ContractError> {
    // load minter data from Cw2981Contract::default()
    let minter = Cw2981Contract::default().minter.load(deps.storage)?;

    // Only the minter can distribute nfts
    if info.sender != minter {
        return Err(ContractError::Unauthorized {});
    }

    // load config data from CONFIG
    let config = CONFIG.load(deps.storage)?;

    // IF the provenance is none
    // OR the provenance is some but the final_proof is empty
    // OR the provenance is some and the token_uri_anchor is greater than 0
    // then return error
    if config.provenance.is_none() // this collection does not have provenance information
        || (config.provenance.is_some()
                // the final proof is invalid
            && (config.provenance.clone().unwrap().final_proof.is_empty()
                // this function's been called before
                || !config.provenance.clone().unwrap().elements_proof.is_empty()))
    {
        return Err(ContractError::Std(StdError::generic_err(
            "Re-distibuting NFTs is not neccessary",
        )));
    }

    // TODO: Maybe we will random following value later
    // OR the below function will activate another random mechanism to get the token_uri_anchor
    let request_token_uri_anchor = request_token_uri_anchor();

    let provenance = ProvenanceInfo {
        final_proof: config.provenance.unwrap().final_proof,
        elements_proof: elements_proof.clone(),
        token_uri_anchor: request_token_uri_anchor,
    };
    CONFIG.save(
        deps.storage,
        &Config {
            provenance: Some(provenance),
            ..config
        },
    )?;

    Ok(Response::new().add_attributes([
        ("action", "distribute_nfts"),
        ("elements_proof", &elements_proof),
    ]))
}

pub fn request_token_uri_anchor() -> u32 {
    // TODO: implement the random mechanism to get the token_uri_anchor
    0
}
