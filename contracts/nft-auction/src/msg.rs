use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

use crate::state::{AuctionConfigInput, OrderComponents, NFT};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Addr,
}

#[cw_serde]
pub enum ExecuteMsg {
    // user auction nft
    AuctionNft {
        nft: NFT,
        auction_config: AuctionConfigInput,
    },
    // user bid nft
    BidAuction {
        nft: NFT,
        bid_price: u128,
    },
    // terminate auction
    SettleAuction {
        nft: NFT,
    },
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // get auction of a nft
    #[returns(OrderComponents)]
    NftAuction {
        contract_address: String,
        token_id: String,
    },
    // get all auctions of owner
    #[returns(AuctionsResponse)]
    OwnerAuctions {
        owner: String,
        start_after_nft: Option<NFT>,
        limit: Option<u32>,
    },
    // get all auctions of a buyer
    #[returns(AuctionsResponse)]
    BuyerAuctions {
        buyer: String,
        start_after_nft: Option<NFT>,
        limit: Option<u32>,
    },
    // get a valid price for bidding of a nft
    #[returns(u128)]
    ValidPrice {
        contract_address: String,
        token_id: String,
    },
}

#[cw_serde]
pub struct AuctionsResponse {
    pub auctions: Vec<OrderComponents>,
}
