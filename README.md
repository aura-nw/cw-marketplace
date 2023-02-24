[![CircleCI](https://dl.circleci.com/status-badge/img/gh/aura-nw/cw-marketplace/tree/main.svg?style=svg)](https://dl.circleci.com/status-badge/redirect/gh/aura-nw/cw-marketplace/tree/main)
[![codecov](https://codecov.io/gh/aura-nw/cw-marketplace/branch/main/graph/badge.svg?token=ZIQKZ3B8C9)](https://codecov.io/gh/aura-nw/cw-marketplace)

# NFT Marketplace

This repo contains smart contracts for a NFT Marketplace and Launchpad in a Cosmos blockchain using [CosmWasm](https://cosmwasm.com/).

## Maintainers

This repo is maintained by [Aura Network](https://aura.network).

## How it works

There are 4 contracts:
- cw2981-royalties: a modified cw2981-royalties of cw-nfts. We changed it to support contract-level royalties instead of token-level royalties.
- bidding-token: a modified cw20-base of [cw-plus](https://github.com/CosmWasm/cw-plus). We changed it to a warped token used only by marketplace contract.
- nft-marketplace: a NFT marketplace contract which allow any users to list their NFTs and offer others.
- launchpad: a NFT launchpad that allow creators to sell NFTs.

### Marketplace contract

This contract supports 2 functions:
- Users list NFTs for sale.
- Users make offer on others' NFTs.

For better user experience, we do not lock users' NFTs when they are listed on sale. This indeed can lead to issues with invalid listings. We will look in to ways to resolve those issues without compromise on UX. For the same reason, we also do not lock users' token when they make offers with those token.

```mermaid
sequenceDiagram

participant U as User
Participant M as Marketplace
participant C as Collection

U ->>+ M: list_nft(nft, price)
M ->>+ C: check_owner(user, nft)
C -->>- M: owner
alt user is owner
  M ->> M: create_listing(nft, price)
  M -->>  U: listing
else user is not owner
  M -->>- U: tx failed
end
```
User lists NFT in marketplace for sale

```mermaid
sequenceDiagram

participant U as User
participant M as Marketplace
participant C as Collection
participant B as Bank

U ->>+ M: buy(listing)
alt can_buy(user, listing)
  M ->>+ C: transfer(listing.nft, user)
  C -->>- M: result
  M ->>+ B: transfer(listing.price, listing.seller)
  B -->>- M: result
  alt all transfers succeeded
    M -->> U: tx succeeded
  else
    M -->> U: tx failed
  end
else invalid request
  M -->>- U: tx failed
end
```
User buys a listing in marketplace

```mermaid
sequenceDiagram

participant U as User
participant M as Marketplace
participant C as Collection
participant B as Bidding Token

U ->>+ M: make_offer(nft, price)
M ->>+ C: check_owner(user, nft)
C -->>- M: owner
M ->>+ B: get_balance(user)
B -->>- M: user_balance
alt user == owner AND user_balance >= price
  M ->> M: create_offer(user, nft, price)
  M -->> U: offer
else invalid offer
  M -->>- U: tx failed
end
```
User makes offer on a NFT on marketplace

```mermaid
sequenceDiagram

participant U as User
participant M as Marketplace
participant C as Collection
participant B as Bidding Token

U ->>+ M: accept(offer)
alt is_valid(offer, user)
  M ->>+ C: transfer(offer.nft, offer.offerer)
  C -->>- M: result
  M ->>+ B: transfer(offer.price, offer.offerer, user.address)
  B -->>- M: result
  alt all transfers succeeded
    M ->> M: remove_any_listing(offer.nft)
    M -->> U: tx succeeded
  else any transfer failed
    M -->> U: tx failed
  end
else not valid
  M -->>- U: tx failed
end
```
User accepts an offer

### Launchpad contract

```mermaid
graph TD
  Admin -- 1.a. instantiate --> Launchpad
  Launchpad -- 1.b. instantiate --> Collection

	Admin -- 2. set_whitelist --> Launchpad
  Admin -- 3. set_mint_phase --> Launchpad

  User -- 4.a. mint --> Launchpad
  Launchpad -- 4.b. mint_to_user   --> Collection
```

Current version of the launchpad uses a simple procedure for generating random NFT IDs. It is verifiable but not a true random number. For example, block proposer could manipulate the result by changing the order of executing mint transactions. However, given the low block time of Cosmos chain in general, it is mostly sufficient for 

## Contributing

Aura Network welcome any feedback for improving the contracts or fixing security issues.
Please raise issues and/or PRs. You can also join [Aura Network discord](https://aura.network/discord) for discussing any ideas and questions.
