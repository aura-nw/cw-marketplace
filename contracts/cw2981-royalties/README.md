# CW-2981 Contract-level Royalties

This contract is based on cw2981 implementation of [cw-nfts](https://github.com/CosmWasm/cw-nfts) with modification for contract-level royalties instead of token-level royalties.

Builds on top of the metadata pattern in `cw721-metadata-onchain`.

All of the CW-721 logic and behaviour you would expect for an NFT is implemented as normal, but additionally at mint time, royalty information can be attached to a token.

Exposes two new query message types that can be called:

```rust
// Should be called on sale to see if royalties are owed
// by the marketplace selling the NFT.
// See https://eips.ethereum.org/EIPS/eip-2981
RoyaltyInfo {
    token_id: String,j
    // the denom of this sale must also be the denom returned by RoyaltiesInfoResponse
    sale_price: Uint128,
},
// Called against the contract to signal that CW-2981 is implemented
CheckRoyalties {},
```

The responses are:

```rust
#[cw_serde]
pub struct RoyaltiesInfoResponse {
    pub address: String,
    // Note that this must be the same denom as that passed in to RoyaltyInfo
    // rounding up or down is at the discretion of the implementer
    pub royalty_amount: Uint128,
}

/// Shows if the contract implements royalties
/// if royalty_payments is true, marketplaces should pay them
#[cw_serde]
pub struct CheckRoyaltiesResponse {
    pub royalty_payments: bool,
}
```

Royalties information need to be set at contract instantiation:
```rust
#[cw_serde]
pub struct InstantiateMsg {
    pub name: String,
    pub symbol: String,
    pub minter: String,
    pub royalty_percentage: Option<u64>,
    pub royalty_payment_address: Option<String>,
}
```

For compatibility with the implementation of *cw-nfts*, we decided to write the royalties information to token medatata when minting. However, attempt to overwrite it when minting will result in error.

Note that the `royalty_payment_address` could of course be a single address, a multisig, or a DAO.
