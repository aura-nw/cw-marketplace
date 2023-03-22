#[cfg(test)]
pub mod env {
    use cosmwasm_std::{Addr, Coin, Empty, StdError, Uint128};
    use cw20::{Cw20Coin, MinterResponse};
    use cw2981_royalties::msg::InstantiateMsg as Cw2981InstantiateMsg;
    use cw2981_royalties::{
        execute as cw2981_execute, instantiate as cw2981_instantiate, query as cw2981_query,
    };

    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    // The bidding token contract
    use bidding_token::contract::{
        execute as cw20_execute, instantiate as cw20_instantiate, query as cw20_query,
    };
    use bidding_token::state::InstantiateMsg as Cw20InstantiateMsg;

    use crate::contract::{
        execute as MarketPlaceExecute, instantiate as MarketPlaceInstantiate,
        query as MarketPlaceQuery,
    };
    use crate::msg::{ExecuteMsg as MarketPlaceExecuteMsg, InstantiateMsg};

    // ****************************************
    // You MUST define the constants value here
    // ****************************************
    pub const OWNER: &str = "aura1000000000000000000000000000000000admin";
    pub const USER_1: &str = "aura1000000000000000000000000000000000user1";
    pub const USER_2: &str = "aura1000000000000000000000000000000000user2";

    pub const NATIVE_DENOM: &str = "uaura";
    pub const NATIVE_BALANCE: u128 = 1_000_000_000_000u128;

    pub const NATIVE_DENOM_2: &str = "uaura1";
    pub const NATIVE_BALANCE_2: u128 = 500_000_000_000u128;

    pub const TOKEN_INITIAL_BALANCE: u128 = 1_000_000_000_000u128;

    pub struct ContractInfo {
        pub contract_addr: String,
        pub contract_code_id: u64,
    }

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(OWNER),
                    vec![
                        Coin {
                            denom: NATIVE_DENOM.to_string(),
                            amount: Uint128::new(NATIVE_BALANCE),
                        },
                        Coin {
                            denom: NATIVE_DENOM_2.to_string(),
                            amount: Uint128::new(NATIVE_BALANCE_2),
                        },
                    ],
                )
                .unwrap();
        })
    }

    // *********************************************************
    // You MUST define the templates of all contracts here
    // Follow the example (1) below:
    //  pub fn contract_template() -> Box<dyn Contract<Empty>> {
    //      let contract = ContractWrapper::new(
    //          crate::contract::execute,
    //          crate::contract::instantiate,
    //          crate::contract::query,
    //      );
    //      Box::new(contract)
    //  }
    // *********************************************************
    fn cw20_contract_template() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(cw20_execute, cw20_instantiate, cw20_query);
        Box::new(contract)
    }

    fn cw2981_contract_template() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(cw2981_execute, cw2981_instantiate, cw2981_query);
        Box::new(contract)
    }

    fn nft_marketplace_contract_template() -> Box<dyn Contract<Empty>> {
        let contract =
            ContractWrapper::new(MarketPlaceExecute, MarketPlaceInstantiate, MarketPlaceQuery);
        Box::new(contract)
    }

    // *********************************************************
    // You MUST store code and instantiate all contracts here
    // Follow the example (2) below:
    // @return App: the mock app
    // @return String: the address of the contract
    // @return u64: the code id of the contract
    //    pub fn instantiate_contracts() -> (App, String, u64) {
    //        // Create a new app instance
    //        let mut app = mock_app();
    //
    //        // store the code of all contracts to the app and get the code ids
    //        let contract_code_id = app.store_code(contract_template());
    //
    //        // create instantiate message for contract
    //        let contract_instantiate_msg = InstantiateMsg {
    //            name: "Contract_A".to_string(),
    //        };
    //
    //        // instantiate contract
    //        let contract_addr = app
    //            .instantiate_contract(
    //                contract_code_id,
    //                Addr::unchecked(OWNER),
    //                &contract_instantiate_msg,
    //                &[],
    //                "test instantiate contract",
    //                None,
    //            )
    //            .unwrap();
    //
    //        // return the app instance, the addresses and code IDs of all contracts
    //        (app, contract_addr, contract_code_id)
    //    }
    // *********************************************************
    pub fn instantiate_contracts() -> (App, Vec<ContractInfo>) {
        // Create a new app instance
        let mut app = mock_app();

        // Mint 1000000000 native token to USER_1
        app.sudo(cw_multi_test::SudoMsg::Bank(
            cw_multi_test::BankSudo::Mint {
                to_address: USER_1.to_string(),
                amount: vec![Coin {
                    amount: Uint128::from(1000000000u128),
                    denom: NATIVE_DENOM.to_string(),
                }],
            },
        ))
        .unwrap();

        // Mint 1000000000 native token to USER_2
        app.sudo(cw_multi_test::SudoMsg::Bank(
            cw_multi_test::BankSudo::Mint {
                to_address: USER_2.to_string(),
                amount: vec![Coin {
                    amount: Uint128::from(1000000000u128),
                    denom: NATIVE_DENOM.to_string(),
                }],
            },
        ))
        .unwrap();

        // Cw2981 contract
        // store the code of all contracts to the app and get the code ids
        let cw2981_contract_code_id = app.store_code(cw2981_contract_template());

        let mut contract_info_vec: Vec<ContractInfo> = Vec::new();

        // create instantiate message for contract
        let cw2981_msg = Cw2981InstantiateMsg {
            name: "NFT_A".to_string(),
            symbol: "NFT".to_string(),
            minter: OWNER.to_string(),
            royalty_percentage: Some(20),
            royalty_payment_address: Some(OWNER.to_string()),
            creator: None,
        };

        // instantiate contract
        let cw2981_contract_addr = app
            .instantiate_contract(
                cw2981_contract_code_id,
                Addr::unchecked(OWNER),
                &cw2981_msg,
                &[],
                "test instantiate cw2981 contract",
                None,
            )
            .unwrap();

        // add contract info to the vector
        contract_info_vec.push(ContractInfo {
            contract_addr: cw2981_contract_addr.to_string(),
            contract_code_id: cw2981_contract_code_id,
        });

        // NFT Marketplace contract
        // store the code of all contracts to the app and get the code ids
        let marketplace_contract_code_id = app.store_code(nft_marketplace_contract_template());

        // create instantiate message for contract
        let msg = InstantiateMsg {
            owner: Addr::unchecked(OWNER),
        };

        // instantiate contract
        let marketplace_contract_addr = app
            .instantiate_contract(
                marketplace_contract_code_id,
                Addr::unchecked(OWNER),
                &msg,
                &[],
                "test instantiate marketplace contract",
                None,
            )
            .unwrap();

        // add contract info to the vector
        contract_info_vec.push(ContractInfo {
            contract_addr: marketplace_contract_addr.to_string(),
            contract_code_id: marketplace_contract_code_id,
        });

        // Cw20 contract
        // store the code of all contracts to the app and get the code ids
        let contract_code_id = app.store_code(cw20_contract_template());

        // create instantiate message for contract
        let contract_instantiate_msg = Cw20InstantiateMsg {
            name: "Contract_A".to_string(),
            symbol: "vAura".to_string(),
            decimals: 6,
            initial_balances: [].to_vec(),
            mint: Some(MinterResponse {
                minter: marketplace_contract_addr.to_string(), // the minter of the cw20 token must be the marketplace contract
                cap: Some(Uint128::new(TOKEN_INITIAL_BALANCE)),
            }),
            marketplace_address: marketplace_contract_addr.to_string(),
            native_denom: NATIVE_DENOM.to_string(),
        };

        // instantiate contract
        let contract_addr = app
            .instantiate_contract(
                contract_code_id,
                Addr::unchecked(OWNER),
                &contract_instantiate_msg,
                &[],
                "test instantiate contract",
                None,
            )
            .unwrap();

        // add contract info to the vector
        contract_info_vec.push(ContractInfo {
            contract_addr: contract_addr.to_string(),
            contract_code_id,
        });

        // set the vaura token address in the marketplace contract
        let set_vaura_token_msg = MarketPlaceExecuteMsg::EditVauraToken {
            token_address: contract_addr.to_string(),
        };

        let res = app.execute_contract(
            Addr::unchecked(OWNER),
            Addr::unchecked(marketplace_contract_addr),
            &set_vaura_token_msg,
            &[],
        );
        assert!(res.is_ok());

        // return the app instance, the addresses and code IDs of all contracts
        (app, contract_info_vec)
    }

    // cannot instantiate bidding-token contract with initial_balance
    #[test]
    fn cannot_instantiate_cw20_with_initial_balance() {
        let mut app = mock_app();
        let contract_code_id = app.store_code(cw20_contract_template());
        let contract_instantiate_msg = Cw20InstantiateMsg {
            name: "Contract_A".to_string(),
            symbol: "vAura".to_string(),
            decimals: 6,
            initial_balances: vec![Cw20Coin {
                address: Addr::unchecked(OWNER).to_string(),
                amount: Uint128::new(TOKEN_INITIAL_BALANCE),
            }],
            mint: Some(MinterResponse {
                minter: Addr::unchecked(OWNER).to_string(),
                cap: Some(Uint128::new(TOKEN_INITIAL_BALANCE)),
            }),
            marketplace_address: Addr::unchecked(OWNER).to_string(),
            native_denom: NATIVE_DENOM.to_string(),
        };
        let err = app
            .instantiate_contract(
                contract_code_id,
                Addr::unchecked(OWNER),
                &contract_instantiate_msg,
                &[],
                "test instantiate contract",
                None,
            )
            .unwrap_err();
        assert_eq!(
            err.source().unwrap().to_string(),
            StdError::generic_err("Initial balances must be empty").to_string()
        );
    }
}
