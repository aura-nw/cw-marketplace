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
list_nft: {
    "contract_address": "the nft contract address",
    "token_id": "the nft token id",
    "auction_config": "the listing config"
}
```

#### Buy a listed NFT

To buy a listed NFT, simply call `buy_nft` message:
```json
buy_nft: {
    "contract_address": "the nft contract address",
    "token_id": "the nft token id"
}
```

Depends on the required configuration, the contract will check for attached funds or use a `transfer_from` message to transfer asset from buyer to seller.
