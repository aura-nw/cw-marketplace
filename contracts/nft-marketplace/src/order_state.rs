use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, BlockInfo};
use cw721::Expiration;
use cw_storage_plus::{Index, IndexList, IndexedMap, MultiIndex};

pub type Nft = (Addr, String);
pub type User = Addr;

#[cw_serde]
pub enum OrderType {
    OFFER,
    LISTING,
    AUCTION,
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
pub enum Side {
    OFFER,
    CONSIDERATION,
}

#[cw_serde]
pub struct OfferItem {
    pub item: Asset,
}

pub fn offer_item(item: &Asset) -> OfferItem {
    OfferItem { item: item.clone() }
}

#[cw_serde]
pub struct ConsiderationItem {
    pub item: Asset,
    pub start_amount: u128,
    pub step_amount: Option<u8>,
    pub end_amount: Option<u128>,
    pub recipient: Addr,
}

pub fn consideration_item(
    item: &Asset,
    start_amount: &u128,
    step_amount: &Option<u8>,
    end_amount: &Option<u128>,
    recipient: &Addr,
) -> ConsiderationItem {
    ConsiderationItem {
        item: item.clone(),
        start_amount: *start_amount,
        step_amount: *step_amount,
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
    pub order_type: OrderType,
    pub order_id: OrderKey,
    pub offerer: User,
    pub recipient: Option<User>,
    pub offer: Vec<OfferItem>,
    pub consideration: Vec<ConsiderationItem>,
    pub start_time: Option<Expiration>,
    pub end_time: Option<Expiration>,
}

impl OrderComponents {
    // expired is when a listing has passed the end_time
    pub fn is_expired(&self, block_info: &BlockInfo) -> bool {
        match self.end_time {
            Some(end_time) => end_time.is_expired(block_info),
            None => false,
        }
    }
}

pub struct OfferIndexes<'a> {
    pub users: MultiIndex<'a, User, OrderComponents, OrderKey>,
    pub nfts: MultiIndex<'a, (Addr, String), OrderComponents, OrderKey>,
    pub buyer: Option<MultiIndex<'a, User, OrderComponents, OrderKey>>,
}

impl<'a> IndexList<OrderComponents> for OfferIndexes<'a> {
    // this method returns a list of all indexes
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<OrderComponents>> + '_> {
        let v: Vec<&dyn Index<OrderComponents>> = vec![&self.users, &self.nfts];
        Box::new(v.into_iter())
    }
}

// helper function create a IndexedMap for listings
pub fn orders<'a>() -> IndexedMap<'a, OrderKey, OrderComponents, OfferIndexes<'a>> {
    let indexes = OfferIndexes {
        users: MultiIndex::new(
            |_pk: &[u8], l: &OrderComponents| (l.order_id.0.clone()),
            "orders",
            "orders__user_address",
        ),
        nfts: MultiIndex::new(
            |_pk: &[u8], l: &OrderComponents| (l.order_id.1.clone(), l.order_id.2.clone()),
            "orders",
            "orders__nft_identifier",
        ),
        // ignore the buyer index for now
        buyer: None,
    };
    IndexedMap::new("orders", indexes)
}

// helper function create a IndexedMap for listings
pub fn auctions<'a>() -> IndexedMap<'a, OrderKey, OrderComponents, OfferIndexes<'a>> {
    let indexes = OfferIndexes {
        users: MultiIndex::new(
            |_pk: &[u8], l: &OrderComponents| (l.order_id.0.clone()),
            "orders",
            "orders__user_address",
        ),
        nfts: MultiIndex::new(
            |_pk: &[u8], l: &OrderComponents| (l.order_id.1.clone(), l.order_id.2.clone()),
            "orders",
            "orders__nft_identifier",
        ),
        buyer: Some(MultiIndex::new(
            |_pk: &[u8], l: &OrderComponents| (l.recipient.clone().unwrap()),
            "orders",
            "orders__buyer",
        )),
    };
    IndexedMap::new("orders", indexes)
}
