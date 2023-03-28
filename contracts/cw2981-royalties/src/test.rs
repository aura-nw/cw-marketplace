use crate::msg::{
    CheckRoyaltiesResponse, Cw2981ExecuteMsg, Cw2981QueryMsg, InstantiateMsg, RoyaltiesInfoResponse,
};
use crate::{
    check_royalties, execute, instantiate, query, query_royalties_info, Cw2981Contract, ExecuteMsg,
    Metadata, QueryMsg,
};

use cosmwasm_std::{from_binary, Uint128, Uint256};

use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cw721::{Cw721Query, NftInfoResponse};
use cw721_base::MintMsg;

const CREATOR: &str = "creator";

#[test]
fn use_metadata_extension() {
    let mut deps = mock_dependencies();
    let contract = Cw2981Contract::default();

    let info = mock_info(CREATOR, &[]);
    // let royalty_percentage = 101
    let init_msg = InstantiateMsg {
        name: "SpaceShips".to_string(),
        symbol: "SPACE".to_string(),
        minter: CREATOR.to_string(),
        royalty_percentage: Some(50),
        royalty_payment_address: Some("john".to_string()),
        final_proof: None,
    };
    instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap();

    let expected_extension = Some(Metadata {
        description: Some("Spaceship with Warp Drive".into()),
        name: Some("Starship USS Enterprise".to_string()),
        royalty_percentage: Some(50),
        royalty_payment_address: Some("john".to_string()),
        ..Metadata::default()
    });

    let token_id = "Enterprise";
    let mint_msg = MintMsg {
        token_id: token_id.to_string(),
        owner: "john".to_string(),
        token_uri: Some("https://starships.example.com/Starship/Enterprise.json".into()),
        extension: Some(Metadata {
            description: Some("Spaceship with Warp Drive".into()),
            name: Some("Starship USS Enterprise".to_string()),
            ..Metadata::default()
        }),
    };

    let exec_msg = ExecuteMsg::Mint(mint_msg.clone());
    execute(deps.as_mut(), mock_env(), info, exec_msg).unwrap();

    let res = contract.nft_info(deps.as_ref(), token_id.into()).unwrap();
    assert_eq!(res.token_uri, mint_msg.token_uri);
    assert_eq!(res.extension, expected_extension);
}

#[test]
fn validate_royalty_information() {
    let mut deps = mock_dependencies();
    let _contract = Cw2981Contract::default();

    let info = mock_info(CREATOR, &[]);
    // let royalty_percentage = 101
    let init_msg = InstantiateMsg {
        name: "SpaceShips".to_string(),
        symbol: "SPACE".to_string(),
        minter: CREATOR.to_string(),
        royalty_percentage: Some(101),
        royalty_payment_address: Some("john".to_string()),
        final_proof: None,
    };
    // instantiate will fail
    let res = instantiate(deps.as_mut(), mock_env(), info, init_msg);
    assert!(res.is_err());
}

#[test]
fn not_allow_setting_royalty_when_minting() {
    let mut deps = mock_dependencies();
    let _contract = Cw2981Contract::default();

    let info = mock_info(CREATOR, &[]);
    let init_msg = InstantiateMsg {
        name: "SpaceShips".to_string(),
        symbol: "SPACE".to_string(),
        minter: CREATOR.to_string(),
        royalty_percentage: Some(50),
        royalty_payment_address: Some("john".to_string()),
        final_proof: None,
    };
    instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap();

    let token_id = "Enterprise";
    let mint_msg = MintMsg {
        token_id: token_id.to_string(),
        owner: "john".to_string(),
        token_uri: Some("https://starships.example.com/Starship/Enterprise.json".into()),
        extension: Some(Metadata {
            description: Some("Spaceship with Warp Drive".into()),
            name: Some("Starship USS Enterprise".to_string()),
            royalty_percentage: Some(50),
            royalty_payment_address: Some("john".to_string()),
            ..Metadata::default()
        }),
    };

    let exec_msg = ExecuteMsg::Mint(mint_msg);
    let res = execute(deps.as_mut(), mock_env(), info, exec_msg);
    assert!(res.is_err());
}

#[test]
fn check_royalties_response() {
    let mut deps = mock_dependencies();
    let _contract = Cw2981Contract::default();

    let info = mock_info(CREATOR, &[]);
    let init_msg = InstantiateMsg {
        name: "SpaceShips".to_string(),
        symbol: "SPACE".to_string(),
        minter: CREATOR.to_string(),
        royalty_percentage: Some(50),
        royalty_payment_address: Some("john".to_string()),
        final_proof: None,
    };
    instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap();

    let token_id = "Enterprise";
    let mint_msg = MintMsg {
        token_id: token_id.to_string(),
        owner: "john".to_string(),
        token_uri: Some("https://starships.example.com/Starship/Enterprise.json".into()),
        extension: Some(Metadata {
            description: Some("Spaceship with Warp Drive".into()),
            name: Some("Starship USS Enterprise".to_string()),
            ..Metadata::default()
        }),
    };
    let exec_msg = ExecuteMsg::Mint(mint_msg);
    execute(deps.as_mut(), mock_env(), info, exec_msg).unwrap();

    let expected = CheckRoyaltiesResponse {
        royalty_payments: true,
    };
    let res = check_royalties(deps.as_ref()).unwrap();
    assert_eq!(res, expected);

    // also check the longhand way
    let query_msg = QueryMsg::Extension {
        msg: Cw2981QueryMsg::CheckRoyalties {},
    };
    let query_res: CheckRoyaltiesResponse =
        from_binary(&query(deps.as_ref(), mock_env(), query_msg).unwrap()).unwrap();
    assert_eq!(query_res, expected);
}

#[test]
fn check_token_royalties() {
    let mut deps = mock_dependencies();

    let royalty_payment_address = "jeanluc".to_string();

    let info = mock_info(CREATOR, &[]);
    let init_msg = InstantiateMsg {
        name: "SpaceShips".to_string(),
        symbol: "SPACE".to_string(),
        minter: CREATOR.to_string(),
        royalty_percentage: Some(10),
        royalty_payment_address: Some(royalty_payment_address.clone()),
        final_proof: None,
    };
    instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap();

    let token_id = "Enterprise";
    let mint_msg = MintMsg {
        token_id: token_id.to_string(),
        owner: "jeanluc".to_string(),
        token_uri: Some("https://starships.example.com/Starship/Enterprise.json".into()),
        extension: Some(Metadata {
            description: Some("Spaceship with Warp Drive".into()),
            name: Some("Starship USS Enterprise".to_string()),
            ..Metadata::default()
        }),
    };
    let exec_msg = ExecuteMsg::Mint(mint_msg);
    execute(deps.as_mut(), mock_env(), info.clone(), exec_msg).unwrap();

    let expected = RoyaltiesInfoResponse {
        address: royalty_payment_address.clone(),
        royalty_amount: Uint128::new(10),
    };
    let res = query_royalties_info(deps.as_ref(), token_id.to_string(), Uint128::new(100)).unwrap();
    assert_eq!(res, expected);

    // also check the longhand way
    let query_msg = QueryMsg::Extension {
        msg: Cw2981QueryMsg::RoyaltyInfo {
            token_id: token_id.to_string(),
            sale_price: Uint128::new(100),
        },
    };
    let query_res: RoyaltiesInfoResponse =
        from_binary(&query(deps.as_ref(), mock_env(), query_msg).unwrap()).unwrap();
    assert_eq!(query_res, expected);

    // check for rounding down
    // which is the default behaviour
    let voyager_token_id = "Voyager";
    let second_mint_msg = MintMsg {
        token_id: voyager_token_id.to_string(),
        owner: "janeway".to_string(),
        token_uri: Some("https://starships.example.com/Starship/Voyager.json".into()),
        extension: Some(Metadata {
            description: Some("Spaceship with Warp Drive".into()),
            name: Some("Starship USS Voyager".to_string()),
            ..Metadata::default()
        }),
    };
    let voyager_exec_msg = ExecuteMsg::Mint(second_mint_msg);
    execute(deps.as_mut(), mock_env(), info, voyager_exec_msg).unwrap();

    // 43 x 0.10 (i.e., 10%) should be 4.3
    // we expect this to be rounded down to 1
    let voyager_expected = RoyaltiesInfoResponse {
        address: royalty_payment_address,
        royalty_amount: Uint128::new(4),
    };

    let res = query_royalties_info(
        deps.as_ref(),
        voyager_token_id.to_string(),
        Uint128::new(43),
    )
    .unwrap();
    assert_eq!(res, voyager_expected);
}

#[test]
fn check_token_without_royalties() {
    let mut deps = mock_dependencies();

    let info = mock_info(CREATOR, &[]);
    let init_msg = InstantiateMsg {
        name: "SpaceShips".to_string(),
        symbol: "SPACE".to_string(),
        minter: CREATOR.to_string(),
        royalty_percentage: None,
        royalty_payment_address: None,
        final_proof: None,
    };
    instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap();

    let token_id = "Enterprise";
    let mint_msg = MintMsg {
        token_id: token_id.to_string(),
        owner: "jeanluc".to_string(),
        token_uri: Some("https://starships.example.com/Starship/Enterprise.json".into()),
        extension: Some(Metadata {
            description: Some("Spaceship with Warp Drive".into()),
            name: Some("Starship USS Enterprise".to_string()),
            ..Metadata::default()
        }),
    };
    let exec_msg = ExecuteMsg::Mint(mint_msg);
    execute(deps.as_mut(), mock_env(), info, exec_msg).unwrap();

    let expected = RoyaltiesInfoResponse {
        address: "".to_string(),
        royalty_amount: Uint128::new(0),
    };
    let res = query_royalties_info(deps.as_ref(), token_id.to_string(), Uint128::new(100)).unwrap();
    assert_eq!(res, expected);

    // also check the longhand way
    let query_msg = QueryMsg::Extension {
        msg: Cw2981QueryMsg::RoyaltyInfo {
            token_id: token_id.to_string(),
            sale_price: Uint128::new(100),
        },
    };
    let query_res: RoyaltiesInfoResponse =
        from_binary(&query(deps.as_ref(), mock_env(), query_msg).unwrap()).unwrap();
    assert_eq!(query_res, expected);
}

#[test]
fn check_token_without_extension() {
    let mut deps = mock_dependencies();

    let info = mock_info(CREATOR, &[]);
    let init_msg = InstantiateMsg {
        name: "SpaceShips".to_string(),
        symbol: "SPACE".to_string(),
        minter: CREATOR.to_string(),
        royalty_percentage: None,
        royalty_payment_address: None,
        final_proof: None,
    };
    instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap();

    let token_id = "Enterprise";
    let mint_msg = MintMsg {
        token_id: token_id.to_string(),
        owner: "jeanluc".to_string(),
        token_uri: Some("https://starships.example.com/Starship/Enterprise.json".into()),
        extension: None,
    };
    let exec_msg = ExecuteMsg::Mint(mint_msg);
    execute(deps.as_mut(), mock_env(), info, exec_msg).unwrap();

    let expected = RoyaltiesInfoResponse {
        address: "".to_string(),
        royalty_amount: Uint128::new(0),
    };
    let res = query_royalties_info(deps.as_ref(), token_id.to_string(), Uint128::new(100)).unwrap();
    assert_eq!(res, expected);

    // also check the longhand way
    let query_msg = QueryMsg::Extension {
        msg: Cw2981QueryMsg::RoyaltyInfo {
            token_id: token_id.to_string(),
            sale_price: Uint128::new(100),
        },
    };
    let query_res: RoyaltiesInfoResponse =
        from_binary(&query(deps.as_ref(), mock_env(), query_msg).unwrap()).unwrap();
    assert_eq!(query_res, expected);
}

#[test]
fn check_token_with_provenance_distribution() {
    let mut deps = mock_dependencies();

    let info = mock_info(CREATOR, &[]);
    let init_msg = InstantiateMsg {
        name: "SpaceShips".to_string(),
        symbol: "SPACE".to_string(),
        minter: CREATOR.to_string(),
        royalty_percentage: None,
        royalty_payment_address: None,
        final_proof: Some("final_proof".to_string()),
    };
    instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap();

    let token_id = "1";

    // mint the token
    let mint_msg = MintMsg {
        token_id: token_id.to_string(),
        owner: "creator".to_string(),
        token_uri: Some("https://starships.example.com/Starship/{token_id}.json".into()),
        extension: None,
    };

    let exec_msg = ExecuteMsg::Mint(mint_msg);
    execute(deps.as_mut(), mock_env(), info.clone(), exec_msg).unwrap();

    // query nft info
    let query_msg = QueryMsg::NftInfo {
        token_id: token_id.to_string(),
    };
    let query_res: NftInfoResponse<Metadata> =
        from_binary(&query(deps.as_ref(), mock_env(), query_msg).unwrap()).unwrap();
    assert_eq!(
        query_res.token_uri.unwrap(),
        "https://starships.example.com/Starship/{token_id}.json".to_string()
    );

    // distribute the nfts
    let distribute_msg = Cw2981ExecuteMsg::DistributeNfts {
        elements_proof: "elements_proof".to_string(),
        token_uri_anchor: Uint256::from(2u32),
        distinct_elements_number: 4,
    };

    let exec_msg = ExecuteMsg::Extension {
        msg: distribute_msg,
    };
    execute(deps.as_mut(), mock_env(), info, exec_msg).unwrap();

    // query nft info
    let query_msg = QueryMsg::NftInfo {
        token_id: token_id.to_string(),
    };
    let query_res: NftInfoResponse<Metadata> =
        from_binary(&query(deps.as_ref(), mock_env(), query_msg).unwrap()).unwrap();
    assert_eq!(
        query_res.token_uri.unwrap(),
        "https://starships.example.com/Starship/3.json".to_string()
    );
}
