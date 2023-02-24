#[cfg(test)]
pub mod env {
    use crate::contract::{
        execute as LaunchpadExecute, instantiate as LaunchpadInstantiate, query as LaunchpadQuery,
        reply as LaunchpadReply,
    };
    use cosmwasm_std::{Addr, Coin, Empty, Uint128};
    use cw2981_royalties::{
        execute as cw2981_execute, instantiate as cw2981_instantiate, query as cw2981_query,
    };
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper};

    // ****************************************
    // You MUST define the constants value here
    // ****************************************
    pub const ADMIN: &str = "aura1000000000000000000000000000000000admin";
    pub const USER_1: &str = "aura1000000000000000000000000000000000user1";
    pub const USER_2: &str = "aura1000000000000000000000000000000000user2";

    pub const NATIVE_DENOM: &str = "uaura";
    pub const NATIVE_BALANCE: u128 = 1_000_000_000_000u128;

    pub const TOKEN_INITIAL_BALANCE: u128 = 1_000_000_000_000u128;

    pub struct ContractInfo {
        pub contract_code_id: u64,
    }

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(ADMIN),
                    vec![Coin {
                        denom: NATIVE_DENOM.to_string(),
                        amount: Uint128::new(NATIVE_BALANCE),
                    }],
                )
                .unwrap();
        })
    }

    fn cw2981_contract_template() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(cw2981_execute, cw2981_instantiate, cw2981_query);
        Box::new(contract)
    }

    fn nft_launchpad_contract_template() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(LaunchpadExecute, LaunchpadInstantiate, LaunchpadQuery)
            .with_reply(LaunchpadReply);
        Box::new(contract)
    }

    pub fn instantiate_contracts() -> (App, Vec<ContractInfo>) {
        // Create a new app instance
        let mut app = mock_app();

        // Cw2981 contract
        // store the code of all contracts to the app and get the code ids
        let cw2981_contract_code_id = app.store_code(cw2981_contract_template());

        let mut contract_info_vec: Vec<ContractInfo> = Vec::new();

        // add contract info to the vector
        contract_info_vec.push(ContractInfo {
            contract_code_id: cw2981_contract_code_id,
        });

        // NFT Launchpad contract
        // store the code of all contracts to the app and get the code ids
        let launchpad_contract_code_id = app.store_code(nft_launchpad_contract_template());

        // add contract info to the vector
        contract_info_vec.push(ContractInfo {
            contract_code_id: launchpad_contract_code_id,
        });

        // return the app instance, the addresses and code IDs of all contracts
        (app, contract_info_vec)
    }
}
