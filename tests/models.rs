//! Decoding tests for every response model, pinned to the documented payloads.

use aster_client::model::*;
use rust_decimal::Decimal;
use serde_json::json;

fn dec<T: serde::de::DeserializeOwned>(value: serde_json::Value) -> T {
    serde_json::from_value(value).unwrap()
}

#[test]
fn market_models() {
    let _: ServerTime = dec(json!({"serverTime": 1}));
    let _: Empty = dec(json!({}));
    assert!(dec::<Ack>(json!({"code": 200, "msg": "success"})).is_success());
    assert!(dec::<Ack>(json!({"code": "200"})).is_success());

    let one: OneOrMany<PriceTicker> = dec(json!({"symbol": "BTCUSDT", "price": "1", "time": 2}));
    assert_eq!(one.as_slice().len(), 1);
    let many: OneOrMany<PriceTicker> = dec(json!([{"symbol": "BTCUSDT", "price": "1"}]));
    assert_eq!(many.clone().into_vec().len(), 1);
    assert_eq!(many.as_slice()[0].price, Decimal::ONE);

    let agg: AggTrade = dec(json!({
        "a": 1, "p": "1.5", "q": "2", "f": 1, "l": 2, "T": 3, "m": true
    }));
    assert_eq!(agg.last_trade_id, 2);
    assert!(agg.buyer_is_maker);

    let kline: Kline = dec(json!([
        1, "1", "2", "0.5", "1.5", "10", 2, "20", 3, "5", "10", "ignore"
    ]));
    assert_eq!(kline.trades, 3);
    assert_eq!(kline.high, Decimal::from(2));

    let depth: Depth = dec(json!({
        "lastUpdateId": 1, "E": 2, "T": 3,
        "bids": [["1", "2"]], "asks": [["3", "4"]]
    }));
    assert_eq!(depth.bids[0].1, Decimal::from(2));
    assert_eq!(depth.event_time, Some(2));

    let trade: Trade = dec(json!({
        "id": 1, "price": "1", "qty": "2", "quoteQty": "2", "time": 3, "isBuyerMaker": true
    }));
    assert_eq!(trade.quote_qty, Some(Decimal::from(2)));

    let ticker: Ticker24h = dec(json!({
        "symbol": "BTCUSDT", "priceChange": "1", "priceChangePercent": "2",
        "weightedAvgPrice": "1", "prevClosePrice": "1", "lastPrice": "10", "lastQty": "1",
        "openPrice": "9", "highPrice": "11", "lowPrice": "8", "volume": "100",
        "quoteVolume": "1000", "openTime": 1, "closeTime": 2, "firstId": 3, "lastId": 4, "count": 5
    }));
    assert_eq!(ticker.count, Some(5));

    let price: PriceTicker = dec(json!({"symbol": "BTCUSDT", "price": "10", "time": 1}));
    assert_eq!(price.price, Decimal::from(10));
    let book: BookTicker = dec(json!({
        "symbol": "BTCUSDT", "bidPrice": "1", "bidQty": "2", "askPrice": "3", "askQty": "4", "time": 5
    }));
    assert_eq!(book.ask_qty, Decimal::from(4));

    let premium: PremiumIndex = dec(json!({
        "symbol": "BTCUSDT", "markPrice": "1", "indexPrice": "1", "estimatedSettlePrice": "1",
        "lastFundingRate": "0.0001", "nextFundingTime": 1, "interestRate": "0.0001", "time": 2
    }));
    assert_eq!(premium.mark_price, Decimal::ONE);

    let funding: FundingRateRecord =
        dec(json!({"symbol": "BTCUSDT", "fundingRate": "0.1", "fundingTime": 1}));
    assert_eq!(funding.funding_rate, Decimal::new(1, 1));
    let info: FundingInfo = dec(json!({
        "symbol": "BTCUSDT", "interestRate": "0.0001", "time": 1,
        "fundingIntervalHours": 8, "fundingFeeCap": 0.03, "fundingFeeFloor": -0.03
    }));
    assert_eq!(info.funding_interval_hours, 8);

    let refs: IndexReferences = dec(json!({
        "symbol": "BTCUSDT", "time": 1,
        "references": [{"exchange": "binance", "symbol": "BTCUSDT", "weight": "0.5"}]
    }));
    assert_eq!(refs.references[0].exchange, "binance");
    let remaining: RemainingOpenableNotional = dec(json!({"remainingOpenableNotionalValue": "-1"}));
    assert_eq!(
        remaining.remaining_openable_notional_value,
        Decimal::NEGATIVE_ONE
    );

    // Enumerations exposed by exchangeInfo.
    for (wire, want) in [
        ("REQUEST_WEIGHT", RateLimitType::RequestWeight),
        ("ORDERS", RateLimitType::Orders),
        ("RAW_REQUEST", RateLimitType::RawRequest),
        ("OTHER", RateLimitType::Unknown),
    ] {
        let limit: RateLimit = dec(json!({
            "rateLimitType": wire, "interval": "MINUTE", "intervalNum": 1, "limit": 2400
        }));
        assert_eq!(limit.rate_limit_type, want);
    }

    for (wire, want) in [
        ("PENDING_TRADING", SymbolStatus::PendingTrading),
        ("TRADING", SymbolStatus::Trading),
        ("PRE_SETTLE", SymbolStatus::PreSettle),
        ("SETTLING", SymbolStatus::Settling),
        ("CLOSE", SymbolStatus::Close),
        ("UNKNOWN", SymbolStatus::Unknown),
    ] {
        let symbol: SymbolInfo = dec(json!({
            "symbol": "BTCUSDT", "status": wire, "baseAsset": "BTC", "quoteAsset": "USDT",
            "filters": [], "orderTypes": [], "timeInForce": []
        }));
        assert_eq!(symbol.status, want);
    }

    let asset: ExchangeAsset = dec(json!({
        "asset": "USDT", "marginAvailable": true, "autoAssetExchange": 0
    }));
    assert_eq!(asset.margin_available, Some(true));

    let filters: Vec<Filter> = serde_json::from_value(json!([
        {"filterType": "PRICE_FILTER", "tickSize": "0.1", "minPrice": "1", "maxPrice": "10"},
        {"filterType": "LOT_SIZE", "stepSize": "0.001", "minQty": "1", "maxQty": "10"},
        {"filterType": "MARKET_LOT_SIZE", "stepSize": "0.001"},
        {"filterType": "MAX_NUM_ORDERS", "limit": 200},
        {"filterType": "MAX_NUM_ALGO_ORDERS", "limit": 100},
        {"filterType": "MIN_NOTIONAL", "notional": "5"},
        {"filterType": "MAX_NOTIONAL", "maxNotional": "100"},
        {"filterType": "NOTIONAL", "minNotional": "5", "maxNotional": "10", "avgPriceMins": 5,
         "applyMinToMarket": true, "applyMaxToMarket": false},
        {"filterType": "PERCENT_PRICE", "multiplierUp": "5", "multiplierDown": "0.5",
         "multiplierDecimal": 4, "ltMultiplierUp": "1", "ltMultiplierDown": "1"},
        {"filterType": "PERCENT_PRICE_BY_SIDE", "bidMultiplierUp": "5", "askMultiplierUp": "5",
         "bidMultiplierDown": "0.5", "askMultiplierDown": "0.5", "avgPriceMins": 5,
         "multiplierDecimal": 4},
        {"filterType": "BRAND_NEW", "foo": 1}
    ]))
    .unwrap();
    assert_eq!(filters[0].kind(), FilterKind::PriceFilter);
    assert_eq!(filters[10].kind(), FilterKind::Unknown);

    let symbol: SymbolInfo = dec(json!({
        "symbol": "BTCUSDT", "status": "TRADING", "baseAsset": "BTC", "quoteAsset": "USDT",
        "filters": [
            {"filterType": "PRICE_FILTER", "tickSize": "0.10"},
            {"filterType": "LOT_SIZE", "stepSize": "0.001", "maxQty": "100"},
            {"filterType": "MARKET_LOT_SIZE", "stepSize": "0.002"},
            {"filterType": "MIN_NOTIONAL", "notional": "5"}
        ],
        "orderTypes": ["LIMIT", "MARKET", "STOP", "STOP_MARKET", "TAKE_PROFIT",
                       "TAKE_PROFIT_MARKET", "TRAILING_STOP_MARKET", "BRAND_NEW"],
        "timeInForce": ["GTC", "IOC", "FOK", "GTX", "HIDDEN", "NEW"]
    }));
    assert_eq!(symbol.tick_size(), Some(Decimal::new(1, 1)));
    assert_eq!(symbol.step_size(), Some(Decimal::new(1, 3)));
    assert_eq!(symbol.market_step_size(), Some(Decimal::new(2, 3)));
    assert_eq!(symbol.min_notional(), Some(Decimal::from(5)));
    assert_eq!(symbol.max_qty(), Some(Decimal::from(100)));
    assert_eq!(
        symbol.quantize_price(Decimal::new(12345, 2)),
        Some(Decimal::new(1234, 1))
    );
    assert_eq!(
        symbol.quantize_qty(Decimal::new(12345, 4)),
        Some(Decimal::new(1234, 3))
    );
    assert!(symbol.is_trading());
    assert!(symbol.filter(&FilterKind::LotSize).is_some());
    assert_eq!(
        quantize_down(Decimal::from(7), Decimal::ZERO),
        Decimal::from(7)
    );
    assert_eq!(symbol.order_types[0], OrderTypeName::Limit);
    assert_eq!(symbol.order_types[7], OrderTypeName::Other);
    assert_eq!(symbol.time_in_force[2], TimeInForceName::Fok);
    assert_eq!(symbol.time_in_force[5], TimeInForceName::Other);

    let futures: FuturesExchangeInfo = dec(json!({
        "futuresType": "U_MARGINED", "exchangeFilters": [], "rateLimits": [],
        "serverTime": 1, "assets": [], "symbols": [], "timezone": "UTC"
    }));
    assert!(futures.symbol("BTCUSDT").is_none());
    let spot: SpotExchangeInfo = dec(json!({
        "timezone": "UTC", "serverTime": 1, "rateLimits": [], "exchangeFilters": [],
        "assets": [], "symbols": []
    }));
    assert!(spot.symbol("NOPE").is_none());

    for interval in [
        KlineInterval::Min1,
        KlineInterval::Min3,
        KlineInterval::Min5,
        KlineInterval::Min15,
        KlineInterval::Min30,
        KlineInterval::Hour1,
        KlineInterval::Hour2,
        KlineInterval::Hour4,
        KlineInterval::Hour6,
        KlineInterval::Hour8,
        KlineInterval::Hour12,
        KlineInterval::Day1,
        KlineInterval::Day3,
        KlineInterval::Week1,
        KlineInterval::Month1,
    ] {
        let wire = serde_json::to_value(interval).unwrap();
        assert_eq!(wire, json!(interval.as_str()));
    }
}

#[test]
fn order_models() {
    let order: Order = dec(json!({
        "avgPrice": "1", "clientOrderId": "a", "cumQuote": "1", "executedQty": "0",
        "orderId": 1, "origQty": "1", "origType": "LIMIT", "price": "1", "reduceOnly": false,
        "side": "BUY", "positionSide": "BOTH", "status": "PARTIALLY_FILLED", "stopPrice": "0",
        "closePosition": false, "symbol": "BTCUSDT", "time": 1, "timeInForce": "GTC",
        "type": "LIMIT", "activatePrice": "1", "priceRate": "0.1", "workingType": "CONTRACT_PRICE",
        "priceProtect": true, "pegPriceType": "QUEUE_1", "stpMode": "EXPIRE_BOTH", "updateTime": 2
    }));
    assert!(order.status.is_open());
    assert!(!order.status.is_terminal());
    assert_eq!(order.order_type, OrderType::Limit);
    assert_eq!(order.peg_price_type.as_deref(), Some("QUEUE_1"));

    let spot: SpotOrder = dec(json!({
        "orderId": 1, "symbol": "BTCUSDT", "status": "FILLED", "clientOrderId": "s",
        "price": "1", "avgPrice": "1", "origQty": "1", "executedQty": "1", "cumQuote": "1",
        "timeInForce": "GTC", "type": "LIMIT", "stopPrice": "0", "origType": "LIMIT",
        "time": 1, "updateTime": 2, "orderListId": -1, "side": "SELL"
    }));
    assert_eq!(spot.status, OrderStatus::Filled);
    assert!(spot.status.is_terminal());

    let trade: FuturesTrade = dec(json!({
        "symbol": "BTCUSDT", "id": 1, "orderId": 2, "side": "BUY", "price": "1", "qty": "2",
        "realizedPnl": "0.1", "marginAsset": "USDT", "quoteQty": "2", "commission": "0.01",
        "commissionAsset": "USDT", "time": 3, "positionSide": "LONG", "buyer": true, "maker": false
    }));
    assert_eq!(trade.realized_pnl, Some(Decimal::new(1, 1)));
    let spot_trade: SpotTrade = dec(json!({
        "symbol": "BTCUSDT", "id": 1, "orderId": 2, "side": "SELL", "price": "1", "qty": "2",
        "quoteQty": "2", "commission": "0.01", "commissionAsset": "BNB", "time": 3,
        "isBuyer": false, "isMaker": true, "isBestMatch": true
    }));
    assert_eq!(spot_trade.commission_asset, "BNB");

    let batch: Vec<BatchItem<SpotOrder>> = serde_json::from_value(json!([
        {"orderId": 1, "symbol": "BTCUSDT", "status": "NEW", "clientOrderId": "s", "price": "1",
         "avgPrice": "1", "origQty": "1", "executedQty": "0", "type": "LIMIT"},
        {"code": -2011, "msg": "Unknown order sent."}
    ]))
    .unwrap();
    assert!(batch[0].clone().ok().is_some());
    assert_eq!(batch[1].clone().as_result().unwrap_err().code, -2011);

    let transfer: TransferResult = dec(json!({"tranId": 1, "status": "SUCCESS"}));
    assert_eq!(transfer.tran_id, 1);

    // Wire enums serialize to the documented strings.
    assert_eq!(serde_json::to_value(Side::Buy).unwrap(), json!("BUY"));
    assert_eq!(
        serde_json::to_value(OrderType::StopMarket).unwrap(),
        json!("STOP_MARKET")
    );
    assert_eq!(
        serde_json::to_value(TimeInForce::Gtx).unwrap(),
        json!("GTX")
    );
    assert_eq!(
        serde_json::to_value(PositionSide::Short).unwrap(),
        json!("SHORT")
    );
    assert_eq!(
        serde_json::to_value(WorkingType::MarkPrice).unwrap(),
        json!("MARK_PRICE")
    );
    assert_eq!(
        serde_json::to_value(NewOrderRespType::Ack).unwrap(),
        json!("ACK")
    );
    assert_eq!(
        serde_json::to_value(StpMode::ExpireMaker).unwrap(),
        json!("EXPIRE_MAKER")
    );
    assert_eq!(
        serde_json::to_value(PegPriceType::Counterparty20).unwrap(),
        json!("COUNTERPARTY_20")
    );
    assert_eq!(
        serde_json::to_value(MarginType::Crossed).unwrap(),
        json!("CROSSED")
    );
    assert_eq!(
        serde_json::to_value(TransferKind::FutureSpot).unwrap(),
        json!("FUTURE_SPOT")
    );
    assert_eq!(
        serde_json::to_value(StrategyType::Otoco).unwrap(),
        json!("OTOCO")
    );
    assert_eq!(
        serde_json::to_value(SecurityType::CoinFutures).unwrap(),
        json!("COIN_FUTURES")
    );
    assert!(valid_client_order_id("a.b:c/d_e-f"));
    assert!(!valid_client_order_id(""));
    assert!(!valid_client_order_id(&"x".repeat(37)));
    assert!(!valid_client_order_id("sp ace"));

    let countdown: CountdownCancelAllResult =
        dec(json!({"symbol": "BTCUSDT", "countdownTime": 1000}));
    assert_eq!(countdown.countdown_time, 1000);
}

#[test]
fn account_models() {
    let balance: Balance = dec(json!({
        "accountAlias": "x", "asset": "USDT", "balance": "10", "crossWalletBalance": "1",
        "crossUnPnl": "0.5", "availableBalance": "9", "maxWithdrawAmount": "8",
        "marginAvailable": true, "updateTime": 1
    }));
    assert_eq!(balance.balance, Decimal::from(10));

    let account: AccountInfo = dec(json!({
        "feeTier": 0, "canTrade": true, "canDeposit": true, "canWithdraw": false,
        "updateTime": 1, "totalInitialMargin": "1", "totalMaintMargin": "1",
        "totalWalletBalance": "1", "totalUnrealizedProfit": "0", "totalMarginBalance": "1",
        "totalPositionInitialMargin": "0", "totalOpenOrderInitialMargin": "0",
        "totalCrossWalletBalance": "1", "totalCrossUnPnl": "0", "availableBalance": "1",
        "maxWithdrawAmount": "1",
        "assets": [{
            "asset": "USDT", "walletBalance": "1", "unrealizedProfit": "0", "marginBalance": "1",
            "maintMargin": "0", "initialMargin": "0", "positionInitialMargin": "0",
            "openOrderInitialMargin": "0", "crossWalletBalance": "1", "crossUnPnl": "0",
            "availableBalance": "1", "maxWithdrawAmount": "1", "marginAvailable": true,
            "updateTime": 1
        }],
        "positions": [{
            "symbol": "BTCUSDT", "initialMargin": "0", "maintMargin": "0",
            "unrealizedProfit": "0", "positionInitialMargin": "0",
            "openOrderInitialMargin": "0", "leverage": "10", "isolated": false,
            "entryPrice": "1", "maxNotional": "100", "positionSide": "BOTH",
            "positionAmt": "1", "notional": "1", "isolatedWallet": "0", "updateTime": 1
        }]
    }));
    assert_eq!(account.assets[0].asset, "USDT");
    assert_eq!(account.positions[0].position_amt, Decimal::ONE);

    let risk: PositionRisk = dec(json!({
        "entryPrice": "1", "marginType": "isolated", "isAutoAddMargin": "false",
        "isolatedMargin": "1", "leverage": "10", "liquidationPrice": "0", "markPrice": "1",
        "maxNotionalValue": "100", "positionAmt": "-1", "symbol": "BTCUSDT",
        "unRealizedProfit": "-0.1", "positionSide": "SHORT", "notional": "-1",
        "isolatedWallet": "1", "updateTime": 1
    }));
    assert_eq!(risk.position_side, PositionSide::Short);

    let brackets: OneOrMany<LeverageBracket> = dec(json!([{
        "symbol": "BTCUSDT",
        "brackets": [{
            "bracket": 1, "initialLeverage": 75, "notionalCap": 10000, "notionalFloor": 0,
            "maintMarginRatio": 0.0065, "cum": 0
        }]
    }]));
    assert_eq!(brackets.as_slice()[0].brackets[0].initial_leverage, 75);

    let adl: AdlQuantile = dec(json!({
        "symbol": "BTCUSDT", "adlQuantile": {"LONG": 4, "SHORT": 2, "BOTH": 1}
    }));
    assert_eq!(adl.adl_quantile.long, Some(4));

    let force: ForceOrder = dec(json!({
        "orderId": 1, "symbol": "BTCUSDT", "status": "FILLED", "clientOrderId": "x",
        "price": "1", "avgPrice": "1", "cumQuote": "1", "executedQty": "1", "time": 1,
        "updateTime": 1, "origQty": "1", "type": "LIMIT", "side": "BUY",
        "positionSide": "BOTH", "timeInForce": "GTC", "origType": "LIMIT"
    }));
    assert_eq!(force.status, "FILLED");

    let commission: CommissionRate = dec(json!({
        "symbol": "BTCUSDT", "makerCommissionRate": "0.0002", "takerCommissionRate": "0.0004"
    }));
    assert_eq!(commission.taker_commission_rate, Decimal::new(4, 4));

    let income: Income = dec(json!({
        "symbol": "BTCUSDT", "incomeType": "FUNDING_FEE", "income": "-0.1", "asset": "USDT",
        "info": "", "time": 1, "tranId": 2, "tradeId": "3"
    }));
    assert_eq!(income.income, Decimal::new(-1, 1));
    let page: IncomePage = dec(json!({
        "total": 1, "page": 1, "limit": 100, "hasMore": false,
        "rows": [{"incomeType": "COMMISSION", "income": "-0.1", "asset": "USDT", "time": 1}]
    }));
    assert_eq!(page.rows.len(), 1);

    let margin: PositionMarginChange = dec(json!({
        "amount": "1", "asset": "USDT", "symbol": "BTCUSDT", "time": 1, "type": 1,
        "positionSide": "LONG", "deltaType": "TRADE", "clientTranId": ""
    }));
    assert_eq!(margin.change_type, 1);
    let modified: ModifyPositionMarginResult = dec(json!({
        "amount": 100.0, "code": 200, "msg": "ok", "type": 1
    }));
    assert_eq!(modified.amount, Decimal::from(100));

    let dual: DualSidePosition = dec(json!({"dualSidePosition": true}));
    assert!(dual.dual_side_position);
    let stp: StpModeResponse = dec(json!({"stpMode": "EXPIRE_TAKER"}));
    assert_eq!(stp.stp_mode, "EXPIRE_TAKER");
    let multi: MultiAssetsMargin = dec(json!({"multiAssetsMargin": false}));
    assert!(!multi.multi_assets_margin);
    let exchanged: AssetExchangeResult = dec(json!({
        "accountId": 1, "asset": "USDT", "balance": "1", "updateTime": 2
    }));
    assert_eq!(exchanged.account_id, 1);
    let leverage: LeverageChange = dec(json!({
        "leverage": 21, "maxNotionalValue": "1000000", "symbol": "BTCUSDT"
    }));
    assert_eq!(leverage.leverage, 21);

    let spot: SpotAccount = dec(json!({
        "feeTier": 0, "canTrade": true, "canDeposit": true, "canWithdraw": true,
        "canBurnAsset": true, "updateTime": 1,
        "balances": [{"asset": "USDT", "free": "100", "locked": "0"}]
    }));
    assert_eq!(spot.balances[0].free, Decimal::from(100));

    let fee: WithdrawEstimateFee = dec(json!({
        "tokenPrice": 1.00019, "gasCost": 0.5, "gasUsdValue": 0.5
    }));
    assert_eq!(fee.token_price, Decimal::new(100019, 5));
    let withdrawn: WithdrawResult = dec(json!({"withdrawId": "1", "hash": "0xabc"}));
    assert_eq!(withdrawn.hash, "0xabc");
    let record: DepositWithdrawRecord = dec(json!({
        "id": "1", "type": "DEPOSIT", "asset": "USDT", "amount": "100", "state": "SUCCESS",
        "txHash": "0xabc", "time": 1, "chainId": 56, "accountType": "spot"
    }));
    assert_eq!(record.record_type, "DEPOSIT");

    let agent: AgentInfo = dec(json!({
        "agentAddress": "0xa", "agentName": "bot", "expired": 1, "canSpotTrade": true,
        "canPerpTrade": true, "canWithdraw": false, "source": "api", "ipWhitelist": "10.0.0.0/24",
        "extraField": 1
    }));
    assert_eq!(agent.agent_address, "0xa");
    assert_eq!(agent.extra.get("extraField"), Some(&json!(1)));

    // Paginated builder envelopes.
    let page: Paginated<BuilderUserAccount> = dec(json!({
        "total": 1, "currentPage": 1, "totalPages": 1, "pageSize": 50, "hasMore": false,
        "rows": [{
            "address": "0xa", "feeTier": 0, "canTrade": true, "canDeposit": true,
            "canWithdraw": false, "updateTime": 1, "accountType": 0,
            "dualSidePosition": false, "jointMargin": false, "feeBurn": false,
            "feeBurnAssetId": 0,
            "symbolConfig": [{"symbol": "BTCUSDT", "leverage": 20, "notionalLimitCoef": "10"}]
        }],
        "errors": [{"address": "0xb", "errorMsg": "privacy"}]
    }));
    assert_eq!(page.rows[0].symbol_config[0].leverage, Some(20));
    assert_eq!(page.errors[0].address, "0xb");

    let _: Paginated<BuilderUserBalance> = dec(json!({
        "rows": [{"address": "0xa", "asset": "USDT", "walletBalance": "1", "price": "1",
                  "balanceInUsd": "1"}]
    }));
    let _: Paginated<BuilderUserPositionRisk> = dec(json!({
        "rows": [{
            "address": "0xa", "entryPrice": "1", "leverage": "10", "liquidationPrice": "0",
            "markPrice": "1", "positionAmt": "1", "symbol": "BTCUSDT",
            "unRealizedProfit": "0", "positionSide": "BOTH"
        }]
    }));
    let _: Paginated<BuilderUserCommissionRate> = dec(json!({
        "rows": [{"address": "0xa", "symbol": "BTCUSDT", "makerCommissionRate": "0.0001",
                  "takerCommissionRate": "0.0002"}]
    }));
    let _: Paginated<BuilderOrder> = dec(json!({
        "rows": [{
            "address": "0xa", "clientOrderId": "x", "executedQty": "0", "orderId": 1,
            "origQty": "1", "price": "1", "side": "BUY", "status": "NEW", "symbol": "BTCUSDT",
            "type": "LIMIT"
        }]
    }));
    let _: Paginated<BuilderTrade> = dec(json!({
        "rows": [{
            "userAddress": "0xa", "tradeId": 1, "insertTime": 2, "symbol": "BTCUSDT",
            "price": "1", "qty": "2", "baseAsset": "BTC", "baseType": "STOCK",
            "quoteAsset": "USDT", "side": "BUY", "activeBuy": true, "feeAsset": "USDT",
            "totalQuota": "2", "fee": "-0.01", "orderId": 3, "realizedProfit": "0",
            "marginAsset": "USDT", "builderFee": "0.0001"
        }]
    }));
    let _: Paginated<BuilderCommission> = dec(json!({
        "rows": [{"tradeId": 1, "insertTime": 2, "orderId": 3, "symbol": "BTCUSDT",
                  "feeAsset": "USDT", "marginAsset": "USDT", "builderFee": "0.0001",
                  "userAddress": "0xa"}]
    }));
    let _: Paginated<ApprovedUser> = dec(json!({
        "rows": [{"userAddress": "0xa", "builderAddress": "0xb", "maxFeeRate": 0.0001,
                  "builderName": "acme", "approveTime": 1}]
    }));

    let announcements: AnnouncementPage = dec(json!({
        "total": 1,
        "rows": [{
            "id": 1, "category": "DIRECT", "publishTime": 1, "jumpLink": "https://x",
            "contents": [{"language": "en", "title": "t", "subtitle": "s", "content": "c"}]
        }]
    }));
    assert_eq!(announcements.rows[0].contents[0].language, "en");
}

#[test]
fn strategy_models() {
    let result: StrategyOrderResult = dec(json!({
        "strategyId": 1, "strategyType": "OTO", "strategyStatus": "WORKING",
        "updateTime": 1, "failureCode": 0, "failureReason": ""
    }));
    assert!(result.rejection().is_none());
    let rejected: StrategyOrderResult = dec(json!({
        "strategyId": 1, "strategyType": "OTO", "strategyStatus": "REJECTED",
        "failureCode": -4142, "failureReason": "would trigger"
    }));
    assert_eq!(rejected.rejection().unwrap().code, -4142);

    let updated: Vec<StrategyUpdateResult> = dec(json!([{
        "strategyId": 1, "strategyType": "OTO", "strategyStatus": "WORKING",
        "updatedSubOrder": 1, "updateStatus": "SUCCESS", "failureCode": 0
    }]));
    assert!(updated[0].rejection().is_none());
    let failed: StrategyUpdateResult = dec(json!({
        "strategyId": 1, "strategyType": "OTO", "strategyStatus": "WORKING",
        "failureCode": -1, "failureReason": "bad"
    }));
    assert_eq!(failed.rejection().unwrap().code, -1);

    let open: StrategyOrder = dec(json!({
        "strategyId": 1, "clientStrategyId": "s", "strategyType": "OTO",
        "strategyStatus": "WORKING", "bookTime": 1, "updateTime": 2,
        "subOrders": [{
            "strategyId": 1, "orderId": 2, "clientOrderId": "o", "status": "NEW",
            "strategySubId": 1, "firstDrivenId": 0, "firstDrivenOn": "", "firstTrigger": "",
            "secondDrivenId": 0, "secondDrivenOn": "", "secondTrigger": "",
            "securityType": "FUTURE", "symbol": "BTCUSDT", "side": "BUY",
            "positionSide": "LONG", "type": "LIMIT", "timeInForce": "GTC",
            "quantity": "0.001", "reduceOnly": false, "closePosition": false,
            "price": "50000", "avgPrice": "0", "priceProtect": false, "stopPrice": "0",
            "activatePrice": null, "callbackRate": null, "workingType": "CONTRACT_PRICE",
            "triggerTime": 0, "pegPriceType": "QUEUE_1"
        }]
    }));
    assert_eq!(open.sub_orders[0].quantity, Decimal::new(1, 3));
    assert_eq!(
        open.sub_orders[0].peg_price_type.as_deref(),
        Some("QUEUE_1")
    );

    let chase: ChaseOrderResult = dec(json!({
        "strategyId": 1, "clientStrategyId": "c", "symbol": "BTCUSDT", "side": "BUY",
        "positionSide": "BOTH", "quantity": "0.1", "quantityUnit": "BASE", "reduceOnly": false,
        "chaseOffset": "0.5", "chaseOffsetType": "ABSOLUTE", "maxChaseOffset": "10",
        "maxChaseOffsetType": "PERCENTAGE", "timeInForce": "GTX", "strategyStatus": "NEW",
        "bookTime": 1, "updateTime": 2
    }));
    assert_eq!(chase.quantity, Decimal::new(1, 1));
}

#[test]
fn websocket_events() {
    let raw = [
        r#"{"e":"aggTrade","E":1,"s":"BTCUSDT","a":2,"p":"1.5","q":"2","f":1,"l":1,"T":1,"m":true}"#,
        r#"{"e":"markPriceUpdate","E":1,"s":"BTCUSDT","p":"1","i":"2","P":"1","r":"0.1","T":2}"#,
        r#"{"e":"kline","E":1,"s":"BTCUSDT","k":{"t":1,"T":2,"s":"BTCUSDT","i":"1m","o":"1","c":"2","h":"3","l":"0.5","v":"10","n":3,"x":true,"q":"20","V":"5","Q":"10"}}"#,
        r#"{"e":"24hrMiniTicker","E":1,"s":"BTCUSDT","c":"1","o":"1","h":"2","l":"1","v":"10","q":"10"}"#,
        r#"{"e":"24hrTicker","E":1,"s":"BTCUSDT","p":"1","P":"2","c":"3","Q":"0.1","o":"1","h":"2","l":"1","v":"10","q":"10"}"#,
        r#"{"e":"bookTicker","E":1,"u":1,"s":"BTCUSDT","b":"1","B":"2","a":"3","A":"4","T":2}"#,
        r#"{"e":"forceOrder","E":1,"o":{"s":"BTCUSDT","S":"SELL","o":"LIMIT","q":"1","p":"1","ap":"1","X":"FILLED","T":2}}"#,
        r#"{"e":"depthUpdate","E":1,"T":2,"s":"BTCUSDT","U":1,"u":2,"pu":0,"b":[["1","2"]],"a":[["3","4"]]}"#,
        r#"{"e":"tradepro","E":1,"T":2,"s":"BTCUSDT","t":3,"p":"1","q":"2","h":"0xhash","m":["a","b"]}"#,
        r#"{"e":"ACCOUNT_UPDATE","E":1,"T":2,"a":{"m":"ORDER","B":[{"a":"USDT","wb":"1","cw":"1","bc":"0"}],"P":[{"s":"BTCUSDT","pa":"1","ep":"1","cr":"0","up":"0","mt":"isolated","iw":"0","ps":"BOTH"}]}}"#,
        r#"{"e":"ORDER_TRADE_UPDATE","E":1,"T":2,"o":{"s":"BTCUSDT","c":"x","S":"SELL","o":"LIMIT","f":"GTC","q":"1","p":"1","ap":"1","sp":"0","x":"TRADE","X":"FILLED","i":1,"l":"1","z":"1","L":"1","N":"USDT","n":"0.01","T":2,"t":3,"b":"0","a":"0","m":false,"R":false,"wt":"CONTRACT_PRICE","ot":"LIMIT","ps":"BOTH","cp":false,"AP":"0","cr":"0","rp":"0","stp":"EXPIRE_BOTH"}}"#,
        r#"{"e":"MARGIN_CALL","E":1,"cw":"1","p":[{"s":"BTCUSDT","ps":"LONG","pa":"1","mt":"CROSSED","iw":"0","mp":"1","up":"-0.1","mm":"0.1"}]}"#,
        r#"{"e":"ACCOUNT_CONFIG_UPDATE","E":1,"T":2,"ac":{"s":"BTCUSDT","l":25},"ai":{"j":true,"f":false,"d":true}}"#,
        r#"{"e":"listenKeyExpired","E":1}"#,
        r#"{"e":"announcement","en":{"i":1,"t":"t","s":"s","c":"NEW_LISTING","d":1,"S":false,"j":"https://x","C":"<p>x</p>"},"zh":{"i":1,"t":"t","c":"DIRECT","d":1}}"#,
        r#"{"e":"unknownKind","E":1}"#,
    ];
    let events: Vec<Event> = raw
        .iter()
        .map(|r| serde_json::from_str(r).unwrap())
        .collect();
    for event in &events {
        let _ = event.event_time();
    }
    assert!(matches!(events[0], Event::AggTrade(_)));
    assert!(matches!(events[1], Event::MarkPrice(_)));
    assert!(matches!(events[2], Event::Kline(_)));
    assert!(matches!(events[3], Event::MiniTicker(_)));
    assert!(matches!(events[4], Event::Ticker(_)));
    assert!(matches!(events[5], Event::BookTicker(_)));
    assert!(matches!(events[6], Event::ForceOrder(_)));
    assert!(matches!(events[7], Event::DepthUpdate(_)));
    assert!(matches!(events[8], Event::TradePro(_)));
    assert!(matches!(events[9], Event::AccountUpdate(_)));
    assert!(matches!(events[10], Event::OrderTradeUpdate(_)));
    assert!(matches!(events[11], Event::MarginCall(_)));
    assert!(matches!(events[12], Event::AccountConfigUpdate(_)));
    assert!(matches!(events[13], Event::ListenKeyExpired(_)));
    assert!(matches!(events[14], Event::Announcement(_)));
    assert!(matches!(events[15], Event::Unknown));
    assert_eq!(events[0].event_time(), Some(1));

    let wrapped: CombinedStream = serde_json::from_str(
        r#"{"stream":"btcusdt@aggTrade","data":{"e":"aggTrade","E":1,"s":"BTCUSDT","a":2,"p":"1.5","q":"2","f":1,"l":1,"T":1,"m":true}}"#,
    )
    .unwrap();
    assert_eq!(wrapped.stream, "btcusdt@aggTrade");
    assert!(matches!(wrapped.data, Event::AggTrade(_)));
}
