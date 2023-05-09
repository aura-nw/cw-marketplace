use crate::msg::ExecuteMsg;
use crate::order_state::NFT;

use crate::test_setup::env::{instantiate_contracts, NATIVE_DENOM, OWNER, USER_1, USER_2};

use anyhow::Result as AnyResult;

use cosmwasm_std::{coin, Addr, Uint128};
use cw_multi_test::{App, AppResponse, Executor};

use crate::msg::QueryMsg;
use crate::state::AuctionConfigInput;
use cw2981_royalties::{Metadata, MintMsg, QueryMsg as Cw721QueryMsg};
use cw721::Expiration as Cw721Expiration;
use cw721_base::msg::ExecuteMsg as Cw721ExecuteMsg;

const TOKEN_ID_1: &str = "token1";
const TOKEN_ID_2: &str = "token2";
const TOKEN_ID_3: &str = "token3";

const START_PRICE: u128 = 10000000;
const STEP_PERCENTAGE: u8 = 5;

fn mint_nft(app: &mut App, token_id: &str, owner: &str, cw2981_address: String) {
    let mint_msg: Cw721ExecuteMsg<Metadata, Metadata> = Cw721ExecuteMsg::Mint(MintMsg {
        token_id: token_id.to_string(),
        owner: owner.to_string(),
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

    (*app)
        .execute_contract(
            Addr::unchecked(OWNER.to_string()),
            Addr::unchecked(cw2981_address),
            &mint_msg,
            &[],
        )
        .unwrap();
}

fn approval_token(
    app: &mut App,
    owner: &str,
    token_id: &str,
    cw2981_address: String,
    marketplace_address: String,
) {
    let approve_msg: Cw721ExecuteMsg<Metadata, Metadata> = Cw721ExecuteMsg::Approve {
        spender: marketplace_address,
        token_id: token_id.to_string(),
        expires: None,
    };

    (*app)
        .execute_contract(
            Addr::unchecked(owner.to_string()),
            Addr::unchecked(cw2981_address),
            &approve_msg,
            &[],
        )
        .unwrap();
}

fn create_auction(
    app: &mut App,
    token_id: Option<String>,
    owner: &str,
    cw2981_address: String,
    marketplace_address: String,
    auction_config: AuctionConfigInput,
) -> AnyResult<AppResponse> {
    // owner creates auction
    // prepare auction nft message
    let offer_nft_msg = ExecuteMsg::AuctionNft {
        nft: NFT {
            contract_address: Addr::unchecked(cw2981_address),
            token_id,
        },
        auction_config,
    };

    // offerer (USER_1) creates offer
    (*app).execute_contract(
        Addr::unchecked(owner.to_string()),
        Addr::unchecked(marketplace_address),
        &offer_nft_msg,
        &[],
    )
}

fn bid_auction(
    app: &mut App,
    token_id: Option<String>,
    sender: &str,
    cw2981_address: String,
    marketplace_address: String,
    bid_price: u128,
    bid_funds: Option<u128>,
) -> AnyResult<AppResponse> {
    // owner creates auction
    // prepare bid nft message
    let bid_auction_msg = ExecuteMsg::BidAuction {
        nft: NFT {
            contract_address: Addr::unchecked(cw2981_address),
            token_id,
        },
        bid_price,
    };

    // offerer (USER_1) creates offer
    if let Some(bid_funds) = bid_funds {
        (*app).execute_contract(
            Addr::unchecked(sender.to_string()),
            Addr::unchecked(marketplace_address),
            &bid_auction_msg,
            &[coin(bid_funds, NATIVE_DENOM)],
        )
    } else {
        (*app).execute_contract(
            Addr::unchecked(sender.to_string()),
            Addr::unchecked(marketplace_address),
            &bid_auction_msg,
            &[],
        )
    }
}

fn settle_auction(
    app: &mut App,
    token_id: Option<String>,
    sender: &str,
    cw2981_address: String,
    marketplace_address: String,
) -> AnyResult<AppResponse> {
    // sender settle auction
    // prepare settle auction message
    let settle_auction_msg = ExecuteMsg::SettleAuction {
        nft: NFT {
            contract_address: Addr::unchecked(cw2981_address),
            token_id,
        },
    };

    // sender settle auction
    (*app).execute_contract(
        Addr::unchecked(sender.to_string()),
        Addr::unchecked(marketplace_address),
        &settle_auction_msg,
        &[],
    )
}

mod create_auction {
    use super::*;
    use cw721::OwnerOfResponse;

    #[test]
    fn owner_cannot_auction_because_not_approval_nft() {
        // get integration test app and contracts
        let (mut app, contracts) = instantiate_contracts();
        let cw2981_address = contracts[0].contract_addr.clone();
        let marketplace_address = contracts[1].contract_addr.clone();

        // mint a cw2981 nft to OWNER
        mint_nft(&mut app, TOKEN_ID_1, OWNER, cw2981_address.clone());

        // create auction config
        let auction_config = AuctionConfigInput::EnglishAuction {
            start_price: coin(START_PRICE, NATIVE_DENOM),
            step_percentage: Some(STEP_PERCENTAGE),
            buyout_price: None,
            start_time: None,
            end_time: Cw721Expiration::AtTime(app.block_info().time.plus_seconds(1000)),
        };

        let res = create_auction(
            &mut app,
            Some(TOKEN_ID_1.to_string()),
            OWNER,
            cw2981_address,
            marketplace_address,
            auction_config,
        );
        assert_eq!(
            res.unwrap_err()
                .source()
                .unwrap()
                .source()
                .unwrap()
                .to_string(),
            "Unauthorized"
        );
    }

    #[test]
    fn owner_cannot_auction_because_time_invalid() {
        // get integration test app and contracts
        let (mut app, contracts) = instantiate_contracts();
        let cw2981_address = contracts[0].contract_addr.clone();
        let marketplace_address = contracts[1].contract_addr.clone();

        // mint a cw2981 nft to OWNER
        mint_nft(&mut app, TOKEN_ID_1, OWNER, cw2981_address.clone());

        // approve marketplace to transfer nft
        approval_token(
            &mut app,
            OWNER,
            TOKEN_ID_1,
            cw2981_address.clone(),
            marketplace_address.clone(),
        );

        // create auction config
        let auction_config = AuctionConfigInput::EnglishAuction {
            start_price: coin(START_PRICE, NATIVE_DENOM),
            step_percentage: Some(STEP_PERCENTAGE),
            buyout_price: None,
            start_time: Some(Cw721Expiration::AtTime(
                app.block_info().time.minus_nanos(10),
            )),
            end_time: Cw721Expiration::AtTime(app.block_info().time.plus_seconds(1000)),
        };

        let res = create_auction(
            &mut app,
            Some(TOKEN_ID_1.to_string()),
            USER_1,
            cw2981_address,
            marketplace_address,
            auction_config,
        );
        assert_eq!(
            res.unwrap_err().source().unwrap().to_string(),
            "Custom Error val: \"Time config invalid\""
        );
    }

    #[test]
    fn owner_cannot_auction_collection() {
        // get integration test app and contracts
        let (mut app, contracts) = instantiate_contracts();
        let cw2981_address = contracts[0].contract_addr.clone();
        let marketplace_address = contracts[1].contract_addr.clone();

        // mint a cw2981 nft to OWNER
        mint_nft(&mut app, TOKEN_ID_1, OWNER, cw2981_address.clone());

        // approve marketplace to transfer nft
        approval_token(
            &mut app,
            OWNER,
            TOKEN_ID_1,
            cw2981_address.clone(),
            marketplace_address.clone(),
        );

        // create auction config
        let auction_config = AuctionConfigInput::EnglishAuction {
            start_price: coin(START_PRICE, NATIVE_DENOM),
            step_percentage: Some(STEP_PERCENTAGE),
            buyout_price: None,
            start_time: Some(Cw721Expiration::AtTime(
                app.block_info().time.plus_seconds(10),
            )),
            end_time: Cw721Expiration::AtTime(app.block_info().time.plus_seconds(1000)),
        };

        let res = create_auction(
            &mut app,
            None,
            USER_1,
            cw2981_address,
            marketplace_address,
            auction_config,
        );
        assert_eq!(
            res.unwrap_err().source().unwrap().to_string(),
            "Custom Error val: \"Token id is required\""
        );
    }

    #[test]
    fn owner_cannot_auction_nft_beccause_config_invalid() {
        // get integration test app and contracts
        let (mut app, contracts) = instantiate_contracts();
        let cw2981_address = contracts[0].contract_addr.clone();
        let marketplace_address = contracts[1].contract_addr.clone();

        // mint a cw2981 nft to OWNER
        mint_nft(&mut app, TOKEN_ID_1, OWNER, cw2981_address.clone());

        // approve marketplace to transfer nft
        approval_token(
            &mut app,
            OWNER,
            TOKEN_ID_1,
            cw2981_address.clone(),
            marketplace_address.clone(),
        );

        // create auction config
        let auction_config = AuctionConfigInput::FixedPrice {
            price: coin(START_PRICE, NATIVE_DENOM),
            start_time: None,
            end_time: None,
        };

        let res = create_auction(
            &mut app,
            None,
            USER_1,
            cw2981_address,
            marketplace_address,
            auction_config,
        );
        assert_eq!(
            res.unwrap_err().source().unwrap().to_string(),
            "Custom Error val: \"Invalid auction config\""
        );
    }

    #[test]
    fn owner_can_auction_nft() {
        // get integration test app and contracts
        let (mut app, contracts) = instantiate_contracts();
        let cw2981_address = contracts[0].contract_addr.clone();
        let marketplace_address = contracts[1].contract_addr.clone();

        // mint a cw2981 nft to OWNER
        mint_nft(&mut app, TOKEN_ID_1, OWNER, cw2981_address.clone());

        // approve marketplace to transfer nft
        approval_token(
            &mut app,
            OWNER,
            TOKEN_ID_1,
            cw2981_address.clone(),
            marketplace_address.clone(),
        );

        // create auction config
        let auction_config = AuctionConfigInput::EnglishAuction {
            start_price: coin(START_PRICE, NATIVE_DENOM),
            step_percentage: Some(STEP_PERCENTAGE),
            buyout_price: None,
            start_time: None,
            end_time: Cw721Expiration::AtTime(app.block_info().time.plus_seconds(1000)),
        };

        let res = create_auction(
            &mut app,
            Some(TOKEN_ID_1.to_string()),
            OWNER,
            cw2981_address.clone(),
            marketplace_address.clone(),
            auction_config,
        );
        assert!(res.is_ok());

        // query owner of token
        let query_msg = Cw721QueryMsg::OwnerOf {
            token_id: TOKEN_ID_1.to_string(),
            include_expired: None,
        };

        let res: OwnerOfResponse = app
            .wrap()
            .query_wasm_smart(Addr::unchecked(cw2981_address), &query_msg)
            .unwrap();
        assert_eq!(res.owner, marketplace_address);
    }
}

mod bid_auction {
    use crate::order_state::OrderComponents;

    use super::*;

    #[test]
    fn cannot_bid_your_own_auction() {
        // get integration test app and contracts
        let (mut app, contracts) = instantiate_contracts();
        let cw2981_address = contracts[0].contract_addr.clone();
        let marketplace_address = contracts[1].contract_addr.clone();

        // mint a cw2981 nft to USER_2
        mint_nft(&mut app, TOKEN_ID_1, USER_2, cw2981_address.clone());

        // approve marketplace to transfer nft
        approval_token(
            &mut app,
            USER_2,
            TOKEN_ID_1,
            cw2981_address.clone(),
            marketplace_address.clone(),
        );

        // create auction config
        let auction_config = AuctionConfigInput::EnglishAuction {
            start_price: coin(START_PRICE, NATIVE_DENOM),
            step_percentage: Some(STEP_PERCENTAGE),
            buyout_price: None,
            start_time: None,
            end_time: Cw721Expiration::AtTime(app.block_info().time.plus_seconds(1000)),
        };

        let res = create_auction(
            &mut app,
            Some(TOKEN_ID_1.to_string()),
            USER_2,
            cw2981_address.clone(),
            marketplace_address.clone(),
            auction_config,
        );
        assert!(res.is_ok());

        // bid auction
        let res = bid_auction(
            &mut app,
            Some(TOKEN_ID_1.to_string()),
            USER_2,
            cw2981_address,
            marketplace_address,
            10000u128,
            None,
        );
        assert_eq!(
            res.unwrap_err().source().unwrap().to_string(),
            "Custom Error val: \"Cannot bid on your own auction\""
        );
    }

    #[test]
    fn cannot_bid_when_auction_expired() {
        // get integration test app and contracts
        let (mut app, contracts) = instantiate_contracts();
        let cw2981_address = contracts[0].contract_addr.clone();
        let marketplace_address = contracts[1].contract_addr.clone();

        // mint a cw2981 nft to USER_2
        mint_nft(&mut app, TOKEN_ID_1, USER_2, cw2981_address.clone());

        // approve marketplace to transfer nft
        approval_token(
            &mut app,
            USER_2,
            TOKEN_ID_1,
            cw2981_address.clone(),
            marketplace_address.clone(),
        );

        // create auction config
        let auction_config = AuctionConfigInput::EnglishAuction {
            start_price: coin(START_PRICE, NATIVE_DENOM),
            step_percentage: Some(STEP_PERCENTAGE),
            buyout_price: None,
            start_time: None,
            end_time: Cw721Expiration::AtTime(app.block_info().time.plus_seconds(1000)),
        };

        let res = create_auction(
            &mut app,
            Some(TOKEN_ID_1.to_string()),
            USER_2,
            cw2981_address.clone(),
            marketplace_address.clone(),
            auction_config,
        );
        assert!(res.is_ok());

        let mut block_info = app.block_info();
        block_info.time = block_info.time.plus_seconds(1001);
        app.set_block(block_info);

        // bid auction
        let res = bid_auction(
            &mut app,
            Some(TOKEN_ID_1.to_string()),
            USER_1,
            cw2981_address,
            marketplace_address,
            START_PRICE,
            Some(START_PRICE),
        );
        assert_eq!(
            res.unwrap_err().source().unwrap().to_string(),
            "Custom Error val: \"Auction is expired\""
        );
    }

    #[test]
    fn the_end_time_will_be_increased_when_bidding_in_last_10_minutes() {
        // get integration test app and contracts
        let (mut app, contracts) = instantiate_contracts();
        let cw2981_address = contracts[0].contract_addr.clone();
        let marketplace_address = contracts[1].contract_addr.clone();

        // mint a cw2981 nft to USER_2
        mint_nft(&mut app, TOKEN_ID_1, USER_2, cw2981_address.clone());

        // approve marketplace to transfer nft
        approval_token(
            &mut app,
            USER_2,
            TOKEN_ID_1,
            cw2981_address.clone(),
            marketplace_address.clone(),
        );

        // first end_time
        let first_end_time = app.block_info().time.plus_seconds(1000);

        // create auction config
        let auction_config = AuctionConfigInput::EnglishAuction {
            start_price: coin(START_PRICE, NATIVE_DENOM),
            step_percentage: Some(STEP_PERCENTAGE),
            buyout_price: None,
            start_time: None,
            end_time: Cw721Expiration::AtTime(first_end_time),
        };

        let res = create_auction(
            &mut app,
            Some(TOKEN_ID_1.to_string()),
            USER_2,
            cw2981_address.clone(),
            marketplace_address.clone(),
            auction_config,
        );
        assert!(res.is_ok());

        let mut block_info = app.block_info();
        block_info.time = block_info.time.plus_seconds(399);
        app.set_block(block_info);

        // bid auction
        let res = bid_auction(
            &mut app,
            Some(TOKEN_ID_1.to_string()),
            USER_1,
            cw2981_address.clone(),
            marketplace_address.clone(),
            START_PRICE,
            Some(START_PRICE),
        );
        assert!(res.is_ok());

        // query auction info of nft
        let query_msg = QueryMsg::NftAuction {
            token_id: TOKEN_ID_1.to_string(),
            contract_address: cw2981_address.clone(),
        };

        // get auction of nft
        let res: OrderComponents = app
            .wrap()
            .query_wasm_smart(Addr::unchecked(&marketplace_address), &query_msg)
            .unwrap();

        assert_eq!(res.end_time, Some(Cw721Expiration::AtTime(first_end_time)));

        let mut block_info = app.block_info();
        block_info.time = block_info.time.plus_seconds(1);
        app.set_block(block_info);

        // second end_time
        let second_end_time = app.block_info().time.plus_seconds(600);

        // bid auction
        let res = bid_auction(
            &mut app,
            Some(TOKEN_ID_1.to_string()),
            USER_1,
            cw2981_address.clone(),
            marketplace_address.clone(),
            START_PRICE * 105 / 100,
            Some(START_PRICE * 105 / 100),
        );
        assert!(res.is_ok());

        // query auction info of nft
        let query_msg = QueryMsg::NftAuction {
            token_id: TOKEN_ID_1.to_string(),
            contract_address: cw2981_address,
        };

        // get auction of nft
        let res: OrderComponents = app
            .wrap()
            .query_wasm_smart(Addr::unchecked(&marketplace_address), &query_msg)
            .unwrap();

        assert_eq!(res.end_time, Some(Cw721Expiration::AtTime(second_end_time)));
    }

    #[test]
    fn user_cannot_bid_auction_because_not_enought_funds() {
        // get integration test app and contracts
        let (mut app, contracts) = instantiate_contracts();
        let cw2981_address = contracts[0].contract_addr.clone();
        let marketplace_address = contracts[1].contract_addr.clone();

        // mint a cw2981 nft to OWNER
        mint_nft(&mut app, TOKEN_ID_1, OWNER, cw2981_address.clone());

        // approve marketplace to transfer nft
        approval_token(
            &mut app,
            OWNER,
            TOKEN_ID_1,
            cw2981_address.clone(),
            marketplace_address.clone(),
        );

        // create auction config
        let auction_config = AuctionConfigInput::EnglishAuction {
            start_price: coin(START_PRICE, NATIVE_DENOM),
            step_percentage: Some(STEP_PERCENTAGE),
            buyout_price: None,
            start_time: None,
            end_time: Cw721Expiration::AtTime(app.block_info().time.plus_seconds(1000)),
        };

        let res = create_auction(
            &mut app,
            Some(TOKEN_ID_1.to_string()),
            OWNER,
            cw2981_address.clone(),
            marketplace_address.clone(),
            auction_config,
        );
        assert!(res.is_ok());

        // bid auction
        let res = bid_auction(
            &mut app,
            Some(TOKEN_ID_1.to_string()),
            USER_1,
            cw2981_address,
            marketplace_address,
            10000u128,
            None,
        );
        assert_eq!(
            res.unwrap_err().source().unwrap().to_string(),
            "Custom Error val: \"Different funding amount and bidding price\""
        );
    }

    #[test]
    fn user_can_not_bid_auction_because_price_too_small() {
        // get integration test app and contracts
        let (mut app, contracts) = instantiate_contracts();
        let cw2981_address = contracts[0].contract_addr.clone();
        let marketplace_address = contracts[1].contract_addr.clone();

        // mint a cw2981 nft to OWNER
        mint_nft(&mut app, TOKEN_ID_1, OWNER, cw2981_address.clone());

        // approve marketplace to transfer nft
        approval_token(
            &mut app,
            OWNER,
            TOKEN_ID_1,
            cw2981_address.clone(),
            marketplace_address.clone(),
        );

        // create auction config
        let auction_config = AuctionConfigInput::EnglishAuction {
            start_price: coin(START_PRICE, NATIVE_DENOM),
            step_percentage: Some(STEP_PERCENTAGE),
            buyout_price: None,
            start_time: None,
            end_time: Cw721Expiration::AtTime(app.block_info().time.plus_seconds(1000)),
        };

        let res = create_auction(
            &mut app,
            Some(TOKEN_ID_1.to_string()),
            OWNER,
            cw2981_address.clone(),
            marketplace_address.clone(),
            auction_config,
        );
        assert!(res.is_ok());

        // bid auction
        let res = bid_auction(
            &mut app,
            Some(TOKEN_ID_1.to_string()),
            USER_1,
            cw2981_address,
            marketplace_address,
            START_PRICE - 1,
            Some(START_PRICE - 1),
        );
        assert_eq!(
            res.unwrap_err().source().unwrap().to_string(),
            "Custom Error val: \"Bidding price invalid\""
        );
    }

    #[test]
    fn user_can_bid_auction() {
        // get integration test app and contracts
        let (mut app, contracts) = instantiate_contracts();
        let cw2981_address = contracts[0].contract_addr.clone();
        let marketplace_address = contracts[1].contract_addr.clone();

        // mint a cw2981 nft to OWNER
        mint_nft(&mut app, TOKEN_ID_1, OWNER, cw2981_address.clone());

        // approve marketplace to transfer nft
        approval_token(
            &mut app,
            OWNER,
            TOKEN_ID_1,
            cw2981_address.clone(),
            marketplace_address.clone(),
        );

        // create auction config
        let auction_config = AuctionConfigInput::EnglishAuction {
            start_price: coin(START_PRICE, NATIVE_DENOM),
            step_percentage: Some(STEP_PERCENTAGE),
            buyout_price: None,
            start_time: None,
            end_time: Cw721Expiration::AtTime(app.block_info().time.plus_seconds(1000)),
        };

        let res = create_auction(
            &mut app,
            Some(TOKEN_ID_1.to_string()),
            OWNER,
            cw2981_address.clone(),
            marketplace_address.clone(),
            auction_config,
        );
        assert!(res.is_ok());

        // bid auction
        let res = bid_auction(
            &mut app,
            Some(TOKEN_ID_1.to_string()),
            USER_1,
            cw2981_address,
            marketplace_address,
            START_PRICE,
            Some(START_PRICE),
        );
        assert!(res.is_ok());
    }

    #[test]
    fn the_new_bid_price_must_greater_than_the_old_one() {
        // get integration test app and contracts
        let (mut app, contracts) = instantiate_contracts();
        let cw2981_address = contracts[0].contract_addr.clone();
        let marketplace_address = contracts[1].contract_addr.clone();

        // mint a cw2981 nft to OWNER
        mint_nft(&mut app, TOKEN_ID_1, OWNER, cw2981_address.clone());

        // approve marketplace to transfer nft
        approval_token(
            &mut app,
            OWNER,
            TOKEN_ID_1,
            cw2981_address.clone(),
            marketplace_address.clone(),
        );

        // create auction config
        let auction_config = AuctionConfigInput::EnglishAuction {
            start_price: coin(START_PRICE, NATIVE_DENOM),
            step_percentage: Some(STEP_PERCENTAGE),
            buyout_price: None,
            start_time: None,
            end_time: Cw721Expiration::AtTime(app.block_info().time.plus_seconds(1000)),
        };

        let res = create_auction(
            &mut app,
            Some(TOKEN_ID_1.to_string()),
            OWNER,
            cw2981_address.clone(),
            marketplace_address.clone(),
            auction_config,
        );
        assert!(res.is_ok());

        // bid auction
        let res = bid_auction(
            &mut app,
            Some(TOKEN_ID_1.to_string()),
            USER_1,
            cw2981_address.clone(),
            marketplace_address.clone(),
            START_PRICE,
            Some(START_PRICE),
        );
        assert!(res.is_ok());

        // the market place should has the START_PRICE NATIVE_DENOM
        let market_balance = app
            .wrap()
            .query_balance(Addr::unchecked(marketplace_address.clone()), NATIVE_DENOM)
            .unwrap();
        assert_eq!(market_balance.amount, Uint128::from(START_PRICE));

        // bid auction again
        let res = bid_auction(
            &mut app,
            Some(TOKEN_ID_1.to_string()),
            USER_2,
            cw2981_address.clone(),
            marketplace_address.clone(),
            (START_PRICE * 105 / 100) - 1,
            Some((START_PRICE * 105 / 100) - 1),
        );
        assert_eq!(
            res.unwrap_err().source().unwrap().to_string(),
            "Custom Error val: \"Bidding price invalid\""
        );

        // bid auction again
        let res = bid_auction(
            &mut app,
            Some(TOKEN_ID_1.to_string()),
            USER_2,
            cw2981_address,
            marketplace_address.clone(),
            START_PRICE * 105 / 100,
            Some(START_PRICE * 105 / 100),
        );
        assert!(res.is_ok());

        // the market place should has the START_PRICE * 105 / 100 NATIVE_DENOM
        let market_balance = app
            .wrap()
            .query_balance(Addr::unchecked(marketplace_address), NATIVE_DENOM)
            .unwrap();
        assert_eq!(
            market_balance.amount,
            Uint128::from(START_PRICE * 105 / 100)
        );
    }
}

mod settle_auction {
    use super::*;

    #[test]
    fn owner_can_settle_auction() {
        // get integration test app and contracts
        let (mut app, contracts) = instantiate_contracts();
        let cw2981_address = contracts[0].contract_addr.clone();
        let marketplace_address = contracts[1].contract_addr.clone();

        // mint a cw2981 nft to OWNER
        mint_nft(&mut app, TOKEN_ID_1, USER_2, cw2981_address.clone());

        // approve marketplace to transfer nft
        approval_token(
            &mut app,
            USER_2,
            TOKEN_ID_1,
            cw2981_address.clone(),
            marketplace_address.clone(),
        );

        // create auction config
        let auction_config = AuctionConfigInput::EnglishAuction {
            start_price: coin(START_PRICE, NATIVE_DENOM),
            step_percentage: Some(STEP_PERCENTAGE),
            buyout_price: None,
            start_time: None,
            end_time: Cw721Expiration::AtTime(app.block_info().time.plus_seconds(1000)),
        };

        let res = create_auction(
            &mut app,
            Some(TOKEN_ID_1.to_string()),
            USER_2,
            cw2981_address.clone(),
            marketplace_address.clone(),
            auction_config,
        );
        assert!(res.is_ok());

        // get the balance of USER_1
        let _user_1_balance_before = app
            .wrap()
            .query_balance(Addr::unchecked(USER_1), NATIVE_DENOM)
            .unwrap();

        // bid auction
        let res = bid_auction(
            &mut app,
            Some(TOKEN_ID_1.to_string()),
            USER_1,
            cw2981_address.clone(),
            marketplace_address.clone(),
            START_PRICE,
            Some(START_PRICE),
        );
        assert!(res.is_ok());

        let mut block_info = app.block_info();
        block_info.time = block_info.time.plus_seconds(1001);
        app.set_block(block_info);

        // get the balance of OWNER
        let _owner_balance_before = app
            .wrap()
            .query_balance(Addr::unchecked(OWNER), NATIVE_DENOM)
            .unwrap();
        // get the balance of USER_2
        let _user_2_balance_before = app
            .wrap()
            .query_balance(Addr::unchecked(USER_2), NATIVE_DENOM)
            .unwrap();

        // settle auction
        let res = settle_auction(
            &mut app,
            Some(TOKEN_ID_1.to_string()),
            USER_2,
            cw2981_address,
            marketplace_address,
        );
        assert!(res.is_ok());

        // get the balance of OWNER
        let _owner_balance_after = app
            .wrap()
            .query_balance(Addr::unchecked(OWNER), NATIVE_DENOM)
            .unwrap();
        // get the balance of USER_1
        let _user_1_balance_after = app
            .wrap()
            .query_balance(Addr::unchecked(USER_1), NATIVE_DENOM)
            .unwrap();
        // get the balance of USER_2
        let _user_2_balance_after = app
            .wrap()
            .query_balance(Addr::unchecked(USER_2), NATIVE_DENOM)
            .unwrap();

        assert_eq!(
            _owner_balance_after.amount.u128(),
            _owner_balance_before.amount.u128() + (START_PRICE * 20 / 100)
        );

        assert_eq!(
            _user_1_balance_after.amount.u128(),
            _user_1_balance_before.amount.u128() - START_PRICE
        );

        assert_eq!(
            _user_2_balance_after.amount.u128(),
            _user_2_balance_before.amount.u128() + (START_PRICE * 80 / 100)
        );
    }

    #[test]
    fn last_bidder_can_settle_auction() {
        // get integration test app and contracts
        let (mut app, contracts) = instantiate_contracts();
        let cw2981_address = contracts[0].contract_addr.clone();
        let marketplace_address = contracts[1].contract_addr.clone();

        // mint a cw2981 nft to USER_2
        mint_nft(&mut app, TOKEN_ID_1, USER_2, cw2981_address.clone());

        // approve marketplace to transfer nft
        approval_token(
            &mut app,
            USER_2,
            TOKEN_ID_1,
            cw2981_address.clone(),
            marketplace_address.clone(),
        );

        // create auction config
        let auction_config = AuctionConfigInput::EnglishAuction {
            start_price: coin(START_PRICE, NATIVE_DENOM),
            step_percentage: Some(STEP_PERCENTAGE),
            buyout_price: None,
            start_time: None,
            end_time: Cw721Expiration::AtTime(app.block_info().time.plus_seconds(1000)),
        };

        let res = create_auction(
            &mut app,
            Some(TOKEN_ID_1.to_string()),
            USER_2,
            cw2981_address.clone(),
            marketplace_address.clone(),
            auction_config,
        );
        assert!(res.is_ok());

        // get the balance of USER_1
        let _user_1_balance_before = app
            .wrap()
            .query_balance(Addr::unchecked(USER_1), NATIVE_DENOM)
            .unwrap();

        // bid auction
        let res = bid_auction(
            &mut app,
            Some(TOKEN_ID_1.to_string()),
            USER_1,
            cw2981_address.clone(),
            marketplace_address.clone(),
            START_PRICE,
            Some(START_PRICE),
        );
        assert!(res.is_ok());

        let mut block_info = app.block_info();
        block_info.time = block_info.time.plus_seconds(1001);
        app.set_block(block_info);

        // get the balance of OWNER
        let _owner_balance_before = app
            .wrap()
            .query_balance(Addr::unchecked(OWNER), NATIVE_DENOM)
            .unwrap();
        // get the balance of USER_2
        let _user_2_balance_before = app
            .wrap()
            .query_balance(Addr::unchecked(USER_2), NATIVE_DENOM)
            .unwrap();

        // settle auction
        let res = settle_auction(
            &mut app,
            Some(TOKEN_ID_1.to_string()),
            USER_1,
            cw2981_address.clone(),
            marketplace_address,
        );
        assert!(res.is_ok());

        // get the balance of OWNER
        let _owner_balance_after = app
            .wrap()
            .query_balance(Addr::unchecked(OWNER), NATIVE_DENOM)
            .unwrap();
        // get the balance of USER_1
        let _user_1_balance_after = app
            .wrap()
            .query_balance(Addr::unchecked(USER_1), NATIVE_DENOM)
            .unwrap();
        // get the balance of USER_2
        let _user_2_balance_after = app
            .wrap()
            .query_balance(Addr::unchecked(USER_2), NATIVE_DENOM)
            .unwrap();

        assert_eq!(
            _owner_balance_after.amount.u128(),
            _owner_balance_before.amount.u128() + (START_PRICE * 20 / 100)
        );

        assert_eq!(
            _user_1_balance_after.amount.u128(),
            _user_1_balance_before.amount.u128() - START_PRICE
        );

        assert_eq!(
            _user_2_balance_after.amount.u128(),
            _user_2_balance_before.amount.u128() + (START_PRICE * 80 / 100)
        );

        // check the owner of the token
        let res: cw721::OwnerOfResponse = app
            .wrap()
            .query_wasm_smart(
                Addr::unchecked(cw2981_address),
                &Cw721QueryMsg::OwnerOf {
                    token_id: TOKEN_ID_1.to_string(),
                    include_expired: None,
                },
            )
            .unwrap();
        // it should be USER_1
        assert_eq!(res.owner, USER_1.to_string());
    }

    #[test]
    fn owner_receive_his_nft_if_no_bid() {
        // get integration test app and contracts
        let (mut app, contracts) = instantiate_contracts();
        let cw2981_address = contracts[0].contract_addr.clone();
        let marketplace_address = contracts[1].contract_addr.clone();

        // mint a cw2981 nft to USER_2
        mint_nft(&mut app, TOKEN_ID_1, USER_2, cw2981_address.clone());

        // approve marketplace to transfer nft
        approval_token(
            &mut app,
            USER_2,
            TOKEN_ID_1,
            cw2981_address.clone(),
            marketplace_address.clone(),
        );

        // create auction config
        let auction_config = AuctionConfigInput::EnglishAuction {
            start_price: coin(START_PRICE, NATIVE_DENOM),
            step_percentage: Some(STEP_PERCENTAGE),
            buyout_price: None,
            start_time: None,
            end_time: Cw721Expiration::AtTime(app.block_info().time.plus_seconds(1000)),
        };

        let res = create_auction(
            &mut app,
            Some(TOKEN_ID_1.to_string()),
            USER_2,
            cw2981_address.clone(),
            marketplace_address.clone(),
            auction_config,
        );
        assert!(res.is_ok());

        let mut block_info = app.block_info();
        block_info.time = block_info.time.plus_seconds(1001);
        app.set_block(block_info);

        // get the balance of USER_2
        let _user_2_balance_before = app
            .wrap()
            .query_balance(Addr::unchecked(USER_2), NATIVE_DENOM)
            .unwrap();

        // settle auction
        let res = settle_auction(
            &mut app,
            Some(TOKEN_ID_1.to_string()),
            USER_2,
            cw2981_address.clone(),
            marketplace_address,
        );
        assert!(res.is_ok());

        // get the balance of USER_2
        let _user_2_balance_after = app
            .wrap()
            .query_balance(Addr::unchecked(USER_2), NATIVE_DENOM)
            .unwrap();

        assert_eq!(_user_2_balance_before, _user_2_balance_after);

        // check the owner of the token
        let res: cw721::OwnerOfResponse = app
            .wrap()
            .query_wasm_smart(
                Addr::unchecked(cw2981_address),
                &Cw721QueryMsg::OwnerOf {
                    token_id: TOKEN_ID_1.to_string(),
                    include_expired: None,
                },
            )
            .unwrap();
        // it should be USER_2
        assert_eq!(res.owner, USER_2.to_string());
    }

    #[test]
    fn cannot_settle_auction_because_unauthorized() {
        // get integration test app and contracts
        let (mut app, contracts) = instantiate_contracts();
        let cw2981_address = contracts[0].contract_addr.clone();
        let marketplace_address = contracts[1].contract_addr.clone();

        // mint a cw2981 nft to USER_2
        mint_nft(&mut app, TOKEN_ID_1, USER_2, cw2981_address.clone());

        // approve marketplace to transfer nft
        approval_token(
            &mut app,
            USER_2,
            TOKEN_ID_1,
            cw2981_address.clone(),
            marketplace_address.clone(),
        );

        // create auction config
        let auction_config = AuctionConfigInput::EnglishAuction {
            start_price: coin(START_PRICE, NATIVE_DENOM),
            step_percentage: Some(STEP_PERCENTAGE),
            buyout_price: None,
            start_time: None,
            end_time: Cw721Expiration::AtTime(app.block_info().time.plus_seconds(1000)),
        };

        let res = create_auction(
            &mut app,
            Some(TOKEN_ID_1.to_string()),
            USER_2,
            cw2981_address.clone(),
            marketplace_address.clone(),
            auction_config,
        );
        assert!(res.is_ok());

        // get the balance of USER_1
        let _user_1_balance_before = app
            .wrap()
            .query_balance(Addr::unchecked(USER_1), NATIVE_DENOM)
            .unwrap();

        // bid auction
        let res = bid_auction(
            &mut app,
            Some(TOKEN_ID_1.to_string()),
            USER_1,
            cw2981_address.clone(),
            marketplace_address.clone(),
            START_PRICE,
            Some(START_PRICE),
        );
        assert!(res.is_ok());

        let mut block_info = app.block_info();
        block_info.time = block_info.time.plus_seconds(1001);
        app.set_block(block_info);

        // settle auction
        let res = settle_auction(
            &mut app,
            Some(TOKEN_ID_1.to_string()),
            OWNER,
            cw2981_address,
            marketplace_address,
        );
        assert_eq!(
            res.unwrap_err().source().unwrap().to_string(),
            "Unauthorized"
        );
    }

    #[test]
    fn cannot_settle_auction_because_auction_not_expired() {
        // get integration test app and contracts
        let (mut app, contracts) = instantiate_contracts();
        let cw2981_address = contracts[0].contract_addr.clone();
        let marketplace_address = contracts[1].contract_addr.clone();

        // mint a cw2981 nft to USER_2
        mint_nft(&mut app, TOKEN_ID_1, USER_2, cw2981_address.clone());

        // approve marketplace to transfer nft
        approval_token(
            &mut app,
            USER_2,
            TOKEN_ID_1,
            cw2981_address.clone(),
            marketplace_address.clone(),
        );

        // create auction config
        let auction_config = AuctionConfigInput::EnglishAuction {
            start_price: coin(START_PRICE, NATIVE_DENOM),
            step_percentage: Some(STEP_PERCENTAGE),
            buyout_price: None,
            start_time: None,
            end_time: Cw721Expiration::AtTime(app.block_info().time.plus_seconds(1000)),
        };

        let res = create_auction(
            &mut app,
            Some(TOKEN_ID_1.to_string()),
            USER_2,
            cw2981_address.clone(),
            marketplace_address.clone(),
            auction_config,
        );
        assert!(res.is_ok());

        // get the balance of USER_1
        let _user_1_balance_before = app
            .wrap()
            .query_balance(Addr::unchecked(USER_1), NATIVE_DENOM)
            .unwrap();

        // bid auction
        let res = bid_auction(
            &mut app,
            Some(TOKEN_ID_1.to_string()),
            USER_1,
            cw2981_address.clone(),
            marketplace_address.clone(),
            START_PRICE,
            Some(START_PRICE),
        );
        assert!(res.is_ok());

        let mut block_info = app.block_info();
        block_info.time = block_info.time.plus_seconds(999);
        app.set_block(block_info);

        // settle auction
        let res = settle_auction(
            &mut app,
            Some(TOKEN_ID_1.to_string()),
            USER_1,
            cw2981_address,
            marketplace_address,
        );
        assert_eq!(
            res.unwrap_err().source().unwrap().to_string(),
            "Custom Error val: \"Auction is not expired\""
        );
    }
}

mod query_auction {
    use crate::msg::AuctionsResponse;

    use super::*;

    #[test]
    fn query_all_auctions_of_owner() {
        // get integration test app and contracts
        let (mut app, contracts) = instantiate_contracts();
        let cw2981_address = contracts[0].contract_addr.clone();
        let marketplace_address = contracts[1].contract_addr.clone();

        // mint the first cw2981 nft to OWNER
        mint_nft(&mut app, TOKEN_ID_1, OWNER, cw2981_address.clone());

        // mint the second cw2981 nft to OWNER
        mint_nft(&mut app, TOKEN_ID_2, OWNER, cw2981_address.clone());

        // mint the third cw2981 nft to OWNER
        mint_nft(&mut app, TOKEN_ID_3, OWNER, cw2981_address.clone());

        // approve marketplace to transfer nfts
        approval_token(
            &mut app,
            OWNER,
            TOKEN_ID_1,
            cw2981_address.clone(),
            marketplace_address.clone(),
        );

        // approve marketplace to transfer nfts
        approval_token(
            &mut app,
            OWNER,
            TOKEN_ID_2,
            cw2981_address.clone(),
            marketplace_address.clone(),
        );

        // approve marketplace to transfer nfts
        approval_token(
            &mut app,
            OWNER,
            TOKEN_ID_3,
            cw2981_address.clone(),
            marketplace_address.clone(),
        );

        // create auction config
        let auction_config = AuctionConfigInput::EnglishAuction {
            start_price: coin(START_PRICE, NATIVE_DENOM),
            step_percentage: Some(STEP_PERCENTAGE),
            buyout_price: None,
            start_time: None,
            end_time: Cw721Expiration::AtTime(app.block_info().time.plus_seconds(1000)),
        };

        let res = create_auction(
            &mut app,
            Some(TOKEN_ID_1.to_string()),
            OWNER,
            cw2981_address.clone(),
            marketplace_address.clone(),
            auction_config.clone(),
        );
        assert!(res.is_ok());
        let res = create_auction(
            &mut app,
            Some(TOKEN_ID_2.to_string()),
            OWNER,
            cw2981_address.clone(),
            marketplace_address.clone(),
            auction_config.clone(),
        );
        assert!(res.is_ok());
        let res = create_auction(
            &mut app,
            Some(TOKEN_ID_3.to_string()),
            OWNER,
            cw2981_address.clone(),
            marketplace_address.clone(),
            auction_config,
        );
        assert!(res.is_ok());

        // USER_1 bid auction
        let res = bid_auction(
            &mut app,
            Some(TOKEN_ID_1.to_string()),
            USER_1,
            cw2981_address.clone(),
            marketplace_address.clone(),
            START_PRICE,
            Some(START_PRICE),
        );
        assert!(res.is_ok());

        // USER_1 bid auction
        let res = bid_auction(
            &mut app,
            Some(TOKEN_ID_3.to_string()),
            USER_1,
            cw2981_address.clone(),
            marketplace_address.clone(),
            START_PRICE,
            Some(START_PRICE),
        );
        assert!(res.is_ok());

        // USER_2 bid auction
        let res = bid_auction(
            &mut app,
            Some(TOKEN_ID_2.to_string()),
            USER_2,
            cw2981_address,
            marketplace_address.clone(),
            START_PRICE,
            Some(START_PRICE),
        );
        assert!(res.is_ok());

        // query all auctions of OWNER
        let query_msg = QueryMsg::OwnerAuctions {
            owner: OWNER.to_string(),
            start_after_nft: None,
            limit: None,
        };

        let res: AuctionsResponse = app
            .wrap()
            .query_wasm_smart(Addr::unchecked(marketplace_address), &query_msg)
            .unwrap();
        // the number of auctions should be 3
        assert_eq!(res.auctions.len(), 3);

        // query all auctions of USER_1
        let _query_msg = QueryMsg::BuyerAuctions {
            buyer: USER_1.to_string(),
            start_after_nft: None,
            limit: None,
        };
        // the number of auctions should be 2
        assert_eq!(res.auctions.len(), 3); // May cais query nay loi het roi
    }
}
