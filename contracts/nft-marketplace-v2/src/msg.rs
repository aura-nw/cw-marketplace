use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::structs::{AuctionConfig, NftAsset, PaymentAsset};

/// Message type for `instantiate` entry_point
#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    // List a NFT for sale
    ListNft {
        asset: NftAsset,
        auction_config: AuctionConfig,
    },
    // Buy a listed NFT
    Buy {
        asset: NftAsset,
    },
    // Cancel a listed NFT
    Cancel {
        asset: NftAsset,
    },
    // Offer a Nft
    OfferNft {
        asset: PaymentAsset,
        auction_config: AuctionConfig,
    },
    // Accept a Nft offer
    AcceptNftOffer {
        offerer: String,
        nft: NftAsset,
        funds_amount: u128,
    },
    // Cancel offer of User
    CancelOffer {
        nfts: Vec<NftAsset>,
    },
    // // edit contract address of vaura token
    // EditVauraToken {
    //     token_address: String,
    // },
}

/// Message type for `migrate` entry_point
#[cw_serde]
pub enum MigrateMsg {}

/// Message type for `query` entry_point
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // This example query variant indicates that any client can query the contract
    // using `YourQuery` and it will return `YourQueryResponse`
    // This `returns` information will be included in contract's schema
    // which is used for client code generation.
    //
    // #[returns(YourQueryResponse)]
    // YourQuery {},
}

// We define a custom struct for each query response
// #[cw_serde]
// pub struct YourQueryResponse {}
