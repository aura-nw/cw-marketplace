#[cfg(test)]
mod tests {
    use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
    use crate::testing_config::env::{instantiate_contracts, ADMIN, CREATOR, LAUNCHPAD_COLLECTOR};

    use cosmwasm_std::Addr;

    use crate::msg::ColectionInfo;
    use crate::state::LaunchpadInfo;
    use cw2981_royalties::ContractInfoResponse;
    use cw2981_royalties::QueryMsg as Cw721QueryMsg;
    use cw_multi_test::{App, Executor};

    pub const COLLECTION_NAME: &str = "A launchpad collection";
    pub const COLLECTION_SYMBOL: &str = "LPC";

    mod create_launchpad {
        use super::*;

        #[test]
        fn admin_can_instantiate_new_launchpad() {
            // get integration test app and contracts
            let (mut app, contracts) = instantiate_contracts();
            let cw2981_code_id = contracts[0].contract_code_id;
            let launchpad_code_id = contracts[1].contract_code_id;

            // prepare instantiate msg for launchpad contract
            let instantiate_msg = InstantiateMsg {
                colection_code_id: cw2981_code_id,
                collection_info: ColectionInfo {
                    name: COLLECTION_NAME.to_string(),
                    symbol: COLLECTION_SYMBOL.to_string(),
                    royalty_percentage: None,
                    royalty_payment_address: None,
                    max_supply: 5000,
                    uri_prefix:
                        "ipfs://bafybeifm3xas2egfbwzo7cg5wiayw44sbvfn6h5am2bydp2zpnypl7g5tq/images/"
                            .to_string(),
                    uri_suffix: ".json".to_string(),
                    creator: CREATOR.to_string(),
                    reserved_tokens: None,
                },
                random_seed: "9e8e26615f51552aa3b18b6f0bcf0dae5afbe30321e8d1237fa51ebeb1d8fe62"
                    .to_string(),
                launchpad_fee: 10,
                launchpad_collector: Some(LAUNCHPAD_COLLECTOR.to_string()),
            };

            // instantiate launchpad contract
            let launchpad_addr = app
                .instantiate_contract(
                    launchpad_code_id,
                    Addr::unchecked(ADMIN),
                    &instantiate_msg,
                    &[],
                    "test instantiate marketplace contract",
                    None,
                )
                .unwrap();

            // query launchpad info from launchpad contract
            let launchpad_info: LaunchpadInfo = app
                .wrap()
                .query_wasm_smart(
                    Addr::unchecked(launchpad_addr),
                    &QueryMsg::GetLaunchpadInfo {},
                )
                .unwrap();

            // query collection info from cw2981 contract using collection_address from launchpad info
            let res: ContractInfoResponse = app
                .wrap()
                .query_wasm_smart(
                    launchpad_info.collection_address,
                    &Cw721QueryMsg::ContractInfo {},
                )
                .unwrap();

            assert_eq!(res.name, COLLECTION_NAME);
            assert_eq!(res.symbol, COLLECTION_SYMBOL);
            assert_eq!(res.creator.unwrap(), CREATOR);
        }

        #[test]
        fn cannot_instantiate_new_launchpad_because_fee_too_high() {
            // get integration test app and contracts
            let (mut app, contracts) = instantiate_contracts();
            let cw2981_code_id = contracts[0].contract_code_id;
            let launchpad_code_id = contracts[1].contract_code_id;

            // prepare instantiate msg for launchpad contract
            let instantiate_msg = InstantiateMsg {
                colection_code_id: cw2981_code_id,
                collection_info: ColectionInfo {
                    name: COLLECTION_NAME.to_string(),
                    symbol: COLLECTION_SYMBOL.to_string(),
                    royalty_percentage: None,
                    royalty_payment_address: None,
                    max_supply: 5000,
                    uri_prefix:
                        "ipfs://bafybeifm3xas2egfbwzo7cg5wiayw44sbvfn6h5am2bydp2zpnypl7g5tq/images/"
                            .to_string(),
                    uri_suffix: ".json".to_string(),
                    creator: CREATOR.to_string(),
                    reserved_tokens: None,
                },
                random_seed: "9e8e26615f51552aa3b18b6f0bcf0dae5afbe30321e8d1237fa51ebeb1d8fe62"
                    .to_string(),
                launchpad_fee: 100,
                launchpad_collector: Some(LAUNCHPAD_COLLECTOR.to_string()),
            };

            // instantiate launchpad contract
            let res = app.instantiate_contract(
                launchpad_code_id,
                Addr::unchecked(ADMIN),
                &instantiate_msg,
                &[],
                "test instantiate marketplace contract",
                None,
            );
            assert_eq!(
                res.unwrap_err().source().unwrap().to_string(),
                "Invalid launchpad fee"
            );
        }
    }

    pub fn create_launchpad() -> (App, Addr) {
        // get integration test app and contracts
        let (mut app, contracts) = instantiate_contracts();
        let cw2981_code_id = contracts[0].contract_code_id;
        let launchpad_code_id = contracts[1].contract_code_id;

        // prepare instantiate msg for launchpad contract
        let instantiate_msg = InstantiateMsg {
            colection_code_id: cw2981_code_id,
            collection_info: ColectionInfo {
                name: COLLECTION_NAME.to_string(),
                symbol: COLLECTION_SYMBOL.to_string(),
                royalty_percentage: None,
                royalty_payment_address: None,
                max_supply: 5000,
                uri_prefix:
                    "ipfs://bafybeifm3xas2egfbwzo7cg5wiayw44sbvfn6h5am2bydp2zpnypl7g5tq/images/"
                        .to_string(),
                uri_suffix: ".json".to_string(),
                creator: CREATOR.to_string(),
                reserved_tokens: None,
            },
            random_seed: "9e8e26615f51552aa3b18b6f0bcf0dae5afbe30321e8d7ea7fa51ebeb1d8fe62"
                .to_string(),
            launchpad_fee: 10,
            launchpad_collector: Some(LAUNCHPAD_COLLECTOR.to_string()),
        };

        // instantiate launchpad contract
        let launchpad_addr = app
            .instantiate_contract(
                launchpad_code_id,
                Addr::unchecked(ADMIN),
                &instantiate_msg,
                &[],
                "test instantiate marketplace contract",
                None,
            )
            .unwrap();

        (app, launchpad_addr)
    }

    pub fn create_launchpad_with_number_supply(max_cap: u64) -> (App, Addr) {
        // get integration test app and contracts
        let (mut app, contracts) = instantiate_contracts();
        let cw2981_code_id = contracts[0].contract_code_id;
        let launchpad_code_id = contracts[1].contract_code_id;

        // prepare instantiate msg for launchpad contract
        let instantiate_msg = InstantiateMsg {
            colection_code_id: cw2981_code_id,
            collection_info: ColectionInfo {
                name: COLLECTION_NAME.to_string(),
                symbol: COLLECTION_SYMBOL.to_string(),
                royalty_percentage: None,
                royalty_payment_address: None,
                max_supply: max_cap,
                uri_prefix:
                    "ipfs://bafybeifm3xas2egfbwzo7cg5wiayw44sbvfn6h5am2bydp2zpnypl7g5tq/images/"
                        .to_string(),
                uri_suffix: ".json".to_string(),
                creator: CREATOR.to_string(),
                reserved_tokens: None,
            },
            random_seed: "9e8e26615f51552aa3b18b6f0bcf0dae5afbe30321e8d7ea7fa51ebeb1d8fe62"
                .to_string(),

            launchpad_fee: 0,
            launchpad_collector: Some(LAUNCHPAD_COLLECTOR.to_string()),
        };

        // instantiate launchpad contract
        let launchpad_addr = app
            .instantiate_contract(
                launchpad_code_id,
                Addr::unchecked(ADMIN),
                &instantiate_msg,
                &[],
                "test instantiate marketplace contract",
                None,
            )
            .unwrap();

        (app, launchpad_addr)
    }

    pub fn create_launchpad_with_reseved_list() -> (App, Addr) {
        // get integration test app and contracts
        let (mut app, contracts) = instantiate_contracts();
        let cw2981_code_id = contracts[0].contract_code_id;
        let launchpad_code_id = contracts[1].contract_code_id;

        // prepare instantiate msg for launchpad contract
        let instantiate_msg = InstantiateMsg {
            colection_code_id: cw2981_code_id,
            collection_info: ColectionInfo {
                name: COLLECTION_NAME.to_string(),
                symbol: COLLECTION_SYMBOL.to_string(),
                royalty_percentage: None,
                royalty_payment_address: None,
                max_supply: 15,
                uri_prefix:
                    "ipfs://bafybeifm3xas2egfbwzo7cg5wiayw44sbvfn6h5am2bydp2zpnypl7g5tq/images/"
                        .to_string(),
                uri_suffix: ".json".to_string(),
                creator: CREATOR.to_string(),
                reserved_tokens: Some([2, 6, 3, 5, 20].into()),
            },
            random_seed: "9e8e26615f51552aa3b18b6f0bcf0dae5afbe30321e8d7ea7fa51ebeb1d8fe62"
                .to_string(),

            launchpad_fee: 0,
            launchpad_collector: Some(LAUNCHPAD_COLLECTOR.to_string()),
        };

        // instantiate launchpad contract
        let launchpad_addr = app
            .instantiate_contract(
                launchpad_code_id,
                Addr::unchecked(ADMIN),
                &instantiate_msg,
                &[],
                "test instantiate marketplace contract",
                None,
            )
            .unwrap();

        (app, launchpad_addr)
    }

    mod update_launchpad {
        use cosmwasm_std::{coin, BlockInfo, Coin};
        use cw721::TokensResponse;

        use crate::{
            msg::MintableResponse,
            state::{PhaseConfigResponse, PhaseData},
            testing_config::env::{NATIVE_DENOM, USER_1, USER_2},
        };

        use super::*;

        #[test]
        fn admin_can_add_the_first_phase_to_launchpad() {
            // get integration test app and launchpad address
            let (mut app, launchpad_address) = create_launchpad();

            // prepare execute msg for adding new phase to launchpad
            let add_new_phase_msg = ExecuteMsg::AddMintPhase {
                after_phase_id: None,
                phase_data: PhaseData {
                    start_time: app.block_info().time.plus_seconds(110),
                    end_time: app.block_info().time.plus_seconds(200),
                    max_supply: Some(2000),
                    max_nfts_per_address: 2,
                    price: coin(500000, NATIVE_DENOM),
                    is_public: false,
                },
            };

            // execute add new phase msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address),
                &add_new_phase_msg,
                &[],
            );
            println!("res: {:?}", res);
            assert!(res.is_ok());
        }

        #[test]
        fn cannot_add_the_first_phase_to_launchpad_because_time_invalid() {
            // get integration test app and launchpad address
            let (mut app, launchpad_address) = create_launchpad();

            // prepare execute msg for adding new phase to launchpad
            let add_new_phase_msg = ExecuteMsg::AddMintPhase {
                after_phase_id: None,
                phase_data: PhaseData {
                    start_time: app.block_info().time.minus_seconds(110),
                    end_time: app.block_info().time.plus_seconds(200),
                    max_supply: Some(2000),
                    max_nfts_per_address: 2,
                    price: coin(500000, NATIVE_DENOM),
                    is_public: false,
                },
            };

            // execute add new phase msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address),
                &add_new_phase_msg,
                &[],
            );
            assert_eq!(
                res.unwrap_err().source().unwrap().to_string(),
                "Invalid phase time"
            );
        }

        #[test]
        fn cannot_add_another_phase_because_the_launchpad_started() {
            // get integration test app and launchpad address
            let (mut app, launchpad_address) = create_launchpad();

            // ADD FIRST PHASE to the first position
            // prepare execute msg for adding new phase to launchpad
            let add_first_phase_msg = ExecuteMsg::AddMintPhase {
                after_phase_id: None,
                phase_data: PhaseData {
                    start_time: app.block_info().time.plus_seconds(140),
                    end_time: app.block_info().time.plus_seconds(600),
                    max_supply: Some(1000),
                    max_nfts_per_address: 2,
                    price: coin(500000, NATIVE_DENOM),
                    is_public: false,
                },
            };

            // execute add new phase msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_first_phase_msg,
                &[],
            );
            assert!(res.is_ok());

            // admin activate launchpad
            let activate_launchpad_msg = ExecuteMsg::ActivateLaunchpad {};
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &activate_launchpad_msg,
                &[],
            );
            assert!(res.is_ok());

            // change block time increase 400 seconds to make phase active
            app.set_block(BlockInfo {
                time: app.block_info().time.plus_seconds(400),
                height: app.block_info().height + 1,
                chain_id: app.block_info().chain_id,
            });

            // ADD SECOND PHASE to the first position
            // prepare execute msg for adding new phase to launchpad
            let add_second_phase_msg = ExecuteMsg::AddMintPhase {
                after_phase_id: Some(0),
                phase_data: PhaseData {
                    start_time: app.block_info().time.plus_seconds(100),
                    end_time: app.block_info().time.plus_seconds(120),
                    max_supply: Some(1000),
                    max_nfts_per_address: 2,
                    price: coin(500000, NATIVE_DENOM),
                    is_public: false,
                },
            };

            // execute add new phase msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_second_phase_msg,
                &[],
            );
            assert_eq!(
                res.unwrap_err().source().unwrap().to_string(),
                "Launchpad started"
            );

            // query all phases config from launchpad contract
            let phase_config_info: Vec<PhaseConfigResponse> = app
                .wrap()
                .query_wasm_smart(
                    Addr::unchecked(launchpad_address),
                    &QueryMsg::GetAllPhaseConfigs {},
                )
                .unwrap();

            assert_eq!(phase_config_info.len(), 1);
            assert_eq!(phase_config_info[0].phase_id, 1);
        }

        #[test]
        fn admin_can_add_a_phase_in_specify_position_to_launchpad() {
            // get integration test app and launchpad address
            let (mut app, launchpad_address) = create_launchpad();

            // ADD FIRST PHASE to the first position
            // prepare execute msg for adding new phase to launchpad
            let add_first_phase_msg = ExecuteMsg::AddMintPhase {
                after_phase_id: None,
                phase_data: PhaseData {
                    start_time: app.block_info().time.plus_seconds(140),
                    end_time: app.block_info().time.plus_seconds(180),
                    max_supply: Some(1000),
                    max_nfts_per_address: 2,
                    price: coin(500000, NATIVE_DENOM),
                    is_public: false,
                },
            };

            // execute add new phase msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_first_phase_msg,
                &[],
            );
            assert!(res.is_ok());

            // ADD SECOND PHASE to the first position
            // prepare execute msg for adding new phase to launchpad
            let add_second_phase_msg = ExecuteMsg::AddMintPhase {
                after_phase_id: Some(0),
                phase_data: PhaseData {
                    start_time: app.block_info().time.plus_seconds(100),
                    end_time: app.block_info().time.plus_seconds(120),
                    max_supply: Some(1000),
                    max_nfts_per_address: 2,
                    price: coin(500000, NATIVE_DENOM),
                    is_public: false,
                },
            };

            // execute add new phase msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_second_phase_msg,
                &[],
            );
            println!("res: {:?}", res);
            assert!(res.is_ok());

            // query all phases config from launchpad contract
            let phase_config_info: Vec<PhaseConfigResponse> = app
                .wrap()
                .query_wasm_smart(
                    Addr::unchecked(launchpad_address),
                    &QueryMsg::GetAllPhaseConfigs {},
                )
                .unwrap();

            assert_eq!(phase_config_info.len(), 2);
            assert_eq!(phase_config_info[0].phase_id, 2);
            assert_eq!(phase_config_info[1].phase_id, 1);
        }

        #[test]
        fn cannot_add_a_phase_in_specify_position_because_time_invalid() {
            // get integration test app and launchpad address
            let (mut app, launchpad_address) = create_launchpad();

            // ADD FIRST PHASE to the first position
            // prepare execute msg for adding new phase to launchpad
            let add_first_phase_msg = ExecuteMsg::AddMintPhase {
                after_phase_id: None,
                phase_data: PhaseData {
                    start_time: app.block_info().time.plus_seconds(140),
                    end_time: app.block_info().time.plus_seconds(180),
                    max_supply: Some(1000),
                    max_nfts_per_address: 2,
                    price: coin(500000, NATIVE_DENOM),
                    is_public: false,
                },
            };

            // execute add new phase msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_first_phase_msg,
                &[],
            );
            assert!(res.is_ok());

            // ADD SECOND PHASE to the first position
            // prepare execute msg for adding new phase to launchpad
            let add_second_phase_msg = ExecuteMsg::AddMintPhase {
                after_phase_id: Some(0),
                phase_data: PhaseData {
                    start_time: app.block_info().time.plus_seconds(20),
                    end_time: app.block_info().time.plus_seconds(160),
                    max_supply: Some(1000),
                    max_nfts_per_address: 2,
                    price: coin(500000, NATIVE_DENOM),
                    is_public: false,
                },
            };

            // execute add new phase msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address),
                &add_second_phase_msg,
                &[],
            );
            assert_eq!(
                res.unwrap_err().source().unwrap().to_string(),
                "Invalid phase time"
            );
        }

        #[test]
        fn cannot_add_another_phase_at_the_last_position_because_time_invalid() {
            // get integration test app and launchpad address
            let (mut app, launchpad_address) = create_launchpad();

            // ADD FIRST PHASE to the first position
            // prepare execute msg for adding new phase to launchpad
            let add_first_phase_msg = ExecuteMsg::AddMintPhase {
                after_phase_id: None,
                phase_data: PhaseData {
                    start_time: app.block_info().time.plus_seconds(140),
                    end_time: app.block_info().time.plus_seconds(180),
                    max_supply: Some(1000),
                    max_nfts_per_address: 2,
                    price: coin(500000, NATIVE_DENOM),
                    is_public: false,
                },
            };

            // execute add new phase msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_first_phase_msg,
                &[],
            );
            assert!(res.is_ok());

            // ADD SECOND PHASE to the first position
            // prepare execute msg for adding new phase to launchpad
            let add_second_phase_msg = ExecuteMsg::AddMintPhase {
                after_phase_id: None,
                phase_data: PhaseData {
                    start_time: app.block_info().time.plus_seconds(179),
                    end_time: app.block_info().time.plus_seconds(200),
                    max_supply: Some(1000),
                    max_nfts_per_address: 2,
                    price: coin(500000, NATIVE_DENOM),
                    is_public: false,
                },
            };

            // execute add new phase msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address),
                &add_second_phase_msg,
                &[],
            );
            assert_eq!(
                res.unwrap_err().source().unwrap().to_string(),
                "Invalid phase time"
            );
        }

        #[test]
        fn cannot_add_new_phase_because_not_admin() {
            // get integration test app and launchpad address
            let (mut app, launchpad_address) = create_launchpad();

            // prepare execute msg for adding new phase to launchpad
            let add_new_phase_msg = ExecuteMsg::AddMintPhase {
                after_phase_id: None,
                phase_data: PhaseData {
                    start_time: app.block_info().time.plus_seconds(110),
                    end_time: app.block_info().time.plus_seconds(200),
                    max_supply: Some(2000),
                    max_nfts_per_address: 2,
                    price: coin(500000, NATIVE_DENOM),
                    is_public: false,
                },
            };

            // execute add new phase msg
            let res = app.execute_contract(
                Addr::unchecked(USER_1),
                Addr::unchecked(launchpad_address),
                &add_new_phase_msg,
                &[],
            );
            assert_eq!(
                res.unwrap_err().source().unwrap().to_string(),
                "Unauthorized"
            );
        }

        #[test]
        fn cannot_add_new_phase_because_the_time_invalid() {
            // get integration test app and launchpad address
            let (mut app, launchpad_address) = create_launchpad();

            // prepare execute msg for adding new phase to launchpad
            let add_new_phase_msg = ExecuteMsg::AddMintPhase {
                after_phase_id: None,
                phase_data: PhaseData {
                    start_time: app.block_info().time.plus_seconds(110),
                    end_time: app.block_info().time.plus_seconds(100),
                    max_supply: Some(2000),
                    max_nfts_per_address: 2,
                    price: coin(500000, NATIVE_DENOM),
                    is_public: false,
                },
            };

            // execute add new phase msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address),
                &add_new_phase_msg,
                &[],
            );
            assert_eq!(
                res.unwrap_err().source().unwrap().to_string(),
                "Invalid phase time"
            );
        }

        #[test]
        fn admin_can_add_a_phase_in_the_last_position() {
            // get integration test app and launchpad address
            let (mut app, launchpad_address) = create_launchpad();

            // ADD FIRST PHASE to the first position
            // prepare execute msg for adding new phase to launchpad
            let add_first_phase_msg = ExecuteMsg::AddMintPhase {
                after_phase_id: None,
                phase_data: PhaseData {
                    start_time: app.block_info().time.plus_seconds(140),
                    end_time: app.block_info().time.plus_seconds(180),
                    max_supply: Some(1000),
                    max_nfts_per_address: 2,
                    price: coin(500000, NATIVE_DENOM),
                    is_public: false,
                },
            };

            // execute add new phase msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_first_phase_msg,
                &[],
            );
            assert!(res.is_ok());

            // ADD SECOND PHASE to the first position
            // prepare execute msg for adding new phase to launchpad
            let add_second_phase_msg = ExecuteMsg::AddMintPhase {
                after_phase_id: Some(1),
                phase_data: PhaseData {
                    start_time: app.block_info().time.plus_seconds(200),
                    end_time: app.block_info().time.plus_seconds(1200),
                    max_supply: Some(1000),
                    max_nfts_per_address: 2,
                    price: coin(500000, NATIVE_DENOM),
                    is_public: false,
                },
            };

            // execute add new phase msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_second_phase_msg,
                &[],
            );
            assert!(res.is_ok());

            // query all phases config from launchpad contract
            let phase_config_info: Vec<PhaseConfigResponse> = app
                .wrap()
                .query_wasm_smart(
                    Addr::unchecked(launchpad_address),
                    &QueryMsg::GetAllPhaseConfigs {},
                )
                .unwrap();

            assert_eq!(phase_config_info.len(), 2);
            assert_eq!(phase_config_info[0].phase_id, 1);
            assert_eq!(phase_config_info[1].phase_id, 2);
        }

        #[test]
        fn cannot_update_phase_because_the_launchpad_started() {
            // get integration test app and launchpad address
            let (mut app, launchpad_address) = create_launchpad();

            // ADD FIRST PHASE to the first position
            // prepare execute msg for adding new phase to launchpad
            let add_first_phase_msg = ExecuteMsg::AddMintPhase {
                after_phase_id: None,
                phase_data: PhaseData {
                    start_time: app.block_info().time.plus_seconds(140),
                    end_time: app.block_info().time.plus_seconds(600),
                    max_supply: Some(1000),
                    max_nfts_per_address: 2,
                    price: coin(500000, NATIVE_DENOM),
                    is_public: false,
                },
            };

            // execute add new phase msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_first_phase_msg,
                &[],
            );
            assert!(res.is_ok());

            // admin activate the luanchpad
            let activate_launchpad_msg = ExecuteMsg::ActivateLaunchpad {};
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &activate_launchpad_msg,
                &[],
            );
            assert!(res.is_ok());

            // prepare execute msg for updating phase config
            let update_phase_msg = ExecuteMsg::UpdateMintPhase {
                phase_id: 1,
                phase_data: PhaseData {
                    start_time: app.block_info().time.plus_seconds(200),
                    end_time: app.block_info().time.plus_seconds(210),
                    max_supply: Some(2000),
                    max_nfts_per_address: 2,
                    price: coin(500000, NATIVE_DENOM),
                    is_public: false,
                },
            };

            // execute update phase config msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address),
                &update_phase_msg,
                &[],
            );
            assert_eq!(
                res.unwrap_err().source().unwrap().to_string(),
                "Launchpad started"
            );
        }

        #[test]
        fn admin_can_update_phase_config() {
            // get integration test app and launchpad address
            let (mut app, launchpad_address) = create_launchpad();

            // ADD FIRST PHASE to the first position
            // prepare execute msg for adding new phase to launchpad
            let add_first_phase_msg = ExecuteMsg::AddMintPhase {
                after_phase_id: None,
                phase_data: PhaseData {
                    start_time: app.block_info().time.plus_seconds(200),
                    end_time: app.block_info().time.plus_seconds(210),
                    max_supply: Some(1000),
                    max_nfts_per_address: 2,
                    price: coin(500000, NATIVE_DENOM),
                    is_public: false,
                },
            };

            // execute add new phase msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_first_phase_msg,
                &[],
            );
            assert!(res.is_ok());

            // prepare execute msg for updating phase config
            let update_phase_msg = ExecuteMsg::UpdateMintPhase {
                phase_id: 1,
                phase_data: PhaseData {
                    start_time: app.block_info().time.plus_seconds(200),
                    end_time: app.block_info().time.plus_seconds(210),
                    max_supply: Some(2000),
                    max_nfts_per_address: 2,
                    price: coin(500000, NATIVE_DENOM),
                    is_public: false,
                },
            };

            // execute update phase config msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &update_phase_msg,
                &[],
            );
            println!("res: {:?}", res);
            assert!(res.is_ok());

            // query all phases config from launchpad contract
            let phase_config_info: Vec<PhaseConfigResponse> = app
                .wrap()
                .query_wasm_smart(
                    Addr::unchecked(launchpad_address),
                    &QueryMsg::GetAllPhaseConfigs {},
                )
                .unwrap();

            assert_eq!(phase_config_info.len(), 1);
            assert_eq!(phase_config_info[0].phase_id, 1);
            assert_eq!(phase_config_info[0].max_supply.unwrap(), 2000);
        }

        #[test]
        fn cannot_update_phase_config_because_not_admin() {
            // get integration test app and launchpad address
            let (mut app, launchpad_address) = create_launchpad();

            // ADD FIRST PHASE to the first position
            // prepare execute msg for adding new phase to launchpad
            let add_first_phase_msg = ExecuteMsg::AddMintPhase {
                after_phase_id: None,
                phase_data: PhaseData {
                    start_time: app.block_info().time.plus_seconds(200),
                    end_time: app.block_info().time.plus_seconds(210),
                    max_supply: Some(1000),
                    max_nfts_per_address: 2,
                    price: coin(500000, NATIVE_DENOM),
                    is_public: false,
                },
            };

            // execute add new phase msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_first_phase_msg,
                &[],
            );
            assert!(res.is_ok());

            // prepare execute msg for updating phase config
            let update_phase_msg = ExecuteMsg::UpdateMintPhase {
                phase_id: 1,
                phase_data: PhaseData {
                    start_time: app.block_info().time.plus_seconds(200),
                    end_time: app.block_info().time.plus_seconds(210),
                    max_supply: Some(2000),
                    max_nfts_per_address: 2,
                    price: coin(500000, NATIVE_DENOM),
                    is_public: false,
                },
            };

            // execute update phase config msg
            let res = app.execute_contract(
                Addr::unchecked(USER_1),
                Addr::unchecked(launchpad_address),
                &update_phase_msg,
                &[],
            );
            assert_eq!(
                res.unwrap_err().source().unwrap().to_string(),
                "Unauthorized"
            );
        }

        #[test]
        fn cannot_update_phase_config_because_phase_id_invalid() {
            // get integration test app and launchpad address
            let (mut app, launchpad_address) = create_launchpad();

            // ADD FIRST PHASE to the first position
            // prepare execute msg for adding new phase to launchpad
            let add_first_phase_msg = ExecuteMsg::AddMintPhase {
                after_phase_id: None,
                phase_data: PhaseData {
                    start_time: app.block_info().time.plus_seconds(200),
                    end_time: app.block_info().time.plus_seconds(210),
                    max_supply: Some(1000),
                    max_nfts_per_address: 2,
                    price: coin(500000, NATIVE_DENOM),
                    is_public: false,
                },
            };

            // execute add new phase msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_first_phase_msg,
                &[],
            );
            assert!(res.is_ok());

            // prepare execute msg for updating phase config
            let update_phase_msg = ExecuteMsg::UpdateMintPhase {
                phase_id: 2,
                phase_data: PhaseData {
                    start_time: app.block_info().time.plus_seconds(200),
                    end_time: app.block_info().time.plus_seconds(210),
                    max_supply: Some(2000),
                    max_nfts_per_address: 2,
                    price: coin(500000, NATIVE_DENOM),
                    is_public: false,
                },
            };

            // execute update phase config msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address),
                &update_phase_msg,
                &[],
            );
            assert_eq!(
                res.unwrap_err().source().unwrap().to_string(),
                "Invalid phase id"
            );
        }

        #[test]
        fn cannot_update_phase_config_because_invalid_phase_time() {
            // get integration test app and launchpad address
            let (mut app, launchpad_address) = create_launchpad();

            // ADD FIRST PHASE to the first position
            // prepare execute msg for adding new phase to launchpad
            let add_first_phase_msg = ExecuteMsg::AddMintPhase {
                after_phase_id: None,
                phase_data: PhaseData {
                    start_time: app.block_info().time.plus_seconds(200),
                    end_time: app.block_info().time.plus_seconds(210),
                    max_supply: Some(1000),
                    max_nfts_per_address: 2,
                    price: coin(500000, NATIVE_DENOM),
                    is_public: false,
                },
            };

            // execute add new phase msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_first_phase_msg,
                &[],
            );
            assert!(res.is_ok());

            // prepare execute msg for updating phase config
            let update_phase_msg = ExecuteMsg::UpdateMintPhase {
                phase_id: 1,
                phase_data: PhaseData {
                    start_time: app.block_info().time.minus_seconds(220),
                    end_time: app.block_info().time.plus_seconds(400),
                    max_supply: Some(2000),
                    max_nfts_per_address: 2,
                    price: coin(500000, NATIVE_DENOM),
                    is_public: false,
                },
            };

            // execute update phase config msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address),
                &update_phase_msg,
                &[],
            );
            assert_eq!(
                res.unwrap_err().source().unwrap().to_string(),
                "Invalid phase time"
            );
        }

        #[test]
        fn cannot_remove_phase_because_the_launchpad_started() {
            // get integration test app and launchpad address
            let (mut app, launchpad_address) = create_launchpad();

            // ADD FIRST PHASE to the first position
            // prepare execute msg for adding new phase to launchpad
            let add_first_phase_msg = ExecuteMsg::AddMintPhase {
                after_phase_id: None,
                phase_data: PhaseData {
                    start_time: app.block_info().time.plus_seconds(200),
                    end_time: app.block_info().time.plus_seconds(210),
                    max_supply: Some(1000),
                    max_nfts_per_address: 2,
                    price: coin(500000, NATIVE_DENOM),
                    is_public: false,
                },
            };

            // execute add new phase msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_first_phase_msg,
                &[],
            );
            assert!(res.is_ok());

            // ADD SECOND PHASE to the first position
            // prepare execute msg for adding new phase to launchpad
            let add_second_phase_msg = ExecuteMsg::AddMintPhase {
                after_phase_id: Some(1),
                phase_data: PhaseData {
                    start_time: app.block_info().time.plus_seconds(300),
                    end_time: app.block_info().time.plus_seconds(1200),
                    max_supply: Some(1000),
                    max_nfts_per_address: 2,
                    price: coin(500000, NATIVE_DENOM),
                    is_public: false,
                },
            };

            // execute add new phase msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_second_phase_msg,
                &[],
            );
            assert!(res.is_ok());

            // query all phases config from launchpad contract
            let phase_config_info: Vec<PhaseConfigResponse> = app
                .wrap()
                .query_wasm_smart(
                    Addr::unchecked(launchpad_address.clone()),
                    &QueryMsg::GetAllPhaseConfigs {},
                )
                .unwrap();

            assert_eq!(phase_config_info.len(), 2);
            assert_eq!(phase_config_info[0].phase_id, 1);
            assert_eq!(phase_config_info[1].phase_id, 2);

            // admin activate the launchpad
            let activate_msg = ExecuteMsg::ActivateLaunchpad {};
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &activate_msg,
                &[],
            );
            assert!(res.is_ok());

            // prepare execute msg for removing phase config
            let remove_phase_msg = ExecuteMsg::RemoveMintPhase { phase_id: 1 };

            // execute remove phase config msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address),
                &remove_phase_msg,
                &[],
            );
            assert_eq!(
                res.unwrap_err().source().unwrap().to_string(),
                "Launchpad started"
            );
        }

        #[test]
        fn admin_can_remove_mint_phase() {
            // get integration test app and launchpad address
            let (mut app, launchpad_address) = create_launchpad();

            // ADD FIRST PHASE to the first position
            // prepare execute msg for adding new phase to launchpad
            let add_first_phase_msg = ExecuteMsg::AddMintPhase {
                after_phase_id: None,
                phase_data: PhaseData {
                    start_time: app.block_info().time.plus_seconds(200),
                    end_time: app.block_info().time.plus_seconds(210),
                    max_supply: Some(1000),
                    max_nfts_per_address: 2,
                    price: coin(500000, NATIVE_DENOM),
                    is_public: false,
                },
            };

            // execute add new phase msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_first_phase_msg,
                &[],
            );
            assert!(res.is_ok());

            // ADD SECOND PHASE to the first position
            // prepare execute msg for adding new phase to launchpad
            let add_second_phase_msg = ExecuteMsg::AddMintPhase {
                after_phase_id: Some(1),
                phase_data: PhaseData {
                    start_time: app.block_info().time.plus_seconds(300),
                    end_time: app.block_info().time.plus_seconds(1200),
                    max_supply: Some(1000),
                    max_nfts_per_address: 2,
                    price: coin(500000, NATIVE_DENOM),
                    is_public: false,
                },
            };

            // execute add new phase msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_second_phase_msg,
                &[],
            );
            assert!(res.is_ok());

            // query all phases config from launchpad contract
            let phase_config_info: Vec<PhaseConfigResponse> = app
                .wrap()
                .query_wasm_smart(
                    Addr::unchecked(launchpad_address.clone()),
                    &QueryMsg::GetAllPhaseConfigs {},
                )
                .unwrap();

            assert_eq!(phase_config_info.len(), 2);
            assert_eq!(phase_config_info[0].phase_id, 1);
            assert_eq!(phase_config_info[1].phase_id, 2);

            // prepare execute msg for removing phase config
            let remove_phase_msg = ExecuteMsg::RemoveMintPhase { phase_id: 1 };

            // execute remove phase config msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &remove_phase_msg,
                &[],
            );
            assert!(res.is_ok());

            // query all phases config from launchpad contract
            let phase_config_info: Vec<PhaseConfigResponse> = app
                .wrap()
                .query_wasm_smart(
                    Addr::unchecked(launchpad_address),
                    &QueryMsg::GetAllPhaseConfigs {},
                )
                .unwrap();

            assert_eq!(phase_config_info.len(), 1);
            assert_eq!(phase_config_info[0].phase_id, 2);
        }

        #[test]
        fn cannot_remove_mint_phase_because_not_admin() {
            // get integration test app and launchpad address
            let (mut app, launchpad_address) = create_launchpad();

            // ADD FIRST PHASE to the first position
            // prepare execute msg for adding new phase to launchpad
            let add_first_phase_msg = ExecuteMsg::AddMintPhase {
                after_phase_id: None,
                phase_data: PhaseData {
                    start_time: app.block_info().time.plus_seconds(200),
                    end_time: app.block_info().time.plus_seconds(210),
                    max_supply: Some(1000),
                    max_nfts_per_address: 2,
                    price: coin(500000, NATIVE_DENOM),
                    is_public: false,
                },
            };

            // execute add new phase msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_first_phase_msg,
                &[],
            );
            assert!(res.is_ok());

            // prepare execute msg for removing phase config
            let remove_phase_msg = ExecuteMsg::RemoveMintPhase { phase_id: 1 };

            // execute remove phase config msg
            let res = app.execute_contract(
                Addr::unchecked(USER_1),
                Addr::unchecked(launchpad_address),
                &remove_phase_msg,
                &[],
            );

            assert_eq!(
                res.unwrap_err().source().unwrap().to_string(),
                "Unauthorized"
            );
        }

        #[test]
        fn cannot_remove_mint_phase_because_invalid_phase_id() {
            // get integration test app and launchpad address
            let (mut app, launchpad_address) = create_launchpad();

            // prepare execute msg for removing phase config
            let remove_phase_msg = ExecuteMsg::RemoveMintPhase { phase_id: 1 };

            // execute remove phase config msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address),
                &remove_phase_msg,
                &[],
            );
            assert_eq!(
                res.unwrap_err().source().unwrap().to_string(),
                "Invalid phase id"
            );
        }

        #[test]
        fn cannot_remove_mint_phase_id_0() {
            // get integration test app and launchpad address
            let (mut app, launchpad_address) = create_launchpad();

            // prepare execute msg for removing phase config
            let remove_phase_msg = ExecuteMsg::RemoveMintPhase { phase_id: 0 };

            // execute remove phase config msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address),
                &remove_phase_msg,
                &[],
            );
            assert_eq!(
                res.unwrap_err().source().unwrap().to_string(),
                "Invalid phase id"
            );
        }

        #[test]
        fn cannot_add_whitelist_because_not_admin() {
            // get integration test app and launchpad address
            let (mut app, launchpad_address) = create_launchpad();

            // ADD FIRST PHASE to the first position
            // prepare execute msg for adding new phase to launchpad
            let add_first_phase_msg = ExecuteMsg::AddMintPhase {
                after_phase_id: None,
                phase_data: PhaseData {
                    start_time: app.block_info().time.plus_seconds(200),
                    end_time: app.block_info().time.plus_seconds(1000),
                    max_supply: Some(1000),
                    max_nfts_per_address: 2,
                    price: coin(500000, NATIVE_DENOM),
                    is_public: false,
                },
            };

            // execute add new phase msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_first_phase_msg,
                &[],
            );
            assert!(res.is_ok());

            // prepare execute msg for adding new whitelist to launchpad
            let add_whitelist_msg = ExecuteMsg::AddWhitelist {
                whitelists: [ADMIN.to_string(), USER_1.to_string()].to_vec(),
                phase_id: 1,
            };

            // execute add new whitelist msg
            let res = app.execute_contract(
                Addr::unchecked(USER_1),
                Addr::unchecked(launchpad_address),
                &add_whitelist_msg,
                &[],
            );
            assert_eq!(
                res.unwrap_err().source().unwrap().to_string(),
                "Unauthorized"
            );
        }

        #[test]
        fn cannot_add_whitelist_because_the_launchpad_started() {
            // get integration test app and launchpad address
            let (mut app, launchpad_address) = create_launchpad();

            // ADD FIRST PHASE to the first position
            // prepare execute msg for adding new phase to launchpad
            let add_first_phase_msg = ExecuteMsg::AddMintPhase {
                after_phase_id: None,
                phase_data: PhaseData {
                    start_time: app.block_info().time.plus_seconds(200),
                    end_time: app.block_info().time.plus_seconds(1000),
                    max_supply: Some(1000),
                    max_nfts_per_address: 2,
                    price: coin(500000, NATIVE_DENOM),
                    is_public: false,
                },
            };

            // execute add new phase msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_first_phase_msg,
                &[],
            );
            assert!(res.is_ok());

            // admin activate the launchpad
            let activate_msg = ExecuteMsg::ActivateLaunchpad {};
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &activate_msg,
                &[],
            );
            assert!(res.is_ok());

            // prepare execute msg for adding new whitelist to launchpad
            let add_whitelist_msg = ExecuteMsg::AddWhitelist {
                whitelists: [ADMIN.to_string(), USER_1.to_string()].to_vec(),
                phase_id: 1,
            };

            // execute add new whitelist msg
            let res = app.execute_contract(
                Addr::unchecked(USER_1),
                Addr::unchecked(launchpad_address),
                &add_whitelist_msg,
                &[],
            );
            assert_eq!(
                res.unwrap_err().source().unwrap().to_string(),
                "Launchpad started"
            );
        }

        #[test]
        fn user_can_mint_after_admin_add_whitelist() {
            // get integration test app and launchpad address
            let (mut app, launchpad_address) = create_launchpad();

            // ADD FIRST PHASE to the first position
            // prepare execute msg for adding new phase to launchpad
            let add_first_phase_msg = ExecuteMsg::AddMintPhase {
                after_phase_id: None,
                phase_data: PhaseData {
                    start_time: app.block_info().time.plus_seconds(200),
                    end_time: app.block_info().time.plus_seconds(1000),
                    max_supply: Some(1000),
                    max_nfts_per_address: 2,
                    price: coin(500000, NATIVE_DENOM),
                    is_public: false,
                },
            };

            // execute add new phase msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_first_phase_msg,
                &[],
            );
            assert!(res.is_ok());

            // ADD SECOND PHASE to the first position
            // prepare execute msg for adding new phase to launchpad
            let add_second_phase_msg = ExecuteMsg::AddMintPhase {
                after_phase_id: Some(1),
                phase_data: PhaseData {
                    start_time: app.block_info().time.plus_seconds(1200),
                    end_time: app.block_info().time.plus_seconds(2000),
                    max_supply: Some(1000),
                    max_nfts_per_address: 1,
                    price: coin(500000, NATIVE_DENOM),
                    is_public: true,
                },
            };

            // execute add new phase msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_second_phase_msg,
                &[],
            );
            assert!(res.is_ok());

            // prepare execute msg for adding new whitelist to launchpad
            let add_whitelist_msg = ExecuteMsg::AddWhitelist {
                whitelists: [ADMIN.to_string(), USER_1.to_string()].to_vec(),
                phase_id: 1,
            };

            // execute add new whitelist msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_whitelist_msg,
                &[],
            );
            assert!(res.is_ok());

            // admin activate the launchpad
            let activate_msg = ExecuteMsg::ActivateLaunchpad {};
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &activate_msg,
                &[],
            );
            assert!(res.is_ok());

            // query mintable of user 1
            let res: Vec<MintableResponse> = app
                .wrap()
                .query_wasm_smart(
                    Addr::unchecked(launchpad_address.clone()),
                    &QueryMsg::Mintable {
                        user: USER_1.to_string(),
                    },
                )
                .unwrap();
            assert_eq!(
                res,
                vec![
                    MintableResponse {
                        phase_id: 1,
                        remaining_nfts: 2
                    },
                    MintableResponse {
                        phase_id: 2,
                        remaining_nfts: 1,
                    },
                ]
            );

            // query mintable of user 2
            let res: Vec<MintableResponse> = app
                .wrap()
                .query_wasm_smart(
                    Addr::unchecked(launchpad_address.clone()),
                    &QueryMsg::Mintable {
                        user: USER_2.to_string(),
                    },
                )
                .unwrap();
            assert_eq!(
                res,
                vec![
                    MintableResponse {
                        phase_id: 1,
                        remaining_nfts: 0
                    },
                    MintableResponse {
                        phase_id: 2,
                        remaining_nfts: 1,
                    },
                ]
            );

            // USER_1 try to mint nft
            // prepare execute msg for minting nft
            let mint_nft_msg = ExecuteMsg::Mint {
                phase_id: 1,
                amount: Option::from(1),
            };

            // execute mint nft msg will fail because phase is not active
            let res = app.execute_contract(
                Addr::unchecked(USER_1),
                Addr::unchecked(launchpad_address.clone()),
                &mint_nft_msg,
                &[],
            );
            assert_eq!(
                res.unwrap_err().source().unwrap().to_string(),
                "Phase is inactivated"
            );

            // change block time increase 400 seconds to make phase active
            app.set_block(BlockInfo {
                time: app.block_info().time.plus_seconds(400),
                height: app.block_info().height + 1,
                chain_id: app.block_info().chain_id,
            });

            // execute mint nft msg again
            let res = app.execute_contract(
                Addr::unchecked(USER_1),
                Addr::unchecked(launchpad_address.clone()),
                &mint_nft_msg,
                &[],
            );

            // the res will be fail because fund is not enough
            assert_eq!(
                res.unwrap_err().source().unwrap().to_string(),
                "Not enough funds"
            );

            // USER_1 try to mint nft again with enough fund
            // get the price of phase 1
            let phase_config_info: Vec<PhaseConfigResponse> = app
                .wrap()
                .query_wasm_smart(
                    Addr::unchecked(launchpad_address.clone()),
                    &QueryMsg::GetAllPhaseConfigs {},
                )
                .unwrap();

            // Mint 1000000000 native token to USER_1
            app.sudo(cw_multi_test::SudoMsg::Bank(
                cw_multi_test::BankSudo::Mint {
                    to_address: USER_1.to_string(),
                    amount: vec![Coin {
                        amount: 1000000000u128.into(),
                        denom: NATIVE_DENOM.to_string(),
                    }],
                },
            ))
            .unwrap();

            let res = app.execute_contract(
                Addr::unchecked(USER_1),
                Addr::unchecked(launchpad_address.clone()),
                &mint_nft_msg,
                &[Coin {
                    denom: NATIVE_DENOM.to_string(),
                    amount: phase_config_info[0].price.amount,
                }],
            );
            assert!(res.is_ok());

            // query mintable of user 1
            let res: Vec<MintableResponse> = app
                .wrap()
                .query_wasm_smart(
                    Addr::unchecked(launchpad_address.clone()),
                    &QueryMsg::Mintable {
                        user: USER_1.to_string(),
                    },
                )
                .unwrap();
            assert_eq!(
                res,
                vec![
                    MintableResponse {
                        phase_id: 1,
                        remaining_nfts: 1
                    },
                    MintableResponse {
                        phase_id: 2,
                        remaining_nfts: 1,
                    },
                ]
            );

            // query the nft info from collection contract in launchpad info
            // prepare query msg for getting nft info
            let nft_info_msg = Cw721QueryMsg::Tokens {
                owner: USER_1.to_string(),
                start_after: None,
                limit: None,
            };

            // query launchpad info from launchpad contract
            let launchpad_info: LaunchpadInfo = app
                .wrap()
                .query_wasm_smart(
                    Addr::unchecked(launchpad_address),
                    &QueryMsg::GetLaunchpadInfo {},
                )
                .unwrap();

            // query nft info
            let nft_info: TokensResponse = app
                .wrap()
                .query_wasm_smart(
                    Addr::unchecked(launchpad_info.collection_address),
                    &nft_info_msg,
                )
                .unwrap();

            assert_eq!(nft_info.tokens.len(), 1);
        }

        #[test]
        fn admin_can_remove_whitelist() {
            // get integration test app and launchpad address
            let (mut app, launchpad_address) = create_launchpad();

            // ADD FIRST PHASE to the first position
            // prepare execute msg for adding new phase to launchpad
            let add_first_phase_msg = ExecuteMsg::AddMintPhase {
                after_phase_id: None,
                phase_data: PhaseData {
                    start_time: app.block_info().time.plus_seconds(200),
                    end_time: app.block_info().time.plus_seconds(210),
                    max_supply: Some(1000),
                    max_nfts_per_address: 2,
                    price: coin(500000, NATIVE_DENOM),
                    is_public: false,
                },
            };

            // execute add new phase msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_first_phase_msg,
                &[],
            );
            assert!(res.is_ok());

            // ADD SECOND PHASE to the first position
            // prepare execute msg for adding new phase to launchpad
            let add_second_phase_msg = ExecuteMsg::AddMintPhase {
                after_phase_id: Some(1),
                phase_data: PhaseData {
                    start_time: app.block_info().time.plus_seconds(300),
                    end_time: app.block_info().time.plus_seconds(1200),
                    max_supply: Some(1000),
                    max_nfts_per_address: 2,
                    price: coin(500000, NATIVE_DENOM),
                    is_public: false,
                },
            };

            // execute add new phase msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_second_phase_msg,
                &[],
            );
            assert!(res.is_ok());

            // prepare execute msg for adding new whitelist to launchpad
            let add_whitelist_msg = ExecuteMsg::AddWhitelist {
                whitelists: [ADMIN.to_string(), USER_1.to_string()].to_vec(),
                phase_id: 1,
            };

            // execute add new whitelist msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_whitelist_msg,
                &[],
            );
            assert!(res.is_ok());

            // prepare execute msg for removing whitelist from launchpad
            let remove_whitelist_msg = ExecuteMsg::RemoveWhitelist {
                addresses: [USER_1.to_string()].to_vec(),
                phase_id: 1,
            };

            // execute remove whitelist msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &remove_whitelist_msg,
                &[],
            );
            assert!(res.is_ok());

            // remove the last phase
            let remove_phase_msg = ExecuteMsg::RemoveMintPhase { phase_id: 2 };

            // execute remove phase msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &remove_phase_msg,
                &[],
            );
            assert!(res.is_ok());

            // query phase config info
            let phase_config_info: Vec<PhaseConfigResponse> = app
                .wrap()
                .query_wasm_smart(
                    Addr::unchecked(launchpad_address),
                    &QueryMsg::GetAllPhaseConfigs {},
                )
                .unwrap();

            assert_eq!(phase_config_info.len(), 1);
        }

        #[test]
        fn cannot_remove_whitelist_because_the_launchpad_started() {
            // get integration test app and launchpad address
            let (mut app, launchpad_address) = create_launchpad();

            // ADD FIRST PHASE to the first position
            // prepare execute msg for adding new phase to launchpad
            let add_first_phase_msg = ExecuteMsg::AddMintPhase {
                after_phase_id: None,
                phase_data: PhaseData {
                    start_time: app.block_info().time.plus_seconds(200),
                    end_time: app.block_info().time.plus_seconds(210),
                    max_supply: Some(1000),
                    max_nfts_per_address: 2,
                    price: coin(500000, NATIVE_DENOM),
                    is_public: false,
                },
            };

            // execute add new phase msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_first_phase_msg,
                &[],
            );
            assert!(res.is_ok());

            // prepare execute msg for adding new whitelist to launchpad
            let add_whitelist_msg = ExecuteMsg::AddWhitelist {
                whitelists: [ADMIN.to_string(), USER_1.to_string()].to_vec(),
                phase_id: 1,
            };

            // execute add new whitelist msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_whitelist_msg,
                &[],
            );
            assert!(res.is_ok());

            // admin activate the launchpad
            let activate_launchpad_msg = ExecuteMsg::ActivateLaunchpad {};
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &activate_launchpad_msg,
                &[],
            );
            assert!(res.is_ok());

            // prepare execute msg for removing whitelist from launchpad
            let remove_whitelist_msg = ExecuteMsg::RemoveWhitelist {
                addresses: [USER_1.to_string()].to_vec(),
                phase_id: 1,
            };

            // execute remove whitelist msg
            let res = app.execute_contract(
                Addr::unchecked(USER_1),
                Addr::unchecked(launchpad_address),
                &remove_whitelist_msg,
                &[],
            );
            assert_eq!(
                res.unwrap_err().source().unwrap().to_string(),
                "Launchpad started"
            );
        }

        #[test]
        fn cannot_remove_whitelist_because_not_admin() {
            // get integration test app and launchpad address
            let (mut app, launchpad_address) = create_launchpad();

            // ADD FIRST PHASE to the first position
            // prepare execute msg for adding new phase to launchpad
            let add_first_phase_msg = ExecuteMsg::AddMintPhase {
                after_phase_id: None,
                phase_data: PhaseData {
                    start_time: app.block_info().time.plus_seconds(200),
                    end_time: app.block_info().time.plus_seconds(210),
                    max_supply: Some(1000),
                    max_nfts_per_address: 2,
                    price: coin(500000, NATIVE_DENOM),
                    is_public: false,
                },
            };

            // execute add new phase msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_first_phase_msg,
                &[],
            );
            assert!(res.is_ok());

            // prepare execute msg for adding new whitelist to launchpad
            let add_whitelist_msg = ExecuteMsg::AddWhitelist {
                whitelists: [ADMIN.to_string(), USER_1.to_string()].to_vec(),
                phase_id: 1,
            };

            // execute add new whitelist msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_whitelist_msg,
                &[],
            );
            assert!(res.is_ok());

            // prepare execute msg for removing whitelist from launchpad
            let remove_whitelist_msg = ExecuteMsg::RemoveWhitelist {
                addresses: [USER_1.to_string()].to_vec(),
                phase_id: 1,
            };

            // execute remove whitelist msg
            let res = app.execute_contract(
                Addr::unchecked(USER_1),
                Addr::unchecked(launchpad_address),
                &remove_whitelist_msg,
                &[],
            );
            assert_eq!(
                res.unwrap_err().source().unwrap().to_string(),
                "Unauthorized"
            );
        }

        #[test]
        fn cannot_add_new_phase_follow_a_phase_id_because_phase_time_invalid() {
            // get integration test app and launchpad address
            let (mut app, launchpad_address) = create_launchpad();

            // ADD FIRST PHASE to the first position
            // prepare execute msg for adding new phase to launchpad
            let add_first_phase_msg = ExecuteMsg::AddMintPhase {
                after_phase_id: None,
                phase_data: PhaseData {
                    start_time: app.block_info().time.plus_seconds(200),
                    end_time: app.block_info().time.plus_seconds(210),
                    max_supply: Some(1000),
                    max_nfts_per_address: 2,
                    price: coin(500000, NATIVE_DENOM),
                    is_public: false,
                },
            };

            // execute add new phase msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_first_phase_msg,
                &[],
            );
            assert!(res.is_ok());

            // ADD SECOND PHASE to the first position
            // prepare execute msg for adding new phase to launchpad
            let add_phase_msg = ExecuteMsg::AddMintPhase {
                after_phase_id: Some(0),
                phase_data: PhaseData {
                    start_time: app.block_info().time.minus_seconds(100),
                    end_time: app.block_info().time.plus_seconds(200),
                    max_supply: Some(1000),
                    max_nfts_per_address: 2,
                    price: coin(500000, NATIVE_DENOM),
                    is_public: false,
                },
            };

            // execute add new phase msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address),
                &add_phase_msg,
                &[],
            );

            // the res will be fail because phase time is invalid
            assert_eq!(
                res.unwrap_err().source().unwrap().to_string(),
                "Invalid phase time"
            );
        }

        #[test]
        fn cannot_mint_because_not_in_whitelist() {
            // get integration test app and launchpad address
            let (mut app, launchpad_address) = create_launchpad();

            // ADD FIRST PHASE to the first position
            // prepare execute msg for adding new phase to launchpad
            let add_first_phase_msg = ExecuteMsg::AddMintPhase {
                after_phase_id: None,
                phase_data: PhaseData {
                    start_time: app.block_info().time.plus_seconds(200),
                    end_time: app.block_info().time.plus_seconds(210),
                    max_supply: Some(1000),
                    max_nfts_per_address: 2,
                    price: coin(500000, NATIVE_DENOM),
                    is_public: false,
                },
            };

            // execute add new phase msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_first_phase_msg,
                &[],
            );
            assert!(res.is_ok());

            // prepare execute msg for adding new whitelist to launchpad
            let add_whitelist_msg = ExecuteMsg::AddWhitelist {
                whitelists: [ADMIN.to_string()].to_vec(),
                phase_id: 1,
            };

            // execute add new whitelist msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_whitelist_msg,
                &[],
            );
            assert!(res.is_ok());

            // admin activate the launchpad
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &ExecuteMsg::ActivateLaunchpad {},
                &[],
            );
            assert!(res.is_ok());

            // prepare execute msg for minting nft
            let mint_msg = ExecuteMsg::Mint {
                phase_id: 1,
                amount: Option::from(1),
            };

            // execute mint msg
            let res = app.execute_contract(
                Addr::unchecked(USER_1),
                Addr::unchecked(launchpad_address),
                &mint_msg,
                &[],
            );

            // the res will be fail because user is not in whitelist
            assert_eq!(
                res.unwrap_err().source().unwrap().to_string(),
                "Unauthorized"
            );
        }

        #[test]
        fn cannot_mint_because_max_supply_of_phase_reached() {
            // get integration test app and launchpad address
            let (mut app, launchpad_address) = create_launchpad();

            // ADD FIRST PHASE to the first position
            // prepare execute msg for adding new phase to launchpad
            let add_first_phase_msg = ExecuteMsg::AddMintPhase {
                after_phase_id: None,
                phase_data: PhaseData {
                    start_time: app.block_info().time.plus_seconds(200),
                    end_time: app.block_info().time.plus_seconds(610),
                    max_supply: Some(1),
                    max_nfts_per_address: 2,
                    price: coin(500000, NATIVE_DENOM),
                    is_public: false,
                },
            };

            // execute add new phase msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_first_phase_msg,
                &[],
            );
            assert!(res.is_ok());

            // prepare execute msg for adding new whitelist to launchpad
            let add_whitelist_msg = ExecuteMsg::AddWhitelist {
                whitelists: [ADMIN.to_string(), USER_1.to_string()].to_vec(),
                phase_id: 1,
            };

            // execute add new whitelist msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_whitelist_msg,
                &[],
            );
            assert!(res.is_ok());

            // admin activate the launchpad
            let activate_msg = ExecuteMsg::ActivateLaunchpad {};
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &activate_msg,
                &[],
            );
            assert!(res.is_ok());

            // change block time increase 400 seconds to make phase active
            app.set_block(BlockInfo {
                time: app.block_info().time.plus_seconds(400),
                height: app.block_info().height + 1,
                chain_id: app.block_info().chain_id,
            });

            // get the price of phase 1
            let phase_config_info: Vec<PhaseConfigResponse> = app
                .wrap()
                .query_wasm_smart(
                    Addr::unchecked(launchpad_address.clone()),
                    &QueryMsg::GetAllPhaseConfigs {},
                )
                .unwrap();

            // Mint 1000000000 native token to USER_1
            app.sudo(cw_multi_test::SudoMsg::Bank(
                cw_multi_test::BankSudo::Mint {
                    to_address: USER_1.to_string(),
                    amount: vec![Coin {
                        amount: 1000000000u128.into(),
                        denom: NATIVE_DENOM.to_string(),
                    }],
                },
            ))
            .unwrap();

            // prepare execute msg for minting nft
            let mint_msg = ExecuteMsg::Mint {
                phase_id: 1,
                amount: Option::from(1),
            };

            // execute mint msg
            let res = app.execute_contract(
                Addr::unchecked(USER_1),
                Addr::unchecked(launchpad_address.clone()),
                &mint_msg,
                &[Coin {
                    denom: NATIVE_DENOM.to_string(),
                    amount: phase_config_info[0].price.amount,
                }],
            );
            assert!(res.is_ok());

            // execute mint msg
            let res = app.execute_contract(
                Addr::unchecked(USER_1),
                Addr::unchecked(launchpad_address),
                &mint_msg,
                &[Coin {
                    denom: NATIVE_DENOM.to_string(),
                    amount: phase_config_info[0].price.amount,
                }],
            );
            // the res will be fail because max supply of phase reached
            assert_eq!(
                res.unwrap_err().source().unwrap().to_string(),
                "Max supply reached"
            );
        }

        #[test]
        fn cannot_mint_because_max_supply_of_launchpad_reached() {
            // get integration test app and launchpad address
            let (mut app, launchpad_address) = create_launchpad_with_number_supply(1);

            // ADD FIRST PHASE to the first position
            // prepare execute msg for adding new phase to launchpad
            let add_first_phase_msg = ExecuteMsg::AddMintPhase {
                after_phase_id: None,
                phase_data: PhaseData {
                    start_time: app.block_info().time.plus_seconds(200),
                    end_time: app.block_info().time.plus_seconds(610),
                    max_supply: Some(1000),
                    max_nfts_per_address: 2,
                    price: coin(500000, NATIVE_DENOM),
                    is_public: false,
                },
            };

            // execute add new phase msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_first_phase_msg,
                &[],
            );
            assert!(res.is_ok());

            // prepare execute msg for adding new whitelist to launchpad
            let add_whitelist_msg = ExecuteMsg::AddWhitelist {
                whitelists: [ADMIN.to_string(), USER_1.to_string()].to_vec(),
                phase_id: 1,
            };

            // execute add new whitelist msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_whitelist_msg,
                &[],
            );
            assert!(res.is_ok());

            // admin activate launchpad
            let activate_msg = ExecuteMsg::ActivateLaunchpad {};
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &activate_msg,
                &[],
            );
            assert!(res.is_ok());

            // change block time increase 400 seconds to make phase active
            app.set_block(BlockInfo {
                time: app.block_info().time.plus_seconds(400),
                height: app.block_info().height + 1,
                chain_id: app.block_info().chain_id,
            });

            // get the price of phase 1
            let phase_config_info: Vec<PhaseConfigResponse> = app
                .wrap()
                .query_wasm_smart(
                    Addr::unchecked(launchpad_address.clone()),
                    &QueryMsg::GetAllPhaseConfigs {},
                )
                .unwrap();

            // Mint 1000000000 native token to USER_1
            app.sudo(cw_multi_test::SudoMsg::Bank(
                cw_multi_test::BankSudo::Mint {
                    to_address: USER_1.to_string(),
                    amount: vec![Coin {
                        amount: 1000000000u128.into(),
                        denom: NATIVE_DENOM.to_string(),
                    }],
                },
            ))
            .unwrap();

            // prepare execute msg for minting nft
            let mint_msg = ExecuteMsg::Mint {
                phase_id: 1,
                amount: Option::from(1),
            };

            // execute mint msg
            let res = app.execute_contract(
                Addr::unchecked(USER_1),
                Addr::unchecked(launchpad_address.clone()),
                &mint_msg,
                &[Coin {
                    denom: NATIVE_DENOM.to_string(),
                    amount: phase_config_info[0].price.amount,
                }],
            );
            assert!(res.is_ok());

            // execute mint msg
            let res = app.execute_contract(
                Addr::unchecked(USER_1),
                Addr::unchecked(launchpad_address),
                &mint_msg,
                &[Coin {
                    denom: NATIVE_DENOM.to_string(),
                    amount: phase_config_info[0].price.amount,
                }],
            );
            // the res will be fail because max supply of phase reached
            assert_eq!(
                res.unwrap_err().source().unwrap().to_string(),
                "Max supply reached"
            );
        }

        #[test]
        fn cannot_mint_because_user_mint_too_much() {
            // get integration test app and launchpad address
            let (mut app, launchpad_address) = create_launchpad();

            // ADD FIRST PHASE to the first position
            // prepare execute msg for adding new phase to launchpad
            let add_first_phase_msg = ExecuteMsg::AddMintPhase {
                after_phase_id: None,
                phase_data: PhaseData {
                    start_time: app.block_info().time.plus_seconds(200),
                    end_time: app.block_info().time.plus_seconds(610),
                    max_supply: Some(10),
                    max_nfts_per_address: 1,
                    price: coin(500000, NATIVE_DENOM),
                    is_public: false,
                },
            };

            // execute add new phase msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_first_phase_msg,
                &[],
            );
            assert!(res.is_ok());

            // prepare execute msg for adding new whitelist to launchpad
            let add_whitelist_msg = ExecuteMsg::AddWhitelist {
                whitelists: [ADMIN.to_string(), USER_1.to_string()].to_vec(),
                phase_id: 1,
            };

            // execute add new whitelist msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_whitelist_msg,
                &[],
            );
            assert!(res.is_ok());

            // admin activate the launchpad
            let activate_msg = ExecuteMsg::ActivateLaunchpad {};
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &activate_msg,
                &[],
            );
            assert!(res.is_ok());

            // change block time increase 400 seconds to make phase active
            app.set_block(BlockInfo {
                time: app.block_info().time.plus_seconds(400),
                height: app.block_info().height + 1,
                chain_id: app.block_info().chain_id,
            });

            // get the price of phase 1
            let phase_config_info: Vec<PhaseConfigResponse> = app
                .wrap()
                .query_wasm_smart(
                    Addr::unchecked(launchpad_address.clone()),
                    &QueryMsg::GetAllPhaseConfigs {},
                )
                .unwrap();

            // Mint 1000000000 native token to USER_1
            app.sudo(cw_multi_test::SudoMsg::Bank(
                cw_multi_test::BankSudo::Mint {
                    to_address: USER_1.to_string(),
                    amount: vec![Coin {
                        amount: 1000000000u128.into(),
                        denom: NATIVE_DENOM.to_string(),
                    }],
                },
            ))
            .unwrap();

            // prepare execute msg for minting nft
            let mint_msg = ExecuteMsg::Mint {
                phase_id: 1,
                amount: Option::from(1),
            };

            // execute mint msg
            let res = app.execute_contract(
                Addr::unchecked(USER_1),
                Addr::unchecked(launchpad_address.clone()),
                &mint_msg,
                &[Coin {
                    denom: NATIVE_DENOM.to_string(),
                    amount: phase_config_info[0].price.amount,
                }],
            );
            assert!(res.is_ok());

            // execute mint msg
            let res = app.execute_contract(
                Addr::unchecked(USER_1),
                Addr::unchecked(launchpad_address),
                &mint_msg,
                &[Coin {
                    denom: NATIVE_DENOM.to_string(),
                    amount: phase_config_info[0].price.amount,
                }],
            );
            // the res will be fail because max supply of phase reached
            assert_eq!(
                res.unwrap_err().source().unwrap().to_string(),
                "User minted too much nfts"
            );
        }

        #[test]
        fn the_token_id_of_nfts_is_unique() {
            // get integration test app and launchpad address
            let (mut app, launchpad_address) = create_launchpad_with_number_supply(20);

            // ADD FIRST PHASE to the first position
            // prepare execute msg for adding new phase to launchpad
            let add_first_phase_msg = ExecuteMsg::AddMintPhase {
                after_phase_id: None,
                phase_data: PhaseData {
                    start_time: app.block_info().time.plus_seconds(200),
                    end_time: app.block_info().time.plus_seconds(610),
                    max_supply: Some(20),
                    max_nfts_per_address: 10,
                    price: coin(500, NATIVE_DENOM),
                    is_public: false,
                },
            };

            // execute add new phase msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_first_phase_msg,
                &[],
            );
            assert!(res.is_ok());

            // prepare execute msg for adding new whitelist to launchpad
            let add_whitelist_msg = ExecuteMsg::AddWhitelist {
                whitelists: [ADMIN.to_string(), USER_1.to_string()].to_vec(),
                phase_id: 1,
            };

            // execute add new whitelist msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_whitelist_msg,
                &[],
            );
            assert!(res.is_ok());

            // admin activate the launchpad
            let activate_msg = ExecuteMsg::ActivateLaunchpad {};
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &activate_msg,
                &[],
            );
            assert!(res.is_ok());

            // change block time increase 400 seconds to make phase active
            app.set_block(BlockInfo {
                time: app.block_info().time.plus_seconds(400),
                height: app.block_info().height + 1,
                chain_id: app.block_info().chain_id,
            });

            // get the price of phase 1
            let phase_config_info: Vec<PhaseConfigResponse> = app
                .wrap()
                .query_wasm_smart(
                    Addr::unchecked(launchpad_address.clone()),
                    &QueryMsg::GetAllPhaseConfigs {},
                )
                .unwrap();

            // Mint 1000000000 native token to USER_1
            app.sudo(cw_multi_test::SudoMsg::Bank(
                cw_multi_test::BankSudo::Mint {
                    to_address: USER_1.to_string(),
                    amount: vec![Coin {
                        amount: 1000000000u128.into(),
                        denom: NATIVE_DENOM.to_string(),
                    }],
                },
            ))
            .unwrap();

            // we need an array to store the minted token ids of nfts
            let mut token_ids: Vec<String> = Vec::new();

            // let's USER1 mint 10 nfts
            for _ in 0..10 {
                // prepare execute msg for minting nft
                let mint_msg = ExecuteMsg::Mint {
                    phase_id: 1,
                    amount: Option::from(1),
                };

                // execute mint msg
                let res = app.execute_contract(
                    Addr::unchecked(USER_1),
                    Addr::unchecked(launchpad_address.clone()),
                    &mint_msg,
                    &[Coin {
                        denom: NATIVE_DENOM.to_string(),
                        amount: phase_config_info[0].price.amount,
                    }],
                );
                assert!(res.is_ok());

                // get the token id of the minted nft
                let token_id = &res.unwrap().events[3].attributes[4].value;

                // check if the token id is unique
                assert!(!token_ids.contains(token_id));

                // push the token id to the array
                token_ids.push(token_id.to_string());
            }

            // let's ADMIN mint 10 nfts again
            for _ in 0..10 {
                // prepare execute msg for minting nft
                let mint_msg = ExecuteMsg::Mint {
                    phase_id: 1,
                    amount: Option::from(1),
                };

                // execute mint msg
                let res = app.execute_contract(
                    Addr::unchecked(ADMIN),
                    Addr::unchecked(launchpad_address.clone()),
                    &mint_msg,
                    &[Coin {
                        denom: NATIVE_DENOM.to_string(),
                        amount: phase_config_info[0].price.amount,
                    }],
                );
                assert!(res.is_ok());

                // get the token id of the minted nft
                let token_id = &res.unwrap().events[3].attributes[4].value;

                // check if the token id is unique
                assert!(!token_ids.contains(token_id));

                // push the token id to the array
                token_ids.push(token_id.to_string());
            }

            // assert tpken_ids array correct
            let expected_ids = [
                "15", "16", "12", "5", "8", "1", "7", "6", "9", "18", "4", "19", "20", "10", "2",
                "14", "13", "17", "11", "3",
            ]
            .to_vec();
            assert_eq!(token_ids, expected_ids);
        }

        #[test]
        fn the_token_id_of_nfts_is_unique_with_reseved_tokens() {
            // get integration test app and launchpad address
            let (mut app, launchpad_address) = create_launchpad_with_reseved_list();

            // ADD FIRST PHASE to the first position
            // prepare execute msg for adding new phase to launchpad
            let add_first_phase_msg = ExecuteMsg::AddMintPhase {
                after_phase_id: None,
                phase_data: PhaseData {
                    start_time: app.block_info().time.plus_seconds(200),
                    end_time: app.block_info().time.plus_seconds(610),
                    max_supply: Some(15),
                    max_nfts_per_address: 10,
                    price: coin(500, NATIVE_DENOM),
                    is_public: false,
                },
            };

            // execute add new phase msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_first_phase_msg,
                &[],
            );
            assert!(res.is_ok());

            // prepare execute msg for adding new whitelist to launchpad
            let add_whitelist_msg = ExecuteMsg::AddWhitelist {
                whitelists: [ADMIN.to_string(), USER_1.to_string()].to_vec(),
                phase_id: 1,
            };

            // execute add new whitelist msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_whitelist_msg,
                &[],
            );
            assert!(res.is_ok());

            // admin activate the launchpad
            let activate_msg = ExecuteMsg::ActivateLaunchpad {};
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &activate_msg,
                &[],
            );
            assert!(res.is_ok());

            // change block time increase 400 seconds to make phase active
            app.set_block(BlockInfo {
                time: app.block_info().time.plus_seconds(400),
                height: app.block_info().height + 1,
                chain_id: app.block_info().chain_id,
            });

            // get the price of phase 1
            let phase_config_info: Vec<PhaseConfigResponse> = app
                .wrap()
                .query_wasm_smart(
                    Addr::unchecked(launchpad_address.clone()),
                    &QueryMsg::GetAllPhaseConfigs {},
                )
                .unwrap();

            // Mint 1000000000 native token to USER_1
            app.sudo(cw_multi_test::SudoMsg::Bank(
                cw_multi_test::BankSudo::Mint {
                    to_address: USER_1.to_string(),
                    amount: vec![Coin {
                        amount: 1000000000u128.into(),
                        denom: NATIVE_DENOM.to_string(),
                    }],
                },
            ))
            .unwrap();

            // we need an array to store the minted token ids of nfts
            let mut token_ids: Vec<String> = Vec::new();

            // let's USER1 mint 10 nfts
            for _ in 0..10 {
                // prepare execute msg for minting nft
                let mint_msg = ExecuteMsg::Mint {
                    phase_id: 1,
                    amount: Option::from(1),
                };

                // execute mint msg
                let res = app.execute_contract(
                    Addr::unchecked(USER_1),
                    Addr::unchecked(launchpad_address.clone()),
                    &mint_msg,
                    &[Coin {
                        denom: NATIVE_DENOM.to_string(),
                        amount: phase_config_info[0].price.amount,
                    }],
                );
                assert!(res.is_ok());

                // get the token id of the minted nft
                let token_id = &res.unwrap().events[3].attributes[4].value;

                // check if the token id is unique
                assert!(!token_ids.contains(token_id));

                // push the token id to the array
                token_ids.push(token_id.to_string());
            }

            // let's ADMIN mint 10 nfts again
            for _ in 0..5 {
                // prepare execute msg for minting nft
                let mint_msg = ExecuteMsg::Mint {
                    phase_id: 1,
                    amount: Option::from(1),
                };

                // execute mint msg
                let res = app.execute_contract(
                    Addr::unchecked(ADMIN),
                    Addr::unchecked(launchpad_address.clone()),
                    &mint_msg,
                    &[Coin {
                        denom: NATIVE_DENOM.to_string(),
                        amount: phase_config_info[0].price.amount,
                    }],
                );
                assert!(res.is_ok());

                // get the token id of the minted nft
                let token_id = &res.unwrap().events[3].attributes[4].value;

                // check if the token id is unique
                assert!(!token_ids.contains(token_id));

                // push the token id to the array
                token_ids.push(token_id.to_string());
            }

            // assert tpken_ids array correct
            let expected_ids = [
                "13", "14", "9", "4", "17", "1", "18", "15", "19", "8", "16", "12", "10", "7", "11",
            ]
            .to_vec();
            assert_eq!(token_ids, expected_ids);
        }

        #[test]
        fn the_token_id_of_nfts_is_unique_with_offset() {
            // get integration test app and launchpad address
            let (mut app, launchpad_address) = create_launchpad_with_number_supply(20);

            // ADD FIRST PHASE to the first position
            // prepare execute msg for adding new phase to launchpad
            let add_first_phase_msg = ExecuteMsg::AddMintPhase {
                after_phase_id: None,
                phase_data: PhaseData {
                    start_time: app.block_info().time.plus_seconds(200),
                    end_time: app.block_info().time.plus_seconds(610),
                    max_supply: Some(20),
                    max_nfts_per_address: 10,
                    price: coin(500, NATIVE_DENOM),
                    is_public: false,
                },
            };

            // execute add new phase msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_first_phase_msg,
                &[],
            );
            assert!(res.is_ok());

            // prepare execute msg for adding new whitelist to launchpad
            let add_whitelist_msg = ExecuteMsg::AddWhitelist {
                whitelists: [ADMIN.to_string(), USER_1.to_string()].to_vec(),
                phase_id: 1,
            };

            // execute add new whitelist msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_whitelist_msg,
                &[],
            );
            assert!(res.is_ok());

            // set offset to 10
            let set_offset_msg = ExecuteMsg::UpdateTokenIdOffset { offset: 10 };

            // execute set offset msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &set_offset_msg,
                &[],
            );
            assert!(res.is_ok());

            // admin activate the launchpad
            let activate_msg = ExecuteMsg::ActivateLaunchpad {};
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &activate_msg,
                &[],
            );
            assert!(res.is_ok());

            // change block time increase 400 seconds to make phase active
            app.set_block(BlockInfo {
                time: app.block_info().time.plus_seconds(400),
                height: app.block_info().height + 1,
                chain_id: app.block_info().chain_id,
            });

            // get the price of phase 1
            let phase_config_info: Vec<PhaseConfigResponse> = app
                .wrap()
                .query_wasm_smart(
                    Addr::unchecked(launchpad_address.clone()),
                    &QueryMsg::GetAllPhaseConfigs {},
                )
                .unwrap();

            // Mint 1000000000 native token to USER_1
            app.sudo(cw_multi_test::SudoMsg::Bank(
                cw_multi_test::BankSudo::Mint {
                    to_address: USER_1.to_string(),
                    amount: vec![Coin {
                        amount: 1000000000u128.into(),
                        denom: NATIVE_DENOM.to_string(),
                    }],
                },
            ))
            .unwrap();

            // we need an array to store the minted token ids of nfts
            let mut token_ids: Vec<String> = Vec::new();

            // let's USER1 mint 10 nfts
            for _ in 0..10 {
                // prepare execute msg for minting nft
                let mint_msg = ExecuteMsg::Mint {
                    phase_id: 1,
                    amount: Option::from(1),
                };

                // execute mint msg
                let res = app.execute_contract(
                    Addr::unchecked(USER_1),
                    Addr::unchecked(launchpad_address.clone()),
                    &mint_msg,
                    &[Coin {
                        denom: NATIVE_DENOM.to_string(),
                        amount: phase_config_info[0].price.amount,
                    }],
                );
                assert!(res.is_ok());

                // get the token id of the minted nft
                let token_id = &res.unwrap().events[3].attributes[4].value;

                // check if the token id is unique
                assert!(!token_ids.contains(token_id));

                // push the token id to the array
                token_ids.push(token_id.to_string());
            }

            // let's ADMIN mint 10 nfts again
            for _ in 0..10 {
                // prepare execute msg for minting nft
                let mint_msg = ExecuteMsg::Mint {
                    phase_id: 1,
                    amount: Option::from(1),
                };

                // execute mint msg
                let res = app.execute_contract(
                    Addr::unchecked(ADMIN),
                    Addr::unchecked(launchpad_address.clone()),
                    &mint_msg,
                    &[Coin {
                        denom: NATIVE_DENOM.to_string(),
                        amount: phase_config_info[0].price.amount,
                    }],
                );
                assert!(res.is_ok());

                // get the token id of the minted nft
                let token_id = &res.unwrap().events[3].attributes[4].value;

                // check if the token id is unique
                assert!(!token_ids.contains(token_id));

                // push the token id to the array
                token_ids.push(token_id.to_string());
            }

            // assert tpken_ids array correct
            let expected_ids = [
                "25", "26", "22", "15", "18", "11", "17", "16", "19", "28", "14", "29", "30", "20",
                "12", "24", "23", "27", "21", "13",
            ]
            .to_vec();
            assert_eq!(token_ids, expected_ids);
        }

        #[test]
        fn mint_100_tokens() {
            // get integration test app and launchpad address
            let (mut app, launchpad_address) = create_launchpad_with_number_supply(199);

            // ADD FIRST PHASE to the first position
            // prepare execute msg for adding new phase to launchpad
            let add_first_phase_msg = ExecuteMsg::AddMintPhase {
                after_phase_id: None,
                phase_data: PhaseData {
                    start_time: app.block_info().time.plus_seconds(200),
                    end_time: app.block_info().time.plus_seconds(1610),
                    max_supply: Some(100),
                    max_nfts_per_address: 50,
                    price: coin(50, NATIVE_DENOM),
                    is_public: false,
                },
            };

            // execute add new phase msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_first_phase_msg,
                &[],
            );
            assert!(res.is_ok());

            // prepare execute msg for adding new whitelist to launchpad
            let add_whitelist_msg = ExecuteMsg::AddWhitelist {
                whitelists: [ADMIN.to_string(), USER_1.to_string()].to_vec(),
                phase_id: 1,
            };

            // execute add new whitelist msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_whitelist_msg,
                &[],
            );
            assert!(res.is_ok());

            // admin activate the launchpad
            let activate_msg = ExecuteMsg::ActivateLaunchpad {};
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &activate_msg,
                &[],
            );
            assert!(res.is_ok());

            // change block time increase 400 seconds to make phase active
            app.set_block(BlockInfo {
                time: app.block_info().time.plus_seconds(400),
                height: app.block_info().height + 1,
                chain_id: app.block_info().chain_id,
            });

            // get the price of phase 1
            let phase_config_info: Vec<PhaseConfigResponse> = app
                .wrap()
                .query_wasm_smart(
                    Addr::unchecked(launchpad_address.clone()),
                    &QueryMsg::GetAllPhaseConfigs {},
                )
                .unwrap();

            // Mint 1000000000 native token to USER_1
            app.sudo(cw_multi_test::SudoMsg::Bank(
                cw_multi_test::BankSudo::Mint {
                    to_address: USER_1.to_string(),
                    amount: vec![Coin {
                        amount: 1000000000u128.into(),
                        denom: NATIVE_DENOM.to_string(),
                    }],
                },
            ))
            .unwrap();

            // we need an array to store the minted token ids of nfts
            let mut token_ids: Vec<String> = Vec::new();

            // prepare execute msg for minting nft
            let mint_msg = ExecuteMsg::Mint {
                phase_id: 1,
                amount: Option::from(1),
            };
            // let's USER1 and ADMIN mint 80 nfts
            for i in 0..80 {
                if i % 2 == 0 {
                    // execute mint msg****
                    let res = app.execute_contract(
                        Addr::unchecked(USER_1),
                        Addr::unchecked(launchpad_address.clone()),
                        &mint_msg,
                        &[Coin {
                            denom: NATIVE_DENOM.to_string(),
                            amount: phase_config_info[0].price.amount,
                        }],
                    );
                    assert!(res.is_ok());

                    // get the token id of the minted nft
                    let token_id = &res.unwrap().events[3].attributes[4].value;

                    // check if the token id is unique
                    assert!(!token_ids.contains(token_id));

                    // push the token id to the array
                    token_ids.push(token_id.to_string());
                } else {
                    // execute mint msg
                    let res = app.execute_contract(
                        Addr::unchecked(ADMIN),
                        Addr::unchecked(launchpad_address.clone()),
                        &mint_msg,
                        &[Coin {
                            denom: NATIVE_DENOM.to_string(),
                            amount: phase_config_info[0].price.amount,
                        }],
                    );
                    assert!(res.is_ok());

                    // get the token id of the minted nft
                    let token_id = &res.unwrap().events[3].attributes[4].value;

                    // check if the token id is unique
                    assert!(!token_ids.contains(token_id));

                    // push the token id to the array
                    token_ids.push(token_id.to_string());
                }
            }

            // let's USER1 mint 11 nfts
            // prepare execute msg for minting nft
            let mint_msg = ExecuteMsg::Mint {
                phase_id: 1,
                amount: Option::from(11),
            };
            // execute mint msg
            let res = app.execute_contract(
                Addr::unchecked(USER_1),
                Addr::unchecked(launchpad_address.clone()),
                &mint_msg,
                &[Coin {
                    denom: NATIVE_DENOM.to_string(),
                    amount: (phase_config_info[0]
                        .price
                        .amount
                        .checked_mul(11u128.into())
                        .unwrap()),
                }],
            );
            // the res should return an error because the max amount per mint is 10
            assert_eq!(
                res.unwrap_err().source().unwrap().to_string(),
                "Too many nfts"
            );

            // let's USER1 mint 10 nfts
            // prepare execute msg for minting nft
            let mint_msg = ExecuteMsg::Mint {
                phase_id: 1,
                amount: Option::from(10),
            };
            // execute mint msg
            let res = app.execute_contract(
                Addr::unchecked(USER_1),
                Addr::unchecked(launchpad_address),
                &mint_msg,
                &[Coin {
                    denom: NATIVE_DENOM.to_string(),
                    amount: phase_config_info[0]
                        .price
                        .amount
                        .checked_mul(10u128.into())
                        .unwrap(),
                }],
            );
            assert!(res.is_ok());
        }

        #[test]
        fn any_user_can_mint_nft_of_public_phase() {
            // get integration test app and launchpad address
            let (mut app, launchpad_address) = create_launchpad();

            // ADD FIRST PHASE to the first position
            // prepare execute msg for adding new phase to launchpad
            let add_first_phase_msg = ExecuteMsg::AddMintPhase {
                after_phase_id: None,
                phase_data: PhaseData {
                    start_time: app.block_info().time.plus_seconds(200),
                    end_time: app.block_info().time.plus_seconds(1610),
                    max_supply: Some(100),
                    max_nfts_per_address: 1,
                    price: coin(50, NATIVE_DENOM),
                    is_public: true,
                },
            };

            // execute add new phase msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_first_phase_msg,
                &[],
            );
            assert!(res.is_ok());

            // change block time increase 400 seconds to make phase active
            app.set_block(BlockInfo {
                time: app.block_info().time.plus_seconds(400),
                height: app.block_info().height + 1,
                chain_id: app.block_info().chain_id,
            });

            // get the price of phase 1
            let phase_config_info: Vec<PhaseConfigResponse> = app
                .wrap()
                .query_wasm_smart(
                    Addr::unchecked(launchpad_address.clone()),
                    &QueryMsg::GetAllPhaseConfigs {},
                )
                .unwrap();

            // Mint 1000000000 native token to USER_1
            app.sudo(cw_multi_test::SudoMsg::Bank(
                cw_multi_test::BankSudo::Mint {
                    to_address: USER_1.to_string(),
                    amount: vec![Coin {
                        amount: 1000000000u128.into(),
                        denom: NATIVE_DENOM.to_string(),
                    }],
                },
            ))
            .unwrap();

            // admin active the launchpad
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &ExecuteMsg::ActivateLaunchpad {},
                &[],
            );
            assert!(res.is_ok());

            // prepare execute msg for minting nft
            let mint_msg = ExecuteMsg::Mint {
                phase_id: 1,
                amount: Option::from(1),
            };

            // execute mint msg
            let res = app.execute_contract(
                Addr::unchecked(USER_1),
                Addr::unchecked(launchpad_address.clone()),
                &mint_msg,
                &[Coin {
                    denom: NATIVE_DENOM.to_string(),
                    amount: phase_config_info[0].price.amount,
                }],
            );
            assert!(res.is_ok());

            // ADMIN want to mint nft of phase 1
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address),
                &mint_msg,
                &[Coin {
                    denom: NATIVE_DENOM.to_string(),
                    amount: phase_config_info[0].price.amount,
                }],
            );
            assert!(res.is_ok());
        }

        #[test]
        fn cannot_mint_nft_in_public_phase_because_user_minted_too_much() {
            // get integration test app and launchpad address
            let (mut app, launchpad_address) = create_launchpad();

            // ADD FIRST PHASE to the first position
            // prepare execute msg for adding new phase to launchpad
            let add_first_phase_msg = ExecuteMsg::AddMintPhase {
                after_phase_id: None,
                phase_data: PhaseData {
                    start_time: app.block_info().time.plus_seconds(200),
                    end_time: app.block_info().time.plus_seconds(1610),
                    max_supply: Some(100),
                    max_nfts_per_address: 1,
                    price: coin(50, NATIVE_DENOM),
                    is_public: true,
                },
            };

            // execute add new phase msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_first_phase_msg,
                &[],
            );
            assert!(res.is_ok());

            // admin active the launchpad
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &ExecuteMsg::ActivateLaunchpad {},
                &[],
            );
            assert!(res.is_ok());

            // change block time increase 400 seconds to make phase active
            app.set_block(BlockInfo {
                time: app.block_info().time.plus_seconds(400),
                height: app.block_info().height + 1,
                chain_id: app.block_info().chain_id,
            });

            // get the price of phase 1
            let phase_config_info: Vec<PhaseConfigResponse> = app
                .wrap()
                .query_wasm_smart(
                    Addr::unchecked(launchpad_address.clone()),
                    &QueryMsg::GetAllPhaseConfigs {},
                )
                .unwrap();

            // Mint 1000000000 native token to USER_1
            app.sudo(cw_multi_test::SudoMsg::Bank(
                cw_multi_test::BankSudo::Mint {
                    to_address: USER_1.to_string(),
                    amount: vec![Coin {
                        amount: 1000000000u128.into(),
                        denom: NATIVE_DENOM.to_string(),
                    }],
                },
            ))
            .unwrap();

            // prepare execute msg for minting nft
            let mint_msg = ExecuteMsg::Mint {
                phase_id: 1,
                amount: Option::from(1),
            };

            // execute mint msg
            let res = app.execute_contract(
                Addr::unchecked(USER_1),
                Addr::unchecked(launchpad_address.clone()),
                &mint_msg,
                &[Coin {
                    denom: NATIVE_DENOM.to_string(),
                    amount: phase_config_info[0].price.amount,
                }],
            );
            assert!(res.is_ok());

            // execute mint msg again
            let res = app.execute_contract(
                Addr::unchecked(USER_1),
                Addr::unchecked(launchpad_address),
                &mint_msg,
                &[Coin {
                    denom: NATIVE_DENOM.to_string(),
                    amount: phase_config_info[0].price.amount,
                }],
            );
            assert_eq!(
                res.unwrap_err().source().unwrap().to_string(),
                "User minted too much nfts"
            );
        }

        #[test]
        fn cannot_deactive_launchpad_because_not_admin() {
            // get integration test app and launchpad address
            let (mut app, launchpad_address) = create_launchpad();

            // prepare execute msg for deactivating launchpad
            let deactivate_launchpad_msg = ExecuteMsg::DeactivateLaunchpad {};

            // execute deactivate launchpad msg
            let res = app.execute_contract(
                Addr::unchecked(USER_1),
                Addr::unchecked(launchpad_address),
                &deactivate_launchpad_msg,
                &[],
            );
            assert_eq!(
                res.unwrap_err().source().unwrap().to_string(),
                "Unauthorized"
            );
        }

        #[test]
        fn cannot_active_launchpad_because_not_admin() {
            // get integration test app and launchpad address
            let (mut app, launchpad_address) = create_launchpad();

            // prepare execute msg for deactivating launchpad
            let activate_launchpad_msg = ExecuteMsg::ActivateLaunchpad {};

            // execute deactivate launchpad msg
            let res = app.execute_contract(
                Addr::unchecked(USER_1),
                Addr::unchecked(launchpad_address),
                &activate_launchpad_msg,
                &[],
            );
            assert_eq!(
                res.unwrap_err().source().unwrap().to_string(),
                "Unauthorized"
            );
        }

        #[test]
        fn cannot_deactive_launchpad_because_it_is_already_deactivated() {
            // get integration test app and launchpad address
            let (mut app, launchpad_address) = create_launchpad();

            // prepare execute msg for deactivating launchpad
            let deactivate_launchpad_msg = ExecuteMsg::DeactivateLaunchpad {};

            // execute deactivate launchpad msg again
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address),
                &deactivate_launchpad_msg,
                &[],
            );

            assert_eq!(
                res.unwrap_err().source().unwrap().to_string(),
                "Launchpad is already deactivated"
            );
        }

        #[test]
        fn cannot_active_launchpad_because_it_is_already_activated() {
            // get integration test app and launchpad address
            let (mut app, launchpad_address) = create_launchpad();

            // prepare execute msg for activating launchpad
            let activate_launchpad_msg = ExecuteMsg::ActivateLaunchpad {};

            // execute activate launchpad
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &activate_launchpad_msg,
                &[],
            );
            assert!(res.is_ok());

            // execute activate launchpad msg again
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &activate_launchpad_msg,
                &[],
            );

            assert_eq!(
                res.unwrap_err().source().unwrap().to_string(),
                "Launchpad is already activated"
            );

            // deactive launchpad
            let deactivate_launchpad_msg = ExecuteMsg::DeactivateLaunchpad {};
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address),
                &deactivate_launchpad_msg,
                &[],
            );
            assert!(res.is_ok());
        }

        #[test]
        fn cannot_mint_nft_because_launchpad_is_deactivated() {
            // get integration test app and launchpad address
            let (mut app, launchpad_address) = create_launchpad();

            // ADD FIRST PHASE to the first position
            // prepare execute msg for adding new phase to launchpad
            let add_first_phase_msg = ExecuteMsg::AddMintPhase {
                after_phase_id: None,
                phase_data: PhaseData {
                    start_time: app.block_info().time.plus_seconds(200),
                    end_time: app.block_info().time.plus_seconds(1610),
                    max_supply: Some(100),
                    max_nfts_per_address: 1,
                    price: coin(50, NATIVE_DENOM),
                    is_public: true,
                },
            };

            // execute add new phase msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_first_phase_msg,
                &[],
            );
            assert!(res.is_ok());

            // change block time increase 400 seconds to make phase active
            app.set_block(BlockInfo {
                time: app.block_info().time.plus_seconds(400),
                height: app.block_info().height + 1,
                chain_id: app.block_info().chain_id,
            });

            // get the price of phase 1
            let phase_config_info: Vec<PhaseConfigResponse> = app
                .wrap()
                .query_wasm_smart(
                    Addr::unchecked(launchpad_address.clone()),
                    &QueryMsg::GetAllPhaseConfigs {},
                )
                .unwrap();

            // Mint 1000000000 native token to USER_1
            app.sudo(cw_multi_test::SudoMsg::Bank(
                cw_multi_test::BankSudo::Mint {
                    to_address: USER_1.to_string(),
                    amount: vec![Coin {
                        amount: 1000000000u128.into(),
                        denom: NATIVE_DENOM.to_string(),
                    }],
                },
            ))
            .unwrap();

            // prepare execute msg for minting nft
            let mint_msg = ExecuteMsg::Mint {
                phase_id: 1,
                amount: Option::from(1),
            };

            // execute mint msg
            let res = app.execute_contract(
                Addr::unchecked(USER_1),
                Addr::unchecked(launchpad_address.clone()),
                &mint_msg,
                &[Coin {
                    denom: NATIVE_DENOM.to_string(),
                    amount: phase_config_info[0].price.amount,
                }],
            );

            assert_eq!(
                res.unwrap_err().source().unwrap().to_string(),
                "Launchpad is already deactivated"
            );

            // prepare execute msg for activating launchpad
            let activate_launchpad_msg = ExecuteMsg::ActivateLaunchpad {};

            // execute activate launchpad msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &activate_launchpad_msg,
                &[],
            );
            assert!(res.is_ok());

            // change block time increase 400 seconds to make phase active
            app.set_block(BlockInfo {
                time: app.block_info().time.plus_seconds(400),
                height: app.block_info().height + 1,
                chain_id: app.block_info().chain_id,
            });

            // execute mint msg
            let res = app.execute_contract(
                Addr::unchecked(USER_1),
                Addr::unchecked(launchpad_address),
                &mint_msg,
                &[Coin {
                    denom: NATIVE_DENOM.to_string(),
                    amount: phase_config_info[0].price.amount,
                }],
            );
            assert!(res.is_ok());
        }
    }

    mod claim_token {
        use cosmwasm_std::{coin, BlockInfo, Coin, Uint128};

        use crate::{
            state::{PhaseConfigResponse, PhaseData},
            testing_config::env::{NATIVE_DENOM, USER_1},
        };

        use super::*;

        #[test]
        fn only_creator_can_withdraw_nfts_profit() {
            // get integration test app and launchpad address
            let (mut app, launchpad_address) = create_launchpad();

            // ADD FIRST PHASE to the first position
            // prepare execute msg for adding new phase to launchpad
            let add_first_phase_msg = ExecuteMsg::AddMintPhase {
                after_phase_id: None,
                phase_data: PhaseData {
                    start_time: app.block_info().time.plus_seconds(200),
                    end_time: app.block_info().time.plus_seconds(1610),
                    max_supply: Some(100),
                    max_nfts_per_address: 50,
                    price: coin(100, NATIVE_DENOM),
                    is_public: true,
                },
            };

            // execute add new phase msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_first_phase_msg,
                &[],
            );
            assert!(res.is_ok());

            // add USER_1 to whitelist
            let add_to_whitelist_msg = ExecuteMsg::AddWhitelist {
                phase_id: 1,
                whitelists: vec![USER_1.to_string()],
            };

            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &add_to_whitelist_msg,
                &[],
            );
            assert!(res.is_ok());

            // admin activate launchpad
            let activate_launchpad_msg = ExecuteMsg::ActivateLaunchpad {};
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &activate_launchpad_msg,
                &[],
            );
            assert!(res.is_ok());

            // change block time increase 400 seconds to make phase active
            app.set_block(BlockInfo {
                time: app.block_info().time.plus_seconds(400),
                height: app.block_info().height + 1,
                chain_id: app.block_info().chain_id,
            });

            // get the price of phase 1
            let phase_config_info: Vec<PhaseConfigResponse> = app
                .wrap()
                .query_wasm_smart(
                    Addr::unchecked(launchpad_address.clone()),
                    &QueryMsg::GetAllPhaseConfigs {},
                )
                .unwrap();

            // Mint 1000000000 native token to USER_1
            app.sudo(cw_multi_test::SudoMsg::Bank(
                cw_multi_test::BankSudo::Mint {
                    to_address: USER_1.to_string(),
                    amount: vec![Coin {
                        amount: 1000000000u128.into(),
                        denom: NATIVE_DENOM.to_string(),
                    }],
                },
            ))
            .unwrap();

            // prepare execute msg for minting nft
            let mint_msg = ExecuteMsg::Mint {
                phase_id: 1,
                amount: Option::from(10),
            };

            // execute mint msg
            let res = app.execute_contract(
                Addr::unchecked(USER_1),
                Addr::unchecked(launchpad_address.clone()),
                &mint_msg,
                &[Coin {
                    denom: NATIVE_DENOM.to_string(),
                    amount: phase_config_info[0]
                        .price
                        .amount
                        .checked_mul(10u128.into())
                        .unwrap(),
                }],
            );
            assert!(res.is_ok());

            // creator withdraw nft profit
            let withdraw_nft_profit_msg = ExecuteMsg::Withdraw {
                denom: NATIVE_DENOM.to_string(),
            };

            // ADMIN execute withdraw nft profit msg
            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(launchpad_address.clone()),
                &withdraw_nft_profit_msg,
                &[],
            );
            assert_eq!(
                res.unwrap_err().source().unwrap().to_string(),
                "Unauthorized"
            );

            // change block time increase 1209 seconds to make phase almost ended
            app.set_block(BlockInfo {
                time: app.block_info().time.plus_seconds(1209),
                height: app.block_info().height + 1,
                chain_id: app.block_info().chain_id,
            });

            // CREATOR cannot execute withdraw nft profit msg because the last phase is not ended
            let res = app.execute_contract(
                Addr::unchecked(CREATOR),
                Addr::unchecked(launchpad_address.clone()),
                &withdraw_nft_profit_msg,
                &[],
            );
            assert_eq!(
                res.unwrap_err().source().unwrap().to_string(),
                "Last phase not finished"
            );

            // change block time increase 1 seconds to make phase ended
            app.set_block(BlockInfo {
                time: app.block_info().time.plus_seconds(1),
                height: app.block_info().height + 1,
                chain_id: app.block_info().chain_id,
            });

            // execute withdraw nft profit msg
            let res = app.execute_contract(
                Addr::unchecked(CREATOR),
                Addr::unchecked(launchpad_address),
                &withdraw_nft_profit_msg,
                &[],
            );
            assert!(res.is_ok());

            // the CREATOR should have (100 * 10) * 90%  native token
            let creator_balance = app
                .wrap()
                .query_balance(Addr::unchecked(CREATOR), NATIVE_DENOM)
                .unwrap();
            assert_eq!(creator_balance.amount, Uint128::from(900u128));

            // the LAUNCHPAD_COLLECTOR should have (100 * 10) * 10%  native token
            let launchpad_balance = app
                .wrap()
                .query_balance(Addr::unchecked(LAUNCHPAD_COLLECTOR), NATIVE_DENOM)
                .unwrap();
            assert_eq!(launchpad_balance.amount, Uint128::from(100u128));
        }
    }
}
