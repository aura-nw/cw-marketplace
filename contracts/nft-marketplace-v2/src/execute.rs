use cosmwasm_std::{
    to_json_binary, Addr, BankMsg, Coin, CosmosMsg, DepsMut, Env, MessageInfo, QueryRequest,
    Response, StdResult, Uint128, WasmMsg, WasmQuery,
};
use cw20::{AllowanceResponse, Cw20ExecuteMsg, Cw20QueryMsg};
use cw2981_royalties::{msg::RoyaltiesInfoResponse, QueryMsg as Cw2981QueryMsg};
use cw721::{Cw721ExecuteMsg, Cw721QueryMsg, Expiration as Cw721Expiration};

use crate::state::{listing_key, offer_key, CONFIG, LISTINGS, OFFERS};
use crate::structs::{
    consideration_item, offer_item, order_id, Asset, AuctionConfig, ConsiderationItem, Cw20Asset,
    ItemType, NativeAsset, NftAsset, OfferItem, Order, OrderType, PaymentAsset,
};
use crate::ContractError;

pub fn execute_list_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    asset: NftAsset,
    auction_config: AuctionConfig,
) -> Result<Response, ContractError> {
    // auction time must be valid first
    if auction_config.is_valid() {
        return Err(ContractError::CustomError {
            val: "Invalid auction config".to_string(),
        });
    }

    let contract_address = asset.contract_address.clone();
    // token_id is required
    if asset.token_id.is_none() {
        return Err(ContractError::CustomError {
            val: "Token ID is required".to_string(),
        });
    }
    let token_id = asset.token_id.unwrap();

    // check if user is the owner of the token
    let owner_response: StdResult<cw721::OwnerOfResponse> =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: contract_address.to_string(),
            msg: to_json_binary(&Cw721QueryMsg::OwnerOf {
                token_id: token_id.clone(),
                include_expired: Some(false),
            })?,
        }));
    match owner_response {
        Ok(owner) => {
            if owner.owner != info.sender {
                return Err(ContractError::Unauthorized {});
            }
        }
        Err(_) => {
            return Err(ContractError::Unauthorized {});
        }
    }

    // check that user approves this contract to manage this token
    // for now, we require never expired approval
    let approval_response: StdResult<cw721::ApprovalResponse> =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: contract_address.to_string(),
            msg: to_json_binary(&Cw721QueryMsg::Approval {
                token_id: token_id.clone(),
                spender: env.contract.address.to_string(),
                include_expired: Some(true),
            })?,
        }));
    match approval_response {
        Ok(approval) => match approval.approval.expires {
            Cw721Expiration::Never {} => {}
            _ => return Err(ContractError::Unauthorized {}),
        },
        Err(_) => {
            return Err(ContractError::CustomError {
                val: "Require never expired approval".to_string(),
            });
        }
    }

    // the auction_config must be FixedPrice
    match auction_config.clone() {
        AuctionConfig::FixedPrice {
            price,
            start_time,
            end_time,
        } => {
            // add new listing to orders
            let order_id = order_id(&info.sender, &contract_address, &token_id);

            let offer_item = OfferItem {
                item_type: ItemType::CW721,
                item: Asset::Nft(NftAsset {
                    contract_address: contract_address.clone(),
                    token_id: Some(token_id.clone()),
                }),
                start_amount: 1,
                end_amount: 1,
            };

            let consideration_item = match price {
                PaymentAsset::Native { denom, amount } => ConsiderationItem {
                    item_type: ItemType::NATIVE,
                    item: Asset::Native(NativeAsset { denom, amount }),
                    start_amount: amount,
                    end_amount: amount,
                    recipient: info.sender.clone(),
                },
                PaymentAsset::Cw20 {
                    contract_address,
                    amount,
                } => ConsiderationItem {
                    item_type: ItemType::CW20,
                    item: Asset::Cw20(Cw20Asset {
                        contract_address,
                        amount,
                    }),
                    start_amount: amount,
                    end_amount: amount,
                    recipient: info.sender.clone(),
                },
            };

            let new_listing = Order {
                order_type: OrderType::LISTING,
                order_id,
                owner: info.sender.clone(),
                offer: vec![offer_item],
                consideration: vec![consideration_item],
                start_time,
                end_time,
            };

            let listing_key = listing_key(&contract_address, &token_id);
            // we will override the order if it already exists, so that we can update the auction config
            LISTINGS.update(
                deps.storage,
                listing_key,
                |_old| -> Result<Order, ContractError> { Ok(new_listing) },
            )?;
        }
        _ => {
            return Err(ContractError::CustomError {
                val: "Auction Config Error".to_string(),
            });
        }
    }

    let auction_config_str = serde_json::to_string(&auction_config);

    match auction_config_str {
        Ok(auction_config_str) => Ok(Response::new()
            .add_attribute("method", "list_nft")
            .add_attribute("contract_address", contract_address)
            .add_attribute("token_id", token_id)
            .add_attribute("auction_config", auction_config_str)
            .add_attribute("seller", info.sender.to_string())),
        Err(_) => Err(ContractError::CustomError {
            val: ("Auction Config Error".to_string()),
        }),
    }
}

pub fn execute_buy(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    asset: NftAsset,
) -> Result<Response, ContractError> {
    let contract_address = asset.contract_address.clone();
    // token_id is required
    if asset.token_id.is_none() {
        return Err(ContractError::CustomError {
            val: "Token ID is required".to_string(),
        });
    }
    let token_id = asset.token_id.unwrap();

    // get the listing
    let listing_key = listing_key(&contract_address, &token_id);
    let listing = LISTINGS.load(deps.storage, listing_key.clone())?;

    // check if owner of listing is the same as seller
    if info.sender == listing.owner {
        return Err(ContractError::CustomError {
            val: ("Owner cannot buy".to_string()),
        });
    }

    // remove the listing
    LISTINGS.remove(deps.storage, listing_key)?;

    // check if current block is after start_time
    if listing.start_time.is_some() && !listing.start_time.unwrap().is_expired(&env.block) {
        return Err(ContractError::CustomError {
            val: ("Auction not started".to_string()),
        });
    }

    if listing.end_time.is_some() && listing.end_time.unwrap().is_expired(&env.block) {
        return Err(ContractError::CustomError {
            val: format!(
                "Auction ended: {} {}",
                listing.end_time.unwrap(),
                env.block.time
            ),
        });
    }

    // message to transfer nft to buyer
    let mut res = Response::new().add_message(WasmMsg::Execute {
        contract_addr: contract_address.to_string(),
        msg: to_json_binary(&Cw721ExecuteMsg::TransferNft {
            recipient: info.sender.to_string(),
            token_id: token_id.clone(),
        })?,
        funds: vec![],
    });

    // transfer payment assets to the recipient of listing's consideration
    let payment_asset = PaymentAsset::from(listing.consideration[0].item.clone());
    let payment_messages = payment_with_royalty(
        &deps,
        &contract_address,
        &token_id,
        &payment_asset,
        &info.sender,
        &listing.consideration[0].recipient,
    );

    for payment_message in payment_messages {
        res = res.add_message(payment_message);
    }

    Ok(res
        .add_attribute("method", "buy")
        .add_attribute("contract_address", contract_address.to_string())
        .add_attribute("token_id", token_id.to_string())
        .add_attribute("buyer", info.sender))
}

pub fn execute_cancel(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    asset: NftAsset,
) -> Result<Response, ContractError> {
    let contract_address = asset.contract_address.clone();
    // token_id is required
    if asset.token_id.is_none() {
        return Err(ContractError::CustomError {
            val: "Token ID is required".to_string(),
        });
    }
    let token_id = asset.token_id.unwrap();

    // find listing
    let listing_key = listing_key(&contract_address, &token_id);
    let listing = LISTINGS.load(deps.storage, listing_key.clone())?;

    // if a listing is not expired, only seller can cancel
    if (!listing.is_expired(&env.block)) && (listing.owner != info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    // we will remove the cancelled listing
    LISTINGS.remove(deps.storage, listing_key)?;

    Ok(Response::new()
        .add_attribute("method", "cancel")
        .add_attribute("contract_address", contract_address)
        .add_attribute("token_id", token_id)
        .add_attribute("cancelled_at", env.block.time.to_string()))
}

// function to add new offer nft using ordering style
// the 'offer' of offer_nft will contain the information of price
// the 'consideration' of offer_nft will contain the information of nft
pub fn execute_offer_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    asset: PaymentAsset,
    auction_config: AuctionConfig,
) -> Result<Response, ContractError> {
    // auction time must be valid first
    if auction_config.is_valid() {
        return Err(ContractError::CustomError {
            val: "Invalid auction config".to_string(),
        });
    }

    // ***********
    // OFFERING FUNDS
    // ***********
    // load config
    let config = CONFIG.load(deps.storage)?;

    // check ig the vaura_address is set (the default value is equal to "aura0")
    if config.vaura_address == Addr::unchecked("aura0") {
        return Err(ContractError::VauraAddressNotSet {});
    }

    let (token_address, amount) = match asset {
        PaymentAsset::Cw20 {
            contract_address,
            amount,
        } => (contract_address, amount),
        _ => Err(ContractError::CustomError {
            val: "Asset is not supported".to_string(),
        })?,
    };

    // the token address must be equal to the vaura_address
    if token_address != config.vaura_address {
        return Err(ContractError::InvalidTokenAddress {});
    }

    // check that the allowance of the cw20 offer token is enough
    let allowance_response: AllowanceResponse = deps
        .querier
        .query_wasm_smart(
            &token_address,
            &Cw20QueryMsg::Allowance {
                owner: info.sender.to_string(),
                spender: env.contract.address.to_string(),
            },
        )
        .unwrap();

    // check if the allowance is greater or equal the offer amount
    if allowance_response.allowance < Uint128::from(amount) {
        return Err(ContractError::InsufficientAllowance {});
    }

    let (contract_address, token_id, start_time, end_time) = match auction_config {
        AuctionConfig::OfferPrice {
            price,
            start_time,
            end_time,
        } => (price.contract_address, price.token_id, start_time, end_time),
        _ => {
            return Err(ContractError::CustomError {
                val: "Auction Config Error".to_string(),
            });
        }
    };

    if token_id.is_none() {
        return Err(ContractError::CustomError {
            val: "Collection offer is not supported".to_string(),
        });
    }

    let token_id = token_id.unwrap();

    let owner_response: StdResult<cw721::OwnerOfResponse> = deps.querier.query_wasm_smart(
        &contract_address,
        &Cw721QueryMsg::OwnerOf {
            token_id: token_id.clone(),
            include_expired: Some(false),
        },
    );

    match owner_response {
        Ok(owner) => {
            if owner.owner == info.sender {
                return Err(ContractError::CustomError {
                    val: ("Cannot offer owned nft".to_string()),
                });
            }
        }
        Err(_) => {
            return Err(ContractError::CustomError {
                val: ("Nft not exist".to_string()),
            });
        }
    }

    // the order id will contain the address and id of NFT
    let order_id = order_id(&info.sender, &contract_address.clone(), &token_id);

    // the offer item will contain the infomation of cw20 token
    let offer_item = offer_item(
        &ItemType::CW20,
        &Asset::from(PaymentAsset::Cw20 {
            contract_address: token_address,
            amount,
        }),
        &amount,
        &amount,
    );

    // the consideration item will contain the infomation of nft
    let consideration_item = consideration_item(
        &ItemType::CW721,
        &Asset::Nft(NftAsset {
            contract_address: contract_address.clone(),
            token_id: Some(token_id.clone()),
        }),
        &1u128,
        &1u128,
        &info.sender,
    );

    // generate new offer
    let new_offer = Order {
        order_type: OrderType::OFFER,
        order_id,
        owner: info.sender.clone(),
        offer: vec![offer_item],
        consideration: vec![consideration_item],
        start_time,
        end_time,
    };

    // generate offer key for this offer based on user address, contract address and token id
    let offer_key = offer_key(&info.sender, &contract_address, &token_id);
    // we will override the order if it already exists
    OFFERS.update(
        deps.storage,
        offer_key,
        |_old| -> Result<Order, ContractError> { Ok(new_offer.clone()) },
    )?;

    let offer_str = serde_json::to_string(&new_offer.offer);
    let consideration_str = serde_json::to_string(&new_offer.consideration);

    // return success
    Ok(Response::new()
        .add_attribute("method", "create_offer")
        .add_attribute("order_type", "OFFER")
        .add_attribute("offerer", new_offer.owner)
        .add_attribute("offer", offer_str.unwrap())
        .add_attribute("consideration", consideration_str.unwrap())
        .add_attribute("end_time", new_offer.end_time.unwrap().to_string()))
}

// function to accept offer nft using ordering style
pub fn execute_accept_nft_offer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    offerer: Addr,
    nft: NftAsset,
    funds_amount: u128,
) -> Result<Response, ContractError> {
    let contract_address = nft.contract_address;
    let token_id = nft.token_id;

    // cannot accept own offer
    if info.sender == offerer {
        return Err(ContractError::CustomError {
            val: ("Cannot accept own offer".to_string()),
        });
    }

    // if the token_id is exist, then this order is offer for a specific nft
    if let Some(token_id) = token_id {
        // generate order key for order components based on user address, contract address and token id
        let offer_key = offer_key(&offerer, &contract_address, &token_id);

        // get offer information
        let offer = OFFERS.load(deps.storage, offer_key.clone())?;

        // if the end time of the offer is expired, then return error
        if offer.is_expired(&env.block) {
            return Err(ContractError::CustomError {
                val: ("Offer is expired".to_string()),
            });
        }

        match &offer.consideration[0].item {
            // match if the consideration item is Nft
            Asset::Nft(NftAsset {
                contract_address,
                token_id,
            }) => {
                // query the owner of the nft
                let owner: cw721::OwnerOfResponse = deps
                    .querier
                    .query_wasm_smart(
                        contract_address,
                        &Cw721QueryMsg::OwnerOf {
                            token_id: token_id.clone().unwrap(),
                            include_expired: Some(false),
                        },
                    )
                    .unwrap();

                // if the nft is not belong to the info.sender, then return error
                if owner.owner != info.sender {
                    return Err(ContractError::Unauthorized {});
                }

                let mut res: Response = Response::new();

                // ***********************
                // TRANSFER CW20 TO SENDER
                // ***********************
                // convert Asset to PaymentAsset
                let payment_item = PaymentAsset::from(offer.offer[0].item.clone());

                // execute cw20 transfer msg from offerer to info.sender
                match &payment_item {
                    PaymentAsset::Cw20 {
                        contract_address: _,
                        amount,
                    } => {
                        if funds_amount != *amount {
                            return Err(ContractError::CustomError {
                                val: ("Insufficient funds".to_string()),
                            });
                        }
                        let payment_messages = payment_with_royalty(
                            &deps,
                            contract_address,
                            token_id.as_ref().unwrap(),
                            &payment_item,
                            &offerer,
                            &info.sender,
                        );

                        // loop through all payment messages and add item to response to execute
                        for payment_message in payment_messages {
                            res = res.add_message(payment_message);
                        }
                    }
                    _ => {
                        return Err(ContractError::CustomError {
                            val: ("Invalid Offer funding type".to_string()),
                        });
                    }
                }

                // ***********************
                // TRANSFER NFT TO OFFERER
                // ***********************
                // message to transfer nft to offerer
                let transfer_nft_msg = WasmMsg::Execute {
                    contract_addr: contract_address.clone().to_string(),
                    msg: to_json_binary(&Cw721ExecuteMsg::TransferNft {
                        recipient: offer.owner.clone().to_string(),
                        token_id: token_id.clone().unwrap(),
                    })?,
                    funds: vec![],
                };

                // add transfer nft message to response to execute
                res = res.add_message(transfer_nft_msg);

                // After the offer is accepted, we will delete the order
                OFFERS.remove(deps.storage, offer_key)?;

                let listing_key = listing_key(contract_address, &token_id.clone().unwrap());
                LISTINGS.remove(deps.storage, listing_key)?;

                Ok(res
                    .add_attribute("method", "execute_accept_nft_offer")
                    .add_attribute("owner", owner.owner)
                    .add_attribute("offerer", offer.owner)
                    .add_attribute("nft_contract_address", contract_address.to_string())
                    .add_attribute("token_id", token_id.clone().unwrap()))
            }
            // if the consideration item is not Nft, then return error
            _ => Err(ContractError::CustomError {
                val: ("Consideration is not NFT".to_string()),
            }),
        }
    } else {
        Err(ContractError::CustomError {
            val: ("Collection offer is not supported".to_string()),
        })
    }
}

pub fn execute_cancel_offer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    nfts: Vec<NftAsset>,
) -> Result<Response, ContractError> {
    // if the number of nfts is greater than 50, then return error
    if nfts.len() > 50 {
        return Err(ContractError::CustomError {
            val: ("Number of NFTs is greater than 50".to_string()),
        });
    }

    // loop through all nfts
    for nft in nfts {
        // generate order key based on the sender address, nft.contract_address and nft.token_id
        let offer_key = offer_key(&info.sender, &nft.contract_address, &nft.token_id.unwrap());

        // check if the order exists
        if !OFFERS.has(deps.storage, offer_key.clone()) {
            return Err(ContractError::CustomError {
                val: ("Offer does not exist".to_string()),
            });
        }

        // we will remove the cancelled offer
        OFFERS.remove(deps.storage, offer_key)?;
    }

    Ok(Response::new()
        .add_attribute("method", "cancel_all_offer")
        .add_attribute("user", info.sender.to_string())
        .add_attribute("cancelled_at", env.block.time.to_string()))
}

// function to process payment transfer with royalty
fn payment_with_royalty(
    deps: &DepsMut,
    nft_contract_address: &Addr,
    nft_id: &str,
    asset: &PaymentAsset,
    sender: &Addr,
    recipient: &Addr,
) -> Vec<CosmosMsg> {
    // create empty vector of CosmosMsg
    let mut res_messages: Vec<CosmosMsg> = vec![];

    // Extract information from token
    let (is_native, token_info, amount) = match asset {
        PaymentAsset::Cw20 {
            contract_address,
            amount,
        } => (
            false,
            (*contract_address).to_string().clone(),
            Uint128::from(*amount),
        ),
        PaymentAsset::Native { denom, amount } => (true, (*denom).clone(), Uint128::from(*amount)),
    };

    // get cw2981 royalties info
    let royalty_query_msg = Cw2981QueryMsg::Extension {
        msg: cw2981_royalties::msg::Cw2981QueryMsg::RoyaltyInfo {
            token_id: nft_id.into(),
            sale_price: amount,
        },
    };

    let royalty_info_rsp: Result<RoyaltiesInfoResponse, cosmwasm_std::StdError> =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: nft_contract_address.to_string(),
            msg: to_json_binary(&royalty_query_msg).unwrap(),
        }));

    let (creator, royalty_amount): (Option<Addr>, Option<Uint128>) = match royalty_info_rsp {
        Ok(RoyaltiesInfoResponse {
            address,
            royalty_amount,
        }) => {
            if address.is_empty() || royalty_amount == Uint128::zero() {
                (None, None)
            } else {
                (
                    Some(deps.api.addr_validate(&address).unwrap()),
                    Some(royalty_amount),
                )
            }
        }
        Err(_) => (None, None),
    };

    // there is no royalty, creator is the recipient, or royalty amount is 0
    if creator.is_none()
        || *creator.as_ref().unwrap() == *recipient
        || royalty_amount.is_none()
        || royalty_amount.unwrap().is_zero()
    {
        match &is_native {
            false => {
                // execute cw20 transfer msg from info.sender to recipient
                let transfer_response = WasmMsg::Execute {
                    contract_addr: deps.api.addr_validate(&token_info).unwrap().to_string(),
                    msg: to_json_binary(&Cw20ExecuteMsg::TransferFrom {
                        owner: sender.to_string(),
                        recipient: recipient.to_string(),
                        amount,
                    })
                    .unwrap(),
                    funds: vec![],
                };
                res_messages.push(transfer_response.into());
            }
            true => {
                // transfer all funds to recipient
                let transfer_response = BankMsg::Send {
                    to_address: recipient.to_string(),
                    amount: vec![Coin {
                        denom: token_info.to_string(),
                        amount,
                    }],
                };
                res_messages.push(transfer_response.into());
            }
        }
    } else if let (Some(creator), Some(royalty_amount)) = (creator, royalty_amount) {
        match &is_native {
            false => {
                // execute cw20 transfer transfer royalty to creator
                let transfer_token_creator_response = WasmMsg::Execute {
                    contract_addr: deps.api.addr_validate(&token_info).unwrap().to_string(),
                    msg: to_json_binary(&Cw20ExecuteMsg::TransferFrom {
                        owner: sender.to_string(),
                        recipient: creator.to_string(),
                        amount: royalty_amount,
                    })
                    .unwrap(),
                    funds: vec![],
                };
                res_messages.push(transfer_token_creator_response.into());

                // execute cw20 transfer remaining funds to recipient
                let transfer_token_seller_msg = WasmMsg::Execute {
                    contract_addr: deps.api.addr_validate(&token_info).unwrap().to_string(),
                    msg: to_json_binary(&Cw20ExecuteMsg::TransferFrom {
                        owner: sender.to_string(),
                        recipient: recipient.to_string(),
                        amount: amount - royalty_amount,
                    })
                    .unwrap(),
                    funds: vec![],
                };
                res_messages.push(transfer_token_seller_msg.into());
            }
            true => {
                // transfer royalty to creator
                let transfer_token_creator_response = BankMsg::Send {
                    to_address: creator.to_string(),
                    amount: vec![Coin {
                        denom: token_info.to_string(),
                        amount: royalty_amount,
                    }],
                };
                res_messages.push(transfer_token_creator_response.into());

                // transfer remaining funds to recipient
                let transfer_token_seller_msg = BankMsg::Send {
                    to_address: recipient.to_string(),
                    amount: vec![Coin {
                        denom: token_info.to_string(),
                        amount: amount - royalty_amount,
                    }],
                };
                res_messages.push(transfer_token_seller_msg.into());
            }
        }
    }

    res_messages
}
