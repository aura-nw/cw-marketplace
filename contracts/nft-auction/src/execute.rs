use crate::state::{
    consideration_item, contract, offer_item, order_key, Asset, AuctionConfigInput,
    DutchAuctionConfig, EnglishAuctionConfig, OrderComponents, PaymentAsset, NATIVE, NFT,
};
use crate::ContractError;
use cosmwasm_std::{
    coin, has_coins, to_binary, Addr, BankMsg, Coin, CosmosMsg, Decimal, DepsMut, Env, MessageInfo,
    QueryRequest, Response, StdResult, Uint128, WasmMsg, WasmQuery,
};
use cw20::Cw20ExecuteMsg;
use cw2981_royalties::{
    msg::RoyaltiesInfoResponse, ExecuteMsg as Cw2981ExecuteMsg, QueryMsg as Cw2981QueryMsg,
};
use cw721::{Cw721QueryMsg, Expiration as Cw721Expiration};

// function to process payment transfer with royalty
fn payment_with_royalty(
    deps: &DepsMut,
    nft_contract_address: &Addr,
    nft_id: &str,
    token: PaymentAsset,
    sender: &Addr,
    recipient: &Addr,
) -> Vec<CosmosMsg> {
    // create empty vector of CosmosMsg
    let mut res_messages: Vec<CosmosMsg> = vec![];

    // Extract information from token
    let (is_native, token_info, amount) = match token {
        PaymentAsset::Cw20 {
            contract_address: token_address,
            amount,
        } => (false, token_address.to_string(), Uint128::from(amount)),
        PaymentAsset::Native { denom, amount } => (true, denom, Uint128::from(amount)),
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
            msg: to_binary(&royalty_query_msg).unwrap(),
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
                    msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
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
                        denom: token_info,
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
                    msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
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
                    msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
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
                        denom: token_info.clone(),
                        amount: royalty_amount,
                    }],
                };
                res_messages.push(transfer_token_creator_response.into());

                // transfer remaining funds to recipient
                let transfer_token_seller_msg = BankMsg::Send {
                    to_address: recipient.to_string(),
                    amount: vec![Coin {
                        denom: token_info,
                        amount: amount - royalty_amount,
                    }],
                };
                res_messages.push(transfer_token_seller_msg.into());
            }
        }
    }

    res_messages
}

pub fn execute_auction_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    nft: NFT,
    auction_config: AuctionConfigInput,
) -> Result<Response, ContractError> {
    // if the AuctionConfig is match with EnglishAuction
    match auction_config {
        AuctionConfigInput::EnglishAuction {
            start_price,
            step_percentage,
            buyout_price,
            start_time,
            end_time,
        } => {
            // if the start_time is not set, then set it to the current time + 1s
            let start_time = start_time
                .unwrap_or_else(|| Cw721Expiration::AtTime(env.block.time.plus_seconds(1)));
            // check if the start_time is greater than the current time
            if start_time.is_expired(&env.block)
                || end_time.eq(&Cw721Expiration::Never {})
                || start_time >= end_time
            {
                return Err(ContractError::CustomError {
                    val: ("Time config invalid".to_string()),
                });
            }

            // match the token_id of nft
            match nft.token_id {
                Some(token_id) => {
                    // check if user is the owner of the token
                    let query_owner_msg = Cw721QueryMsg::OwnerOf {
                        token_id: token_id.clone(),
                        include_expired: Some(false),
                    };
                    let owner_response: StdResult<cw721::OwnerOfResponse> =
                        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                            contract_addr: nft.contract_address.to_string(),
                            msg: to_binary(&query_owner_msg)?,
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

                    let mut res = Response::new();
                    // transfer nft to contract
                    let transfer_nft_msg = WasmMsg::Execute {
                        contract_addr: nft.contract_address.to_string(),
                        msg: to_binary(&Cw2981ExecuteMsg::TransferNft {
                            recipient: env.contract.address.to_string(),
                            token_id: token_id.clone(),
                        })?,
                        funds: vec![],
                    };
                    res = res.add_message(transfer_nft_msg);

                    // create offer item based on the nft
                    let offer_item = offer_item(
                        &Asset::Nft(NFT {
                            contract_address: nft.contract_address.clone(),
                            token_id: Some(token_id.clone()),
                        }),
                        &1u128,
                        &1u128,
                        &info.sender,
                    );

                    // create consideration item based on the auction config
                    let consideration_item = consideration_item(
                        &Asset::Native(NATIVE {
                            denom: start_price.denom.clone(),
                            amount: start_price.amount.into(),
                        }),
                        &start_price.amount.into(),
                        &buyout_price.unwrap_or(0),
                        &info.sender, // the recipient is the offerer by default
                    );

                    // create order key based on the marketplace address, nft.contract_address and nft.token_id
                    let order_key =
                        order_key(&env.contract.address, &nft.contract_address, &token_id);

                    let order_config = EnglishAuctionConfig {
                        order_type: "english_auction".to_string(),
                        step_percentage: step_percentage.unwrap_or(5u64),
                    };

                    // create order
                    let order = OrderComponents {
                        order_id: order_key.clone(),
                        offer: vec![offer_item],
                        consideration: vec![consideration_item],
                        start_time,
                        end_time,
                        config: order_config.to_string(),
                    };

                    // store order
                    contract().auctions.save(deps.storage, order_key, &order)?;

                    Ok(res.add_attributes([
                        ("method", "auction_nft"),
                        ("seller", info.sender.as_str()),
                        ("contract_address", nft.contract_address.as_str()),
                        ("token_id", token_id.as_str()),
                        ("start_price", start_price.amount.to_string().as_str()),
                        ("denom", start_price.denom.as_str()),
                        (
                            "step_percentage",
                            step_percentage.unwrap_or(0).to_string().as_str(),
                        ),
                        (
                            "buyout_price",
                            buyout_price.unwrap_or(0).to_string().as_str(),
                        ),
                        ("start_time", start_time.to_string().as_str()),
                        ("end_time", end_time.to_string().as_str()),
                    ]))
                }
                None => Err(ContractError::CustomError {
                    val: ("Token id is required".to_string()),
                }),
            }
        }
        AuctionConfigInput::DutchAuction {
            start_price,
            end_price,
            start_time,
            end_time,
        } => {
            // if the start_time is not set, then set it to the current time + 1s
            let start_time = start_time
                .unwrap_or_else(|| Cw721Expiration::AtTime(env.block.time.plus_seconds(1)));

            // validate time config
            if start_time.is_expired(&env.block)
                || end_time.eq(&Cw721Expiration::Never {})
                || start_time >= end_time
            {
                return Err(ContractError::CustomError {
                    val: ("Time config invalid".to_string()),
                });
            }

            // match the token_id of nft
            match nft.token_id {
                Some(token_id) => {
                    // check if user is the owner of the token
                    let query_owner_msg = Cw721QueryMsg::OwnerOf {
                        token_id: token_id.clone(),
                        include_expired: Some(false),
                    };
                    let owner_response: StdResult<cw721::OwnerOfResponse> =
                        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                            contract_addr: nft.contract_address.to_string(),
                            msg: to_binary(&query_owner_msg)?,
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

                    let mut res = Response::new();
                    // transfer nft to contract
                    let transfer_nft_msg = WasmMsg::Execute {
                        contract_addr: nft.contract_address.to_string(),
                        msg: to_binary(&Cw2981ExecuteMsg::TransferNft {
                            recipient: env.contract.address.to_string(),
                            token_id: token_id.clone(),
                        })?,
                        funds: vec![],
                    };
                    res = res.add_message(transfer_nft_msg);

                    // create offer item based on the nft
                    let offer_item = offer_item(
                        &Asset::Nft(NFT {
                            contract_address: nft.contract_address.clone(),
                            token_id: Some(token_id.clone()),
                        }),
                        &1u128,
                        &1u128,
                        &info.sender,
                    );

                    // create consideration item based on the auction config
                    let consideration_item = consideration_item(
                        &Asset::Native(NATIVE {
                            denom: start_price.denom.clone(),
                            amount: start_price.amount.into(),
                        }),
                        &start_price.amount.into(),
                        &end_price,
                        &info.sender, // the recipient is the offerer by default
                    );

                    // create order key based on the marketplace address, nft.contract_address and nft.token_id
                    let order_key =
                        order_key(&env.contract.address, &nft.contract_address, &token_id);

                    let order_config = DutchAuctionConfig {
                        order_type: "dutch_auction".to_string(),
                    };

                    // create order
                    let order = OrderComponents {
                        order_id: order_key.clone(),
                        offer: vec![offer_item],
                        consideration: vec![consideration_item],
                        start_time,
                        end_time,
                        config: order_config.to_string(),
                    };

                    // store order
                    contract().auctions.save(deps.storage, order_key, &order)?;

                    Ok(res.add_attributes([
                        ("method", "auction_nft"),
                        ("seller", info.sender.as_str()),
                        ("contract_address", nft.contract_address.as_str()),
                        ("token_id", token_id.as_str()),
                        ("start_price", start_price.amount.to_string().as_str()),
                        ("denom", start_price.denom.as_str()),
                        ("end_price", end_price.to_string().as_str()),
                        ("start_time", start_time.to_string().as_str()),
                        ("end_time", end_time.to_string().as_str()),
                    ]))
                }
                None => Err(ContractError::CustomError {
                    val: ("Token id is required".to_string()),
                }),
            }
        }
    }
}

pub fn execute_bid_auction(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    nft: NFT,
    bid_price: u128,
) -> Result<Response, ContractError> {
    // nft.token_id must be exist
    if nft.token_id.is_none() {
        return Err(ContractError::CustomError {
            val: ("Token id is required".to_string()),
        });
    }

    // create order key based on the offerer address, nft.contract_address and nft.token_id
    let order_key = order_key(
        &env.contract.address,
        &nft.contract_address,
        &nft.token_id.clone().unwrap(),
    );

    // get order
    let order = contract().auctions.load(deps.storage, order_key.clone())?;

    // get config of order
    let order_config = EnglishAuctionConfig::from(order.config.clone());

    // if the type of order is not english_auction, return error
    if order_config.order_type != "english_auction" {
        return Err(ContractError::CustomError {
            val: ("Invalid auction type".to_string()),
        });
    }

    // the sender must be different than the offerer
    if info.sender == order.offer[0].offerer {
        return Err(ContractError::CustomError {
            val: ("Cannot bid on your own auction".to_string()),
        });
    }

    // check if the order is expired
    if order.is_expired(&env.block) {
        return Err(ContractError::CustomError {
            val: ("Auction is expired".to_string()),
        });
    }

    // match the item of consideration of the order
    match &order.consideration[0].item {
        Asset::Native(current_price) => {
            // check if the amount in funds is equal to the bid_price
            if !has_coins(
                &info.funds,
                &Coin {
                    denom: current_price.denom.clone(),
                    amount: bid_price.into(),
                },
            ) {
                return Err(ContractError::CustomError {
                    val: ("Different funding amount and bidding price".to_string()),
                });
            }

            let mut res = Response::new();
            // TODO: if the bidding price is greater than the buyout price, terminate the auction and transfer the nft to the bidder

            // if the recipient's different than offerer (the first bidder),
            // the bid_price must be greater than the current_price + step_price
            // and we must return the previous bid_price to the previous bidder
            let previous_bidder = order.consideration[0].recipient.clone();
            if previous_bidder != order.offer[0].offerer {
                // parse the step_percentage from order.config
                let step_percentage = order_config.step_percentage;

                // check if the bid_price is greater than the current_price + step_price
                let step_price =
                    Uint128::from(current_price.amount) * Decimal::percent(step_percentage);
                if bid_price < current_price.amount.checked_add(step_price.into()).unwrap() {
                    return Err(ContractError::CustomError {
                        val: ("Bidding price invalid".to_string()),
                    });
                }

                // transfer the previous bid_price to the previous bidder
                let bank_transfer = BankMsg::Send {
                    to_address: previous_bidder.to_string(),
                    amount: vec![coin(current_price.amount, current_price.denom.clone())],
                };
                res = res.add_message(bank_transfer);
            } else {
                // if the recipient is the offerer (the first bidder),
                // the bid_price must be greater than or equal the current_price
                if bid_price < current_price.amount {
                    return Err(ContractError::CustomError {
                        val: ("Bidding price invalid".to_string()),
                    });
                }
            }

            // update order information
            let mut new_order = contract().auctions.load(deps.storage, order_key.clone())?;
            // the recipient is the bidder
            new_order.consideration[0].recipient = info.sender.clone();

            // consideration item
            new_order.consideration[0].item = Asset::Native(NATIVE {
                denom: current_price.denom.clone(),
                amount: bid_price,
            });

            // if the remaining time is less than 10 minutes, extend the end_time by 10 minutes
            if new_order
                .end_time
                .le(&Cw721Expiration::AtTime(env.block.time.plus_seconds(600)))
            {
                new_order.end_time = Cw721Expiration::AtTime(env.block.time.plus_seconds(600));
            }

            // save order
            contract()
                .auctions
                .save(deps.storage, order_key, &new_order)?;

            Ok(res.add_attributes([
                ("method", "bid_nft"),
                ("buyer", info.sender.as_str()),
                ("contract_address", nft.contract_address.as_str()),
                ("token_id", &nft.token_id.unwrap()),
                ("bid_price", bid_price.to_string().as_str()),
            ]))
        }
        _ => Err(ContractError::CustomError {
            val: ("Invalid consideration item".to_string()),
        }),
    }
}

pub fn execute_settle_auction(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    nft: NFT,
) -> Result<Response, ContractError> {
    // nft.token_id must be exist
    if nft.token_id.is_none() {
        return Err(ContractError::CustomError {
            val: ("Token id is required".to_string()),
        });
    }

    // create order key based on the offerer address, nft.contract_address and nft.token_id
    let order_key = order_key(
        &env.contract.address,
        &nft.contract_address,
        &nft.token_id.clone().unwrap(),
    );

    // get order
    let order = contract().auctions.load(deps.storage, order_key.clone())?;

    // only the offerer or recipient can terminate the auction
    if info.sender != order.offer[0].offerer
        && info.sender != order.consideration[0].recipient.clone()
    {
        return Err(ContractError::Unauthorized {});
    }

    // check if the order is not expired
    if !order.is_expired(&env.block) {
        return Err(ContractError::CustomError {
            val: ("Auction is not expired".to_string()),
        });
    }

    let mut res = Response::new();

    // transfer the nft to the recipient
    let transfer_nft_msg = WasmMsg::Execute {
        contract_addr: nft.contract_address.to_string(),
        msg: to_binary(&Cw2981ExecuteMsg::TransferNft {
            recipient: order.consideration[0].recipient.to_string(),
            token_id: nft.token_id.clone().unwrap(),
        })?,
        funds: vec![],
    };
    res = res.add_message(transfer_nft_msg);

    // if the auction has no bid, stop the function here
    if order.consideration[0].recipient == order.offer[0].offerer {
        // delete order
        contract().auctions.remove(deps.storage, order_key)?;

        return Ok(res.add_attributes([
            ("method", "settle_auction"),
            ("seller", order.offer[0].offerer.as_str()),
            ("buyer", order.consideration[0].recipient.as_str()),
            ("contract_address", nft.contract_address.as_str()),
            ("token_id", &nft.token_id.unwrap()),
            ("status", "failure"),
        ]));
    }

    // send the native token to the offerer

    let payment = PaymentAsset::from(order.consideration[0].item.clone());

    let payment_messages = payment_with_royalty(
        &deps,
        &nft.contract_address,
        nft.token_id.as_ref().unwrap(),
        payment,
        &env.contract.address,
        &order.offer[0].offerer,
    );

    // add messages to response to execute
    res = res.add_messages(payment_messages);

    // delete order
    contract().auctions.remove(deps.storage, order_key)?;

    Ok(res.add_attributes([
        ("method", "settle_auction"),
        ("seller", order.offer[0].offerer.as_str()),
        ("buyer", order.consideration[0].recipient.as_str()),
        ("contract_address", nft.contract_address.as_str()),
        ("token_id", nft.token_id.unwrap().as_str()),
        ("status", "success"),
    ]))
}
