use crate::msg::ExecuteMsg;
use crate::state::NFT;
use crate::test_setup::env::{instantiate_contracts, NATIVE_DENOM, OWNER};

use anyhow::Result as AnyResult;

use cosmwasm_std::{coin, Addr};
use cw_multi_test::{App, AppResponse, Executor};

use crate::state::AuctionConfigInput;
use cw2981_royalties::{Metadata, MintMsg, QueryMsg as Cw721QueryMsg};

use cw721::OwnerOfResponse;
use cw721_base::msg::ExecuteMsg as Cw721ExecuteMsg;

const TOKEN_ID_1: &str = "token1";
// const TOKEN_ID_2: &str = "token2";
// const TOKEN_ID_3: &str = "token3";

const START_PRICE: u128 = 10000000;
const END_PRICE: u128 = 1000000;

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

fn _bid_auction(
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

fn _settle_auction(
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
        let auction_config = AuctionConfigInput::DutchAuction {
            start_price: coin(START_PRICE, NATIVE_DENOM),
            end_price: END_PRICE,
            start_time: None,
            end_time: app.block_info().time.plus_seconds(1000).nanos(),
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
