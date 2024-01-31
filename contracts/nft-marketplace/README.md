# NFT Marketplace Contract

The marketplace supports buy and sell of NFTs of cw721-compatible contracts. It also takes royalties for sales as specified in [cw2981-royalties](https://github.com/CosmWasm/cw-nfts/tree/main/contracts/cw2981-royalties).

## Interface

### ExecuteMsg

#### List an NFT for sale

To list an NFT for sale, the contract will check:
- sender is owner of the NFT,
- the contract is approved to use the NFT,
- the listing configuration is correct.

Current supported configuration is: 
```rust
{
    "price": {
        "amount": number,
        "denom": SupportedDenom
    },
    "start_time": Option<Timestamp>,
    "end_time": Option<Timestamp>
}
```
We are currently only support the native asset of the deployed chain, but support for other tokens as well as cw20 is being developed.
Both `start_time` and `end_time` is optional. If `start_time` is presented, it must be after current block time.
If `end_time` is presented, it must be after `max(start_time, current_block_time)`.

Transaction message format:
```json
{
    "list_nft": {
        "contract_address": "the nft contract address",
        "token_id": "the nft token id",
        "auction_config": "the listing config"
    }
}
```

Noted that if the NFT is transferred to another address, its listing will be invalid.

#### Buy a listed NFT

To buy a listed NFT, simply call `buy_nft` message:
```json
{
    "buy_nft": {
        "contract_address": "the nft contract address",
        "token_id": "the nft token id"
    }
}
```

Depends on the required configuration, the contract will check for attached funds or use a `transfer_from` message to transfer asset from buyer to seller.

#### Cancel a listing

Owner of a listing can cancel that listing at any time. Transaction message format:

```json
{
    "cancel": {
        "contract_address": "the nft contract address",
        "token_id": "the nft token id"
    }
}
```

#### Offer to buy an NFT

Users can offer to buy NFTs from other users. We require offerers to use cw20 token to offer so that in the case the offer is accepted, the marketplace contract can automatically transfer both the NFT to the offerer and tokens to the NFT owner. For safety reasons, we do not lock offerer's token so there can be cases when an offer is accepted but the offerer doesn't have enough tokens which makes the offer invalid. We expect the marketplace frontend will take care of this case and notify NFT owners. At the moment, we use our [bidding-token](../bidding-token/README.md) for this feature.

Offer message format:
```json
{
    "offer_nft": {
        "nft": {
            "contract_address": "the nft contract address",
            "token_id": "the nft token id"
        },
        "funds_amount": 1234,
        "end_time": "expiration time"
    }
}
```

The expiration time is defined in [cw721](https://docs.rs/cw721/latest/cw721/enum.Expiration.html).

Noted that we require the uniqueness of the tuple `(nft, offerer)`. If an offerer makes multiple offers for the same NFT, only the last offer is stored.

#### Accept an offer

Owner of an NFT can accept an offer for that NFT. Transaction message format:
```json
{
    "accept_nft_offer": {
        "offerer": "the offerer address",
        "nft": {
            "contract_address": "the nft contract address",
            "token_id": "the nft token id"
        }, 
        "funds_amount": 1234
    }
}
```

The `funds_amount` is required for prevent front-running by offerer.

#### Cancel an offer

Offerers can cancel their offers at any time. Transaction message format:
```json
{
    "cancel_offer": {
        "nfts": [
            {
                "contract_address": "the nft contract address",
                "token_id": "the nft token id"
            }
        ]
    }
}
```