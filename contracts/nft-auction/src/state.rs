use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, BlockInfo, Coin};
use cw721::Expiration;
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, MultiIndex};

#[cw_serde]
pub struct Config {
    pub owner: Addr,
}

// New enum data structure for AuctionConfig input only
#[cw_serde]
pub enum AuctionConfigInput {
    EnglishAuction {
        start_price: Coin,           // require start_price to determine the denom
        step_percentage: Option<u8>, // step_percentage is a percentage of the current price
        buyout_price: Option<u128>,  // buyout_price is the wish price amount of the seller
        start_time: Option<Expiration>,
        end_time: Expiration,
    },
}

#[cw_serde]
pub struct EnglishAuctionConfig {
    start_price: Coin,
    step_percentage: u8,
    buyout_price: u128,
    start_time: Expiration,
    end_time: Expiration,
}

#[cw_serde]
pub struct NFT {
    pub contract_address: Addr,
    pub token_id: Option<String>,
}

#[cw_serde]
pub struct CW20 {
    pub contract_address: Addr,
    pub amount: u128,
}

#[cw_serde]
pub struct NATIVE {
    pub denom: String,
    pub amount: u128,
}

#[cw_serde]
pub enum Asset {
    Nft(NFT),
    Native(NATIVE),
    Cw20(CW20),
}

#[cw_serde]
pub enum PaymentAsset {
    Native {
        denom: String,
        amount: u128,
    },
    Cw20 {
        contract_address: Addr,
        amount: u128,
    },
}

impl From<Asset> for PaymentAsset {
    fn from(asset: Asset) -> Self {
        match asset {
            Asset::Native(NATIVE { denom, amount }) => PaymentAsset::Native { denom, amount },
            Asset::Cw20(CW20 {
                contract_address,
                amount,
            }) => PaymentAsset::Cw20 {
                contract_address,
                amount,
            },
            _ => panic!("Asset is not a payment asset"),
        }
    }
}

#[cw_serde]
pub struct Offer {
    pub item: Asset,
    pub start_amount: u128,
    pub end_amount: u128,
    pub offerer: Addr,
}

pub fn offer_item(item: &Asset, start_amount: &u128, end_amount: &u128, offerer: &Addr) -> Offer {
    Offer {
        item: item.clone(),
        start_amount: *start_amount,
        end_amount: *end_amount,
        offerer: offerer.clone(),
    }
}

#[cw_serde]
pub struct Consideration {
    pub item: Asset,
    pub start_amount: u128,
    pub end_amount: u128,
    pub recipient: Addr,
}

pub fn consideration_item(
    item: &Asset,
    start_amount: &u128,
    end_amount: &u128,
    recipient: &Addr,
) -> Consideration {
    Consideration {
        item: item.clone(),
        start_amount: *start_amount,
        end_amount: *end_amount,
        recipient: recipient.clone(),
    }
}

// the OrderKey includes the address and id of NFT
// !DO NOT change the order of the fields
pub type OrderKey = (Addr, Addr, String);

pub fn order_key(user_address: &Addr, contract_address: &Addr, token_id: &str) -> OrderKey {
    (
        user_address.clone(),
        contract_address.clone(),
        token_id.to_owned(),
    )
}

#[cw_serde]
pub struct OrderComponents {
    pub order_id: OrderKey,
    pub offer: Vec<Offer>,
    pub consideration: Vec<Consideration>,
    pub start_time: Expiration,
    pub end_time: Expiration,
    pub config: String,
}

impl OrderComponents {
    // expired is when a listing has passed the end_time
    pub fn is_expired(&self, block_info: &BlockInfo) -> bool {
        self.end_time.is_expired(block_info)
    }
}

pub struct AuctionIndexes<'a> {
    pub owners: MultiIndex<'a, Addr, OrderComponents, OrderKey>,
    pub nfts: MultiIndex<'a, (Addr, String), OrderComponents, OrderKey>,
    pub buyers: MultiIndex<'a, Addr, OrderComponents, OrderKey>,
}

impl<'a> IndexList<OrderComponents> for AuctionIndexes<'a> {
    // this method returns a list of all indexes
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<OrderComponents>> + '_> {
        let v: Vec<&dyn Index<OrderComponents>> = vec![&self.owners, &self.nfts, &self.buyers];
        Box::new(v.into_iter())
    }
}

// helper function create a IndexedMap for listings
pub fn auctions<'a>() -> IndexedMap<'a, OrderKey, OrderComponents, AuctionIndexes<'a>> {
    let indexes = AuctionIndexes {
        owners: MultiIndex::new(
            |_pk: &[u8], l: &OrderComponents| (l.offer[0].offerer.clone()),
            "auctions",
            "auctions__owner_address",
        ),
        nfts: MultiIndex::new(
            |_pk: &[u8], l: &OrderComponents| (l.order_id.1.clone(), l.order_id.2.clone()),
            "auctions",
            "auctions__nft_identifier",
        ),
        buyers: MultiIndex::new(
            |_pk: &[u8], l: &OrderComponents| (l.consideration[0].recipient.clone()),
            "auctions",
            "auctions__buyer_address",
        ),
    };
    IndexedMap::new("auctions", indexes)
}

pub struct AuctionContract<'a> {
    pub auctions: IndexedMap<'a, OrderKey, OrderComponents, AuctionIndexes<'a>>,
}

// impl default for MarketplaceContract
impl Default for AuctionContract<'static> {
    fn default() -> Self {
        AuctionContract {
            auctions: auctions(),
        }
    }
}

// public the default MarketplaceContract
pub fn contract() -> AuctionContract<'static> {
    AuctionContract::default()
}

pub const CONFIG: Item<Config> = Item::new("config");
