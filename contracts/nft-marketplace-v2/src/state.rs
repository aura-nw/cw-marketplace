use cosmwasm_std::Addr;
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, MultiIndex};

use crate::structs::{Config, OfferID, Order, User};

pub struct OfferIndexes<'a> {
    pub users: MultiIndex<'a, User, Order, OfferID>,
    pub nfts: MultiIndex<'a, (Addr, String), Order, OfferID>,
}

impl<'a> IndexList<Order> for OfferIndexes<'a> {
    // this method returns a list of all indexes
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Order>> + '_> {
        let v: Vec<&dyn Index<Order>> = vec![&self.users, &self.nfts];
        Box::new(v.into_iter())
    }
}

// ListingKey is unique for all listings
pub type TokenId = String;
pub type ListingKey = (Addr, TokenId);

pub fn listing_key(contract_address: &Addr, token_id: &TokenId) -> ListingKey {
    (contract_address.clone(), token_id.clone())
}

// listings can be indexed by contract_address
// contract_address can point to multiple listings
pub struct ListingIndexes<'a> {
    pub contract_address: MultiIndex<'a, Addr, Order, ListingKey>,
    pub users: MultiIndex<'a, Addr, Order, ListingKey>,
}
impl<'a> IndexList<Order> for ListingIndexes<'a> {
    // this method returns a list of all indexes
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Order>> + '_> {
        let v: Vec<&dyn Index<Order>> = vec![&self.contract_address];
        Box::new(v.into_iter())
    }
}

// the OfferKey includes the address and id of NFT
// !DO NOT change the order of the fields
pub type OfferKey = (Addr, Addr, String);

pub fn offer_key(user_address: &Addr, contract_address: &Addr, token_id: &str) -> OfferKey {
    (
        user_address.clone(),
        contract_address.clone(),
        token_id.to_owned(),
    )
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const OFFERS: IndexedMap<OfferKey, Order, OfferIndexes> = IndexedMap::new(
    "offers",
    OfferIndexes {
        users: MultiIndex::new(
            |_pk: &[u8], l: &Order| (l.order_id.0.clone()),
            "offers",
            "offers__user_address",
        ),
        nfts: MultiIndex::new(
            |_pk: &[u8], l: &Order| (l.order_id.1.clone(), l.order_id.2.clone()),
            "offers",
            "offers__nft_identifier",
        ),
    },
);
pub const LISTINGS: IndexedMap<ListingKey, Order, ListingIndexes> = IndexedMap::new(
    "listings",
    ListingIndexes {
        contract_address: MultiIndex::new(
            |_pk: &[u8], l: &Order| (l.order_id.0.clone()),
            "listings",
            "listings__contract_address",
        ),
        users: MultiIndex::new(
            |_pk: &[u8], l: &Order| (l.owner.clone()),
            "listings",
            "listings__user_address",
        ),
    },
);
