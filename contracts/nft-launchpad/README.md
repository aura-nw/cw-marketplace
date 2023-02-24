# The Launchpad
## Background
Launchpads are very common in other marketplaces in other blockchains. For example, in Solana, MagicEden offers both a marketplace and a launchpad. In Stargaze, another Cosmos chain, their launchpad is the only way for a creator to launch a NFT project. Recently, OpenSea also starts their own launchpad named Drops. Running a launchpad is a safe way for a marketplace to get more users and creators.

## Goals
The launchpad needs to satisfy the following conditions:
- Managed by our admins. Because projects will be launched under Aura, we need to be able to manage their content and other information.
- Supports multiple launch strategies:
    - **public mint**: anyone with a wallet can mint
    - **whitelist mint**: only whitelisted wallets can mint
    - **randomly mint:** mint a NFT randomly from a list of predefined NFTs.
    - **reserved mint:** different whitelisted tiers mint a from different pools of predefined NFTs.
    - **reveal**: the content of NFTs are revealed after minting done.
    - **scheduled mint phase:** different whitelists mint, public mint (*mint phase*) happens in a scheduled timeline.

## Proposed Solution
```mermaid
graph TD
    Launchpad -- 1.b. instantiate --> Collection

    Admin-- 1.a. instantiate --> Launchpad
	Admin -- 2. setWhitelist --> Launchpad
    Admin -- 3. setSchedule --> Launchpad

    User -- 4.a. mint --> Launchpad
    Launchpad -- 4.b. mint_to_user   --> Collection

    Backend -- 5. sync --> Launchpad
```

## Launchpad Contract
This contract manages an individual launchpad. There is an admin which has total authority over how to config it. A NFT contract will be instantiated together with this contract and set this contract as the only minter.

### ExecuteMsg

There are 7 messages for administrative management purpose and can be called by admin only, they are:
- `AddMintPhase`
- `UpdateMintPhase`
- `RemoveMintPhase`
- `AddWhitelist`
- `RemoveWhitelist`
- `DeactivateLaunchpad`
- `ActivateLaunchpad`

The first 5 messages can be executed **before** launchpad starting. The last 2 messages are used to pause and unpause the launchpad.

`AddMintPhase{after_phase_id, phase_data}` - This message allows admin add new *mint phase* to launchpad. 
The **optional** parameter `after_phase_id` is used to determine the position of *mint phase* in the list of phases. 
By default, new phase will be added to the end of the list.

A `phase_data` contains the following data for a new phase:
```rust
pub struct PhaseData {
    pub start_time: Timestamp,
    pub end_time: Timestamp,
    pub max_supply: Option<u64>,
    pub max_nfts_per_address: u64,
    pub price: u128,
    pub is_public: bool,
}
```

`UpdateMintPhase{phase_id, phase_data}` - Admin can update the data in `phase_data` of a phase by specifying its `phase_id`. 
Requires `phase_id` to point to a valid phase.

`RemoveMintPhase{phase_id}` - Removes a phase pointed by a `phase_id`. Requires `phase_id` to point to a valid phase.

`AddWhitelist{phase_id, whitelist}` - Allows the address of users in `whitelist` to mint the Nfts at the current phase with `phase_id`. 

`RemoveWhitelist{phase_id, whitelist}` - Disallow the address of users in `whitelist` to mint the Nfts at the current phase with `phase_id`.

`DeactivateLaunchpad{}` - Admin can pause this launchpad if it is activated.

`ActivateLaunchpad{}` - Admin can unpause this launchpad if it is deactivated.

`Mint{phase_id}` - A user in whitelist can mint a Nft in the phase pointed by `phase_id`.

### QueryMsg

`GetLaunchpadInfo{}` - User can query the information of launchpad by this message.

The `LaunchpadInfo` response includes:
```rust
pub struct LaunchpadInfo {
    pub collection_address: Addr,
    pub total_supply: u64,
    pub max_supply: u64,
    pub uri_prefix: String,
    pub first_phase_id: u64,
    pub last_phase_id: u64,
    pub last_issued_id: u64,
    pub is_active: bool,
}
```

`GetAllPhaseConfigs{}` - Retrieves the information of all phases of launchpad.

The response of this query is a vector of `PhaseConfigResponse`:
```rust
pub struct PhaseConfigResponse {
    pub phase_id: u64,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
    pub max_supply: Option<u64>,
    pub total_supply: u64,
    pub max_nfts_per_address: u64,
    pub price: u128,
}
```