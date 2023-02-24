# Bidding fund token

This contract is a modified **cw20-base** token, inspired by [cw-plus](https://github.com/CosmWasm/cw-plus). This token serves as a medium for seamlessly running **nft-marketplace** contract.

Basically, this token is a wrapped version of a native coin on a Cosmos blockchain.
The denom of the native coin and the marketplace contract's address must be set when instantiate this contract:
```rust
#[cw_serde]
pub struct InstantiateMsg {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub initial_balances: Vec<Cw20Coin>,
    pub mint: Option<MinterResponse>,
    pub marketplace_address: String,
    pub native_denom: String,
}
```

Users can swap between the native coin and this token buy calling the mint and burn function, which have the same signatures as in cw20-base:
```rust
    // anyone can call Mint to swap from supported native token to bidding token, the native token will be locked in this contract
    Mint { recipient: String, amount: Uint128 },

    // anyone with bidding token can chttps://github.com/CosmWasm/cw-plus/tree/main/contracts/cw20-baseonvert back to supported native token by burning them
    Burn { amount: Uint128 },

```

This contract only allows the corresponding **nft-marketplace** to manage users' token in processing their offers and listings.
Consequently, users do not need to explicit approve the **nft-marketplace**.
We also throw on every `ExecuteMsg` except `TransferFrom`, which will be called by the **nft-marketplace** contract, `Mint` and `Burn`.
