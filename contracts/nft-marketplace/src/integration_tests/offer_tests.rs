use crate::msg::{ExecuteMsg, OffersResponse, QueryMsg};
use crate::order_state::NFT;

use crate::test_setup::env::{instantiate_contracts, NATIVE_DENOM, OWNER, USER_1, USER_2};

use anyhow::Result as AnyResult;

use cosmwasm_std::{Addr, Coin, Uint128};
use cw_multi_test::{App, AppResponse, Executor};

use cw2981_royalties::{Metadata, MintMsg, QueryMsg as Cw721QueryMsg};
use cw721::Expiration as Cw721Expiration;
use cw721_base::msg::ExecuteMsg as Cw721ExecuteMsg;

const MOCK_OFFER_NFT_TOKEN_ID_1: &str = "token1";
const MOCK_OFFER_NFT_TOKEN_ID_2: &str = "token2";

const MOCK_OFFER_CW20_PRICE: u128 = 10000000;

fn mint_nft(app: &mut App, token_id: &str, owner: &str, cw2981_address: String) {
    let mint_msg: Cw721ExecuteMsg<Metadata, Metadata> = Cw721ExecuteMsg::Mint(MintMsg {
        token_id: token_id.to_string(),
        owner: owner.to_string(),
        token_uri: Some(
            "https://ipfs.io/ipfs/Qme7ss3ARVgxv6rXqVPiikMJ8u2NLgmgszg13pYrDKEoiu".to_string(),
        ),
        extension: Metadata::default(),
    });

    (*app)
        .execute_contract(
            Addr::unchecked(OWNER.to_string()),
            Addr::unchecked(cw2981_address),
            &mint_msg,
            &[],
        )
        .unwrap();
}

fn create_offer(
    app: &mut App,
    token_id: &str,
    owner: &str,
    cw2981_address: String,
    marketplace_address: String,
) -> AnyResult<AppResponse> {
    // offerer creates offer
    // prepare offer nft message
    let offer_nft_msg = ExecuteMsg::OfferNft {
        nft: NFT {
            contract_address: Addr::unchecked(cw2981_address),
            token_id: Some(token_id.to_string()),
        },
        funds_amount: MOCK_OFFER_CW20_PRICE,
        end_time: Cw721Expiration::AtTime(app.block_info().time.plus_seconds(1000)),
    };

    // offerer (USER_1) creates offer
    (*app).execute_contract(
        Addr::unchecked(owner.to_string()),
        Addr::unchecked(marketplace_address),
        &offer_nft_msg,
        &[],
    )
}

mod accept_offer {
    use super::*;

    use crate::state::{AuctionConfig, Listing};
    use cosmwasm_std::StdResult;
    use cw20::{BalanceResponse, Cw20QueryMsg};

    #[test]
    fn nft_owner_can_accept_offer() {
        // get integration test app and contracts
        let (mut app, contracts) = instantiate_contracts(false);
        let cw2981_address = contracts[0].contract_addr.clone();
        let marketplace_address = contracts[1].contract_addr.clone();
        let cw20_address = contracts[2].contract_addr.clone();

        // prepare mint cw2981 message to OWNER
        mint_nft(
            &mut app,
            MOCK_OFFER_NFT_TOKEN_ID_1,
            OWNER,
            cw2981_address.clone(),
        );

        mint_nft(
            &mut app,
            MOCK_OFFER_NFT_TOKEN_ID_2,
            USER_2,
            cw2981_address.clone(),
        );

        // execute mint function to convert native token to twilight token
        let _response = app
            .execute_contract(
                Addr::unchecked(USER_1.to_string()),
                Addr::unchecked(&cw20_address),
                &cw20::Cw20ExecuteMsg::Mint {
                    recipient: USER_1.to_string(),
                    amount: Uint128::from(100000000u128),
                },
                &[Coin {
                    amount: Uint128::from(100000000u128),
                    denom: NATIVE_DENOM.to_string(),
                }],
            )
            .unwrap();

        let res = create_offer(
            &mut app,
            MOCK_OFFER_NFT_TOKEN_ID_1,
            USER_1,
            cw2981_address.clone(),
            marketplace_address.clone(),
        );
        assert!(res.is_ok());

        let res = create_offer(
            &mut app,
            MOCK_OFFER_NFT_TOKEN_ID_1,
            USER_2,
            cw2981_address.clone(),
            marketplace_address.clone(),
        );
        assert!(res.is_ok());

        let res = create_offer(
            &mut app,
            MOCK_OFFER_NFT_TOKEN_ID_2,
            USER_1,
            cw2981_address.clone(),
            marketplace_address.clone(),
        );
        assert!(res.is_ok());

        // check the owner of the token
        let res: cw721::OwnerOfResponse = app
            .wrap()
            .query_wasm_smart(
                Addr::unchecked(cw2981_address.clone()),
                &Cw721QueryMsg::OwnerOf {
                    token_id: MOCK_OFFER_NFT_TOKEN_ID_1.to_string(),
                    include_expired: None,
                },
            )
            .unwrap();
        // it should be OWNER
        assert_eq!(res.owner, OWNER.to_string(), "Invalid owner");

        // get offers for nft
        let res: OffersResponse = app
            .wrap()
            .query_wasm_smart(
                Addr::unchecked(&marketplace_address),
                &QueryMsg::NftOffers {
                    contract_address: cw2981_address.clone(),
                    token_id: MOCK_OFFER_NFT_TOKEN_ID_1.to_string(),
                    start_after_offerer: None,
                    limit: None,
                },
            )
            .unwrap();

        assert!(res.offers.len() == 2);

        // get offers of USER_1
        let res: OffersResponse = app
            .wrap()
            .query_wasm_smart(
                Addr::unchecked(&marketplace_address),
                &QueryMsg::UserOffers {
                    offerer: USER_1.to_string(),
                    start_after_nft: None,
                    limit: None,
                },
            )
            .unwrap();
        assert!(res.offers.len() == 2);

        // OWNER approve NFT to marketplace
        let approve_msg = cw721::Cw721ExecuteMsg::ApproveAll {
            operator: marketplace_address.clone(),
            expires: None,
        };
        app.execute_contract(
            Addr::unchecked(OWNER),
            Addr::unchecked(&cw2981_address),
            &approve_msg,
            &[],
        )
        .unwrap();

        // OWNER accepts offer of USER_1
        let accept_offer_msg = ExecuteMsg::AcceptNftOffer {
            offerer: USER_1.to_string(),
            nft: NFT {
                contract_address: Addr::unchecked(&cw2981_address),
                token_id: Some(MOCK_OFFER_NFT_TOKEN_ID_1.to_string()),
            },
            funds_amount: MOCK_OFFER_CW20_PRICE,
        };

        // owner (OWNER) accepts offer
        let res = app.execute_contract(
            Addr::unchecked(OWNER),
            Addr::unchecked(&marketplace_address),
            &accept_offer_msg,
            &[],
        );
        println!("res: {:?}", res);
        assert!(res.is_ok());

        // assert NFT is transfered to USER_1
        let res: cw721::OwnerOfResponse = app
            .wrap()
            .query_wasm_smart(
                Addr::unchecked(cw2981_address),
                &Cw721QueryMsg::OwnerOf {
                    token_id: MOCK_OFFER_NFT_TOKEN_ID_1.to_string(),
                    include_expired: None,
                },
            )
            .unwrap();
        assert_eq!(res.owner, USER_1.to_string(), "Invalid owner");

        // assert token is transfered to OWNER
        let res: cw20::BalanceResponse = app
            .wrap()
            .query_wasm_smart(
                Addr::unchecked(cw20_address),
                &cw20::Cw20QueryMsg::Balance {
                    address: OWNER.to_string(),
                },
            )
            .unwrap();
        assert_eq!(
            res.balance,
            Uint128::from(MOCK_OFFER_CW20_PRICE),
            "Invalid balance"
        );
    }

    #[test]
    fn royalty_with_offer() {
        // get integration test app and contracts
        let (mut app, contracts) = instantiate_contracts(true);
        let cw2981_address = contracts[0].contract_addr.clone();
        let marketplace_address = contracts[1].contract_addr.clone();
        let cw20_address = contracts[2].contract_addr.clone();

        // prepare mint cw2981 message to OWNER
        mint_nft(
            &mut app,
            MOCK_OFFER_NFT_TOKEN_ID_1,
            OWNER,
            cw2981_address.clone(),
        );

        // transfer NFT to USER_1
        let transfer_msg = cw721::Cw721ExecuteMsg::TransferNft {
            recipient: USER_1.to_string(),
            token_id: MOCK_OFFER_NFT_TOKEN_ID_1.to_string(),
        };
        app.execute_contract(
            Addr::unchecked(OWNER),
            Addr::unchecked(&cw2981_address),
            &transfer_msg,
            &[],
        )
        .unwrap();

        // execute mint function to convert native token to twilight token
        let _response = app
            .execute_contract(
                Addr::unchecked(USER_2.to_string()),
                Addr::unchecked(&cw20_address),
                &cw20::Cw20ExecuteMsg::Mint {
                    recipient: USER_2.to_string(),
                    amount: Uint128::from(100000000u128),
                },
                &[Coin {
                    amount: Uint128::from(100000000u128),
                    denom: NATIVE_DENOM.to_string(),
                }],
            )
            .unwrap();

        // USER_2 offers Nft
        let res = create_offer(
            &mut app,
            MOCK_OFFER_NFT_TOKEN_ID_1,
            USER_2,
            cw2981_address.clone(),
            marketplace_address.clone(),
        );
        assert!(res.is_ok());

        // USER_1 approve NFT to marketplace
        let approve_msg = cw721::Cw721ExecuteMsg::ApproveAll {
            operator: marketplace_address.clone(),
            expires: None,
        };
        app.execute_contract(
            Addr::unchecked(USER_1),
            Addr::unchecked(&cw2981_address),
            &approve_msg,
            &[],
        )
        .unwrap();

        // USER_1 accepts offer of USER_2
        let accept_offer_msg = ExecuteMsg::AcceptNftOffer {
            offerer: USER_2.to_string(),
            nft: NFT {
                contract_address: Addr::unchecked(&cw2981_address),
                token_id: Some(MOCK_OFFER_NFT_TOKEN_ID_1.to_string()),
            },
            funds_amount: MOCK_OFFER_CW20_PRICE,
        };
        let res = app.execute_contract(
            Addr::unchecked(USER_1),
            Addr::unchecked(&marketplace_address),
            &accept_offer_msg,
            &[],
        );
        println!("res: {:?}", res);
        assert!(res.is_ok());

        // assert NFT is transfered to USER_2
        let res: cw721::OwnerOfResponse = app
            .wrap()
            .query_wasm_smart(
                Addr::unchecked(cw2981_address),
                &Cw721QueryMsg::OwnerOf {
                    token_id: MOCK_OFFER_NFT_TOKEN_ID_1.to_string(),
                    include_expired: None,
                },
            )
            .unwrap();
        assert_eq!(res.owner, USER_2.to_string(), "Invalid owner");

        // assert token is transfered to USER_1
        let res: cw20::BalanceResponse = app
            .wrap()
            .query_wasm_smart(
                Addr::unchecked(cw20_address.clone()),
                &cw20::Cw20QueryMsg::Balance {
                    address: USER_1.to_string(),
                },
            )
            .unwrap();
        assert_eq!(
            res.balance,
            Uint128::from(MOCK_OFFER_CW20_PRICE).multiply_ratio(80u128, 100u128),
            "Token is not transfered to seller"
        );

        // assert royalty is transfered to OWNER
        let res: cw20::BalanceResponse = app
            .wrap()
            .query_wasm_smart(
                Addr::unchecked(cw20_address),
                &cw20::Cw20QueryMsg::Balance {
                    address: OWNER.to_string(),
                },
            )
            .unwrap();
        assert_eq!(
            res.balance,
            Uint128::from(MOCK_OFFER_CW20_PRICE).multiply_ratio(20u128, 100u128),
            "Royalty is not transfered to owner"
        );
    }

    #[test]
    fn remove_listing_after_accept_offer() {
        // get integration test app and contracts
        let (mut app, contracts) = instantiate_contracts(true);
        let cw2981_address = contracts[0].contract_addr.clone();
        let marketplace_address = contracts[1].contract_addr.clone();
        let cw20_address = contracts[2].contract_addr.clone();

        // prepare mint cw2981 message to OWNER
        let mint_msg: Cw721ExecuteMsg<Metadata, Metadata> = Cw721ExecuteMsg::Mint(MintMsg {
            token_id: MOCK_OFFER_NFT_TOKEN_ID_1.to_string(),
            owner: OWNER.to_string(),
            token_uri: Some(
                "https://ipfs.io/ipfs/Qme7ss3ARVgxv6rXqVPiikMJ8u2NLgmgszg13pYrDKEoiu".to_string(),
            ),
            extension: Metadata {
                image: None,
                image_data: None,
                external_url: None,
                description: None,
                name: None,
                attributes: None,
                background_color: None,
                animation_url: None,
                youtube_url: None,
                royalty_percentage: None,
                royalty_payment_address: None,
            },
        });

        // mint cw2981 token to OWNER
        let res = app.execute_contract(
            Addr::unchecked(OWNER),
            Addr::unchecked(cw2981_address.clone()),
            &mint_msg,
            &[],
        );
        assert!(res.is_ok());

        // approve marketplace to transfer nft token
        let approve_msg: Cw721ExecuteMsg<Metadata, Metadata> = Cw721ExecuteMsg::Approve {
            spender: marketplace_address.clone(),
            token_id: MOCK_OFFER_NFT_TOKEN_ID_1.to_string(),
            expires: None,
        };

        // OWNER approves marketplace to transfer nft token
        let res = app.execute_contract(
            Addr::unchecked(OWNER),
            Addr::unchecked(cw2981_address.clone()),
            &approve_msg,
            &[],
        );
        assert!(res.is_ok());

        // OWNER list the token
        let list_nft_msg = ExecuteMsg::ListNft {
            contract_address: cw2981_address.clone(),
            token_id: MOCK_OFFER_NFT_TOKEN_ID_1.to_string(),
            auction_config: AuctionConfig::FixedPrice {
                price: Coin {
                    denom: "uaura".to_string(),
                    amount: Uint128::from(200u128),
                },
                start_time: None,
                end_time: None,
            },
        };

        // OWNER list the token
        let res = app.execute_contract(
            Addr::unchecked(OWNER),
            Addr::unchecked(marketplace_address.clone()),
            &list_nft_msg,
            &[],
        );

        assert!(res.is_ok());

        // query the listing
        let res: Listing = app
            .wrap()
            .query_wasm_smart(
                Addr::unchecked(marketplace_address.clone()),
                &QueryMsg::Listing {
                    contract_address: cw2981_address.clone(),
                    token_id: MOCK_OFFER_NFT_TOKEN_ID_1.to_string(),
                },
            )
            .unwrap();

        assert_eq!(res.token_id, MOCK_OFFER_NFT_TOKEN_ID_1.to_string());
        assert_eq!(res.contract_address, cw2981_address);

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

        // execute mint function to convert native token to twilight token
        let response = app.execute_contract(
            Addr::unchecked(USER_1.to_string()),
            Addr::unchecked(cw20_address.clone()),
            &cw20::Cw20ExecuteMsg::Mint {
                recipient: USER_1.to_string(),
                amount: Uint128::from(100000000u128),
            },
            &[Coin {
                amount: Uint128::from(100000000u128),
                denom: NATIVE_DENOM.to_string(),
            }],
        );
        assert!(response.is_ok());

        // offerer creates offer
        // prepare offer nft message
        let offer_nft_msg = ExecuteMsg::OfferNft {
            nft: NFT {
                contract_address: Addr::unchecked(cw2981_address.clone()),
                token_id: Some(MOCK_OFFER_NFT_TOKEN_ID_1.to_string()),
            },
            funds_amount: MOCK_OFFER_CW20_PRICE,
            end_time: Cw721Expiration::AtTime(app.block_info().time.plus_seconds(1000)),
        };

        // offerer (USER_1) creates offer
        let res = app.execute_contract(
            Addr::unchecked(USER_1.to_string()),
            Addr::unchecked(marketplace_address.clone()),
            &offer_nft_msg,
            &[],
        );
        assert!(res.is_ok());

        // check the owner of the token
        let res: cw721::OwnerOfResponse = app
            .wrap()
            .query_wasm_smart(
                Addr::unchecked(cw2981_address.clone()),
                &Cw721QueryMsg::OwnerOf {
                    token_id: MOCK_OFFER_NFT_TOKEN_ID_1.to_string(),
                    include_expired: None,
                },
            )
            .unwrap();
        // it should be OWNER
        assert_eq!(res.owner, OWNER.to_string());

        // *******************
        // OWNER ACCEPTS OFFER
        // *******************
        // OWNER must approve marketplace to transfer nft token
        // prepare approve message
        let approve_msg: Cw721ExecuteMsg<Metadata, Metadata> = Cw721ExecuteMsg::Approve {
            spender: marketplace_address.clone(),
            token_id: MOCK_OFFER_NFT_TOKEN_ID_1.to_string(),
            expires: None,
        };

        // OWNER approves marketplace to transfer nft token
        let res = app.execute_contract(
            Addr::unchecked(OWNER),
            Addr::unchecked(cw2981_address.clone()),
            &approve_msg,
            &[],
        );
        assert!(res.is_ok());

        // prepare accept offer message
        let accept_offer_msg = ExecuteMsg::AcceptNftOffer {
            offerer: USER_1.to_string(),
            nft: NFT {
                contract_address: Addr::unchecked(cw2981_address.clone()),
                token_id: Some(MOCK_OFFER_NFT_TOKEN_ID_1.to_string()),
            },
            funds_amount: MOCK_OFFER_CW20_PRICE,
        };

        // owner (OWNER) accepts offer
        let res = app.execute_contract(
            Addr::unchecked(OWNER),
            Addr::unchecked(marketplace_address.clone()),
            &accept_offer_msg,
            &[],
        );
        assert!(res.is_ok());

        // query the listing
        let res: StdResult<Listing> = app.wrap().query_wasm_smart(
            Addr::unchecked(marketplace_address),
            &QueryMsg::Listing {
                contract_address: cw2981_address.clone(),
                token_id: MOCK_OFFER_NFT_TOKEN_ID_1.to_string(),
            },
        );

        assert_eq!(
            res.unwrap_err().to_string(),
            "Generic error: Querier contract error: nft_marketplace::state::Listing not found"
        );

        // check the owner of the token
        let res: cw721::OwnerOfResponse = app
            .wrap()
            .query_wasm_smart(
                Addr::unchecked(cw2981_address),
                &Cw721QueryMsg::OwnerOf {
                    token_id: MOCK_OFFER_NFT_TOKEN_ID_1.to_string(),
                    include_expired: None,
                },
            )
            .unwrap();
        assert_eq!(res.owner, USER_1.to_string());

        // check the balance cw20 of the offerer
        let res: BalanceResponse = app
            .wrap()
            .query_wasm_smart(
                Addr::unchecked(cw20_address),
                &Cw20QueryMsg::Balance {
                    address: OWNER.to_string(),
                },
            )
            .unwrap();

        assert_eq!(res.balance, Uint128::from(MOCK_OFFER_CW20_PRICE));
    }
}
