use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;
use cw721::Expiration;

use crate::{
    order_state::{OrderComponents, NFT},
    state::{AuctionConfig, Listing},
};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Addr,
}

#[cw_serde]
pub enum ExecuteMsg {
    // List a NFT for sale
    ListNft {
        contract_address: String,
        token_id: String,
        auction_config: AuctionConfig,
    },
    // Buy a listed NFT
    Buy {
        contract_address: String,
        token_id: String,
    },
    // Cancel a listed NFT
    Cancel {
        contract_address: String,
        token_id: String,
    },
    // Offer a Nft
    OfferNft {
        nft: NFT,
        funds_amount: u128,
        end_time: Expiration,
    },
    // Accept a Nft offer
    AcceptNftOffer {
        offerer: String,
        nft: NFT,
        funds_amount: u128,
    },
    // Cancel offer of User
    CancelOffer {
        nfts: Vec<NFT>,
    },
    // edit contract address of vaura token
    EditVauraToken {
        token_address: String,
    },
    // user auction nft
    AuctionNft {
        nft: NFT,
        auction_config: AuctionConfig,
    },
    // user bid nft
    BidNft {
        offerer: String,
        nft: NFT,
        bid_price: u128,
    },
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // list config of contract
    #[returns(crate::state::Config)]
    Config {},
    // get listing by contract_address
    #[returns(ListingsResponse)]
    ListingsByContractAddress {
        contract_address: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    // get listing by contract_address and token_id
    #[returns(Listing)]
    Listing {
        contract_address: String,
        token_id: String,
    },
    // get the specific offer
    #[returns(OrderComponents)]
    Offer {
        contract_address: String,
        token_id: String,
        offerer: String,
    },
    // get all offers of a nft
    #[returns(OffersResponse)]
    NftOffers {
        contract_address: String,
        token_id: String,
        start_after_offerer: Option<String>,
        limit: Option<u32>,
    },
    // get all offers of a user
    #[returns(OffersResponse)]
    UserOffers {
        offerer: String,
        start_after_nft: Option<NFT>,
        limit: Option<u32>,
    },
}

#[cw_serde]
pub struct ListingsResponse {
    pub listings: Vec<Listing>,
}

#[cw_serde]
pub struct ValidateResponse {
    pub valid: bool,
}

#[cw_serde]
pub struct OffersResponse {
    pub offers: Vec<OrderComponents>,
}
