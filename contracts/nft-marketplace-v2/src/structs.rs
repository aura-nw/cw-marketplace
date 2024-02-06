use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, BlockInfo};
use cw721::Expiration;

#[cw_serde]
pub enum AuctionConfig {
    FixedPrice {
        price: PaymentAsset,
        start_time: Option<Expiration>, // we use expiration for convinience
        end_time: Option<Expiration>,   // it's required that start_time < end_time
    },
    OfferPrice {
        price: NftAsset,
        start_time: Option<Expiration>,
        end_time: Option<Expiration>,
    },
}

impl AuctionConfig {
    pub fn is_valid(&self) -> bool {
        match &self {
            AuctionConfig::FixedPrice {
                price: _,
                start_time,
                end_time,
            } => {
                // if price.amount.is_zero() {
                //     // since price is Uint128, it cannot be negative, we only
                //     // need to check if it's zero
                //     return false;
                // }
                // if start_time or end_time is not set, we don't need to check
                if start_time.is_some()
                    && end_time.is_some()
                    && start_time.unwrap() >= end_time.unwrap()
                {
                    return false;
                }
                true
            }
            AuctionConfig::OfferPrice {
                price: _,
                start_time,
                end_time,
            } => {
                // if start_time or end_time is not set, we don't need to check
                if start_time.is_some()
                    && end_time.is_some()
                    && start_time.unwrap() >= end_time.unwrap()
                {
                    return false;
                }
                true
            }
        }
    }
}

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub vaura_address: Addr,
}

#[cw_serde]
pub enum OrderType {
    OFFER,
    LISTING,
}

#[cw_serde]
pub enum Asset {
    Nft(NftAsset),
    Native(NativeAsset),
    Cw20(Cw20Asset),
}

impl From<PaymentAsset> for Asset {
    fn from(asset: PaymentAsset) -> Self {
        match asset {
            PaymentAsset::Native { denom, amount } => Asset::Native(NativeAsset { denom, amount }),
            PaymentAsset::Cw20 {
                contract_address,
                amount,
            } => Asset::Cw20(Cw20Asset {
                contract_address,
                amount,
            }),
        }
    }
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
            Asset::Native(NativeAsset { denom, amount }) => PaymentAsset::Native { denom, amount },
            Asset::Cw20(Cw20Asset {
                contract_address,
                amount,
            }) => PaymentAsset::Cw20 {
                contract_address,
                amount,
            },
            _ => panic!("Invalid payment asset"),
        }
    }
}

#[cw_serde]
pub struct NftAsset {
    pub contract_address: Addr,
    pub token_id: Option<String>,
}

#[cw_serde]
pub struct Cw20Asset {
    pub contract_address: Addr,
    pub amount: u128,
}

#[cw_serde]
pub struct NativeAsset {
    pub denom: String,
    pub amount: u128,
}

#[cw_serde]
pub enum Side {
    OFFER,
    CONSIDERATION,
}

#[cw_serde]
pub enum ItemType {
    NATIVE,
    CW20,
    CW721,
}

#[cw_serde]
pub struct OfferItem {
    pub item_type: ItemType,
    pub item: Asset,
    pub start_amount: u128,
    pub end_amount: u128,
}

pub fn offer_item(
    item_type: &ItemType,
    item: &Asset,
    start_amount: &u128,
    end_amount: &u128,
) -> OfferItem {
    OfferItem {
        item_type: item_type.clone(),
        item: item.clone(),
        start_amount: *start_amount,
        end_amount: *end_amount,
    }
}

#[cw_serde]
pub struct ConsiderationItem {
    pub item_type: ItemType,
    pub item: Asset,
    pub start_amount: u128,
    pub end_amount: u128,
    pub recipient: Addr,
}

pub fn consideration_item(
    item_type: &ItemType,
    item: &Asset,
    start_amount: &u128,
    end_amount: &u128,
    recipient: &Addr,
) -> ConsiderationItem {
    ConsiderationItem {
        item_type: item_type.clone(),
        item: item.clone(),
        start_amount: *start_amount,
        end_amount: *end_amount,
        recipient: recipient.clone(),
    }
}

pub type Nft = (Addr, String);
pub type User = Addr;

// the OrderKey includes the address of user, contract address and id of NFT
// !DO NOT change the order of the fields
pub type OfferID = (Addr, Addr, String);
pub fn order_id(user_address: &Addr, contract_address: &Addr, token_id: &str) -> OfferID {
    (
        user_address.clone(),
        contract_address.clone(),
        token_id.to_owned(),
    )
}

#[cw_serde]
pub struct Order {
    pub order_type: OrderType,
    pub order_id: OfferID,
    pub owner: User,
    pub offer: Vec<OfferItem>,
    pub consideration: Vec<ConsiderationItem>,
    pub start_time: Option<Expiration>,
    pub end_time: Option<Expiration>,
}

impl Order {
    // expired is when a listing has passed the end_time
    pub fn is_expired(&self, block_info: &BlockInfo) -> bool {
        match self.end_time {
            Some(time) => time.is_expired(block_info),
            None => false,
        }
    }
}
