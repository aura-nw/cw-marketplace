use crate::msg::{CheckRoyaltiesResponse, RoyaltiesInfoResponse};
use crate::{Cw2981Contract, Metadata, CONFIG};
use cosmwasm_std::{BlockInfo, Decimal, Deps, Env, StdResult, Uint128};
use cw721::{AllNftInfoResponse, NftInfoResponse, OwnerOfResponse};
use cw721_base::state::{Approval, TokenInfo};

/// NOTE: default behaviour here is to round down
/// EIP2981 specifies that the rounding behaviour is at the discretion of the implementer
pub fn query_royalties_info(
    deps: Deps,
    token_id: String,
    sale_price: Uint128,
) -> StdResult<RoyaltiesInfoResponse> {
    let contract = Cw2981Contract::default();
    let token_info = contract.tokens.load(deps.storage, &token_id)?;

    let royalty_percentage = match token_info.extension {
        Some(ref ext) => match ext.royalty_percentage {
            Some(percentage) => Decimal::percent(percentage),
            None => Decimal::percent(0),
        },
        None => Decimal::percent(0),
    };
    let royalty_from_sale_price = sale_price * royalty_percentage;

    let royalty_address = match token_info.extension {
        Some(ext) => match ext.royalty_payment_address {
            Some(addr) => addr,
            None => String::from(""),
        },
        None => String::from(""),
    };

    Ok(RoyaltiesInfoResponse {
        address: royalty_address,
        royalty_amount: royalty_from_sale_price,
    })
}

/// As our default implementation here specifies royalties at token level
/// and not at contract level, it is therefore logically true that
/// on sale, every token managed by this contract should be checked
/// to see if royalties are owed, and to whom. If you are importing
/// this logic, you may want a custom implementation here
pub fn check_royalties(_deps: Deps) -> StdResult<CheckRoyaltiesResponse> {
    Ok(CheckRoyaltiesResponse {
        royalty_payments: true,
    })
}

/// we will overwrite the default nft_info in cw721_base implementation
/// Modify the token_uri to include the anchor
pub fn nft_info(deps: Deps, token_id: String) -> StdResult<NftInfoResponse<Option<Metadata>>> {
    let info = Cw2981Contract::default()
        .tokens
        .load(deps.storage, &token_id)?;

    Ok(NftInfoResponse {
        token_uri: generate_token_uri(deps, token_id, &info),
        extension: info.extension,
    })
}

/// we will overwrite the default all_nft_info in cw721_base implementation
/// Modify the token_uri to include the anchor
pub fn all_nft_info(
    deps: Deps,
    env: Env,
    token_id: String,
    include_expired: bool,
) -> StdResult<AllNftInfoResponse<Option<Metadata>>> {
    let info = Cw2981Contract::default()
        .tokens
        .load(deps.storage, &token_id)?;
    Ok(AllNftInfoResponse {
        access: OwnerOfResponse {
            owner: info.owner.to_string(),
            approvals: humanize_approvals(&env.block, &info, include_expired),
        },
        info: NftInfoResponse {
            token_uri: generate_token_uri(deps, token_id, &info),
            extension: info.extension,
        },
    })
}

// This function is private in the cw721_base implementation, so we need to copy it here
fn humanize_approvals<T>(
    block: &BlockInfo,
    info: &TokenInfo<T>,
    include_expired: bool,
) -> Vec<cw721::Approval> {
    info.approvals
        .iter()
        .filter(|apr| include_expired || !apr.is_expired(block))
        .map(humanize_approval)
        .collect()
}

// This function is private in the cw721_base implementation, so we need to copy it here
fn humanize_approval(approval: &Approval) -> cw721::Approval {
    cw721::Approval {
        spender: approval.spender.to_string(),
        expires: approval.expires,
    }
}

// This function is used to generate the token_uri using the anchor
fn generate_token_uri(
    _deps: Deps,
    token_id: String,
    info: &TokenInfo<Option<Metadata>>,
) -> Option<String> {
    // load provenance
    let provenance_info = CONFIG.load(_deps.storage).unwrap().provenance;

    // if provenance is not set, return the default token_uri
    if let Some(provenance) = provenance_info {
        // if provenance is set,
        // check the distributing NFTs is executed
        if provenance.token_uri_anchor != 0 {
            let token_id_u32: u32 = match token_id.trim().parse() {
                Ok(id) => id,
                Err(_) => return info.token_uri.clone(),
            };
            if let Some(uri) = &info.token_uri {
                // TODO: we need implement the anchor logic here.
                // It should be a rotating array with the size of max_supply

                Some(uri.replace(
                    "{token_id}",
                    &(token_id_u32 + provenance.token_uri_anchor).to_string(),
                ))
            } else {
                info.token_uri.clone()
            }
        } else {
            info.token_uri.clone()
        }
    } else {
        info.token_uri.clone()
    }
}
