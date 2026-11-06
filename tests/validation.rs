//! Validation matrix and low-level helper coverage.

use std::sync::Mutex;

use aster_client::auth::{EvmSigner, Signature, Signer};
use aster_client::client::{
    Client, Endpoint, Environment, HttpRequest, HttpResponse, Method, Security, Transport, Venue,
};
use aster_client::error::Error;
use aster_client::model::*;
use aster_client::params::{Param, Params};
use rust_decimal::Decimal;

const KEY: &str = "0x0123456789012345678901234567890123456789012345678901234567890123";

fn d(value: &str) -> Decimal {
    value.parse().unwrap()
}

struct Fake {
    body: String,
    status: u16,
    seen: Mutex<Vec<HttpRequest>>,
}

impl Default for Fake {
    fn default() -> Self {
        Self {
            body: "{}".into(),
            status: 200,
            seen: Mutex::new(Vec::new()),
        }
    }
}

impl Transport for Fake {
    async fn execute(&self, request: HttpRequest) -> aster_client::Result<HttpResponse> {
        self.seen.lock().unwrap().push(request);
        Ok(HttpResponse {
            status: self.status,
            body: self.body.clone(),
            retry_after: None,
        })
    }
}

/// Synthetic endpoint authenticated with the master wallet (`Security::User`),
/// which the public API has no concrete route for yet.
struct UserOnly;

impl Endpoint for UserOnly {
    type Response = serde_json::Value;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/fapi/v3/account";
    const VENUE: Venue = Venue::Futures;
    const SECURITY: Security = Security::User;
}

#[tokio::test]
async fn master_wallet_authentication_branch() {
    let client = Client::with_transport(Environment::Mainnet, Fake::default())
        .signer(EvmSigner::from_hex(KEY).unwrap());
    // Without `user` the master-authenticated endpoint must fail before sending.
    let error = client.send(UserOnly).await.unwrap_err();
    assert!(matches!(error, Error::MissingCredentials));
    assert!(client.transport().seen.lock().unwrap().is_empty());

    let address: alloy_primitives::Address = "0x63DD5aCC6b1aa0f563956C0e534DD30B6dcF7C4e"
        .parse()
        .unwrap();
    let client = client.with_user(address);
    client.send(UserOnly).await.unwrap();
    let seen = client.transport().seen.lock().unwrap();
    let url = &seen[0].url;
    assert!(
        url.contains("user=0x63dd5acc6b1aa0f563956c0e534dd30b6dcf7c4e"),
        "{url}"
    );
    assert!(url.contains("signature=0x"), "{url}");
}

#[test]
fn new_order_validation_matrix() {
    use aster_client::futures::NewOrder;

    // Each order type rejects a missing mandatory parameter.
    let mut order = NewOrder::limit("BTCUSDT", Side::Buy, "1", "100").unwrap();
    order.time_in_force = None;
    assert!(order.params().is_err(), "LIMIT without timeInForce");

    let mut order = NewOrder::limit("BTCUSDT", Side::Buy, "1", "100").unwrap();
    order.time_in_force = Some(TimeInForce::Gtc);
    order.price = None;
    assert!(order.params().is_err(), "LIMIT without price");

    let mut order = NewOrder::market("BTCUSDT", Side::Buy, "1").unwrap();
    order.quantity = None;
    assert!(order.params().is_err(), "MARKET without quantity");

    let mut order = NewOrder::market("BTCUSDT", Side::Buy, "1").unwrap();
    order.price = Some(d("1"));
    assert!(order.params().is_err(), "MARKET with price");

    let mut order = NewOrder::stop("BTCUSDT", Side::Buy, "1", "100", "99").unwrap();
    order.stop_price = None;
    assert!(order.params().is_err(), "STOP without stopPrice");

    let mut order = NewOrder::take_profit("BTCUSDT", Side::Buy, "1", "100", "101").unwrap();
    order.quantity = None;
    assert!(order.params().is_err(), "TAKE_PROFIT without quantity");

    let mut order = NewOrder::stop_market("BTCUSDT", Side::Sell, "99").unwrap();
    order.stop_price = None;
    assert!(order.params().is_err(), "STOP_MARKET without stopPrice");

    let mut order = NewOrder::stop_market("BTCUSDT", Side::Sell, "99").unwrap();
    order.quantity = None;
    assert!(order.params().is_err(), "STOP_MARKET without quantity");

    let order = NewOrder::stop_market("BTCUSDT", Side::Sell, "99")
        .unwrap()
        .close_position(true);
    assert!(order.params().is_ok(), "close-all needs no quantity");

    let mut order = NewOrder::stop_market("BTCUSDT", Side::Sell, "99")
        .unwrap()
        .close_position(true);
    order.quantity = Some(d("1"));
    assert!(order.params().is_err(), "closePosition + quantity");

    let mut order = NewOrder::stop_market("BTCUSDT", Side::Sell, "99")
        .unwrap()
        .close_position(true);
    order.reduce_only = Some(true);
    assert!(order.params().is_err(), "closePosition + reduceOnly");

    let mut order = NewOrder::limit("BTCUSDT", Side::Buy, "1", "100").unwrap();
    order.time_in_force = Some(TimeInForce::Gtc);
    order.close_position = Some(true);
    assert!(order.params().is_err(), "closePosition on LIMIT");

    let mut order = NewOrder::limit("BTCUSDT", Side::Buy, "1", "100").unwrap();
    order.time_in_force = Some(TimeInForce::Gtc);
    order.stop_price = Some(d("1"));
    assert!(order.params().is_err(), "stopPrice on LIMIT");

    let mut order = NewOrder::take_profit_market("BTCUSDT", Side::Sell, "101").unwrap();
    order.price = Some(d("1"));
    assert!(order.params().is_err(), "price on TAKE_PROFIT_MARKET");

    let mut order = NewOrder::trailing_stop_market("BTCUSDT", Side::Buy, "0.5").unwrap();
    order.callback_rate = None;
    assert!(order.params().is_err(), "TRAILING without callbackRate");

    let mut order = NewOrder::trailing_stop_market("BTCUSDT", Side::Buy, "0.5").unwrap();
    order.callback_rate = Some(d("0.05"));
    assert!(order.params().is_err(), "callbackRate below 0.1");

    let mut order = NewOrder::trailing_stop_market("BTCUSDT", Side::Buy, "0.5").unwrap();
    order.callback_rate = Some(d("6"));
    assert!(order.params().is_err(), "callbackRate above 5");

    let mut order = NewOrder::trailing_stop_market("BTCUSDT", Side::Buy, "0.5").unwrap();
    order.price = Some(d("1"));
    assert!(order.params().is_err(), "price on TRAILING");

    let mut order = NewOrder::limit("BTCUSDT", Side::Buy, "1", "100").unwrap();
    order.time_in_force = Some(TimeInForce::Gtc);
    order.activation_price = Some(d("1"));
    assert!(order.params().is_err(), "activationPrice on LIMIT");

    let mut order = NewOrder::limit("BTCUSDT", Side::Buy, "1", "100").unwrap();
    order.time_in_force = Some(TimeInForce::Gtc);
    order.builder = Some("0xb".into());
    assert!(order.params().is_err(), "builder without feeRate");

    let mut order = NewOrder::limit("BTCUSDT", Side::Buy, "1", "100").unwrap();
    order.time_in_force = Some(TimeInForce::Gtc);
    order.client_order_id = Some("bad id".into());
    assert!(order.params().is_err(), "invalid clientOrderId");

    let mut order = NewOrder::limit("BTCUSDT", Side::Buy, "1", "100").unwrap();
    order.time_in_force = Some(TimeInForce::Gtc);
    order.price = None;
    assert!(order.clone().params().is_err());
    order.price = Some(d("100"));
    assert!(order.params().is_ok());
}

#[tokio::test]
async fn order_lifecycle_validation() {
    use aster_client::futures::*;

    assert!(
        ModifyOrder::new("BTCUSDT", d("1"), d("100"))
            .params()
            .is_err(),
        "no order id"
    );
    assert!(
        CancelOrder {
            symbol: "BTCUSDT".into(),
            order_id: None,
            orig_client_order_id: None
        }
        .params()
        .is_err()
    );
    assert!(
        GuardedCancelOrder {
            symbol: "BTCUSDT".into(),
            order_id: None,
            orig_client_order_id: None,
            nonce: 1,
        }
        .params()
        .is_err()
    );
    assert!(BatchNewOrders { orders: vec![] }.params().is_err());
    let five = || {
        (0..6)
            .map(|i| {
                NewOrder::market("BTCUSDT", Side::Buy, "1")
                    .unwrap()
                    .client_order_id(format!("o{i}"))
            })
            .collect::<Vec<_>>()
    };
    assert!(
        BatchNewOrders {
            orders: five()[..5].to_vec()
        }
        .params()
        .is_ok()
    );
    assert!(BatchNewOrders { orders: five() }.params().is_err(), "max 5");
    assert!(BatchModifyOrders { orders: vec![] }.params().is_err());

    let mut chase = ChaseOrder::new("BTCUSDT", Side::Buy, QuantityUnit::Base, d("1"));
    chase.client_strategy_id = Some("x".repeat(29));
    assert!(chase.params().is_err(), "clientStrategyId longer than 28");
    chase.client_strategy_id = Some("bad id".into());
    assert!(chase.params().is_err(), "clientStrategyId charset");

    let chase = ChaseOrder::new("BTCUSDT", Side::Buy, QuantityUnit::Base, d("1"));
    assert_eq!(chase.quantity_unit.as_str(), "BASE");

    // Strategy lookups require exactly one identifier.
    use aster_client::model::StrategyType;
    let both = StrategyLookup {
        strategy_id: Some(1),
        client_strategy_id: Some("s".into()),
        strategy_type: StrategyType::Oto,
    };
    assert!(StrategyOpenOrder(both).params().is_err());
    let neither = StrategyLookup {
        strategy_id: None,
        client_strategy_id: None,
        strategy_type: StrategyType::Oto,
    };
    assert!(
        StrategyHistoryOrder {
            lookup: neither,
            start_time: None,
            end_time: None,
            limit: None
        }
        .params()
        .is_err()
    );

    // `PlaceStrategyOrder` enforces the leg count per strategy family.
    let leg = |id: u32| StrategySubOrderRequest {
        strategy_sub_id: id,
        security_type: SecurityType::UsdtFutures,
        symbol: "BTCUSDT".into(),
        side: Side::Buy,
        position_side: None,
        order_type: OrderType::Limit,
        quantity: Some(d("1")),
        price: Some(d("100")),
        peg_price_type: None,
        stop_price: None,
        time_in_force: Some(TimeInForce::Gtc),
        working_type: None,
        reduce_only: None,
        close_position: None,
        price_protect: None,
        client_order_id: None,
        activation_price: None,
        callback_rate: None,
        first_driven_id: None,
        first_driven_on: None,
        first_trigger: None,
        second_driven_id: None,
        second_driven_on: None,
        second_trigger: None,
    };
    let single = PlaceStrategyOrder {
        strategy_type: StrategyType::Oto,
        sub_orders: vec![leg(1)],
        client_strategy_id: None,
        builder: None,
        fee_rate: None,
    };
    assert!(single.params().is_err(), "OTO needs two legs");
    let otoco = PlaceStrategyOrder {
        strategy_type: StrategyType::Otoco,
        sub_orders: vec![leg(1), leg(2)],
        client_strategy_id: None,
        builder: None,
        fee_rate: None,
    };
    assert!(otoco.params().is_err(), "OTOCO needs three legs");
    let ok = PlaceStrategyOrder {
        strategy_type: StrategyType::Otoco,
        sub_orders: vec![leg(1), leg(2), leg(3)],
        client_strategy_id: None,
        builder: None,
        fee_rate: None,
    };
    assert!(ok.params().is_ok());

    // Listen keys are unsigned by construction only in tests; params are empty.
    assert!(StartListenKey.params().unwrap().is_empty());
}

#[test]
fn spot_order_validation_extras() {
    use aster_client::spot::NewSpotOrder;

    let mut order = NewSpotOrder::limit("BTCUSDT", Side::Buy, "1", "100").unwrap();
    order.price = None;
    assert!(order.params().is_err(), "LIMIT without price");

    let mut order = NewSpotOrder::market_buy_quote("BTCUSDT", "10").unwrap();
    order.quote_order_qty = None;
    assert!(
        order.params().is_err(),
        "MARKET without quantity/quoteOrderQty"
    );

    let mut order = NewSpotOrder::market_sell("BTCUSDT", "1").unwrap();
    order.price = Some(d("1"));
    assert!(order.params().is_err(), "MARKET with price");

    let mut order = NewSpotOrder::stop("BTCUSDT", Side::Sell, "1", "100", "99").unwrap();
    order.price = None;
    assert!(order.params().is_err(), "STOP without price");

    let mut order = NewSpotOrder::take_profit_market("BTCUSDT", Side::Sell, "1", "100").unwrap();
    order.price = Some(d("1"));
    assert!(order.params().is_err(), "TPM with price");

    let mut order = NewSpotOrder::limit("BTCUSDT", Side::Buy, "1", "100").unwrap();
    order.time_in_force = Some(TimeInForce::Gtc);
    order.builder = Some("0xb".into());
    assert!(order.params().is_err(), "builder without feeRate");

    let mut order = NewSpotOrder::limit("BTCUSDT", Side::Buy, "1", "100").unwrap();
    order.time_in_force = Some(TimeInForce::Gtc);
    order.fee_rate = Some(d("0.001"));
    assert!(order.params().is_err(), "feeRate without builder");
}

#[test]
fn market_weights_and_spot_params() {
    use aster_client::futures;
    use aster_client::futures::StrategyLookup;
    use aster_client::model::KlineInterval;

    for (limit, weight) in [
        (Some(1), 1),
        (Some(100), 2),
        (Some(500), 5),
        (Some(1500), 10),
        (None, 5),
    ] {
        assert_eq!(
            futures::Klines {
                symbol: "BTCUSDT".into(),
                interval: KlineInterval::Min1,
                start_time: None,
                end_time: None,
                limit
            }
            .weight(),
            weight
        );
    }
    assert_eq!(futures::AggTrades::new("BTCUSDT").weight(), 20);
    assert_eq!(futures::HistoricalTrades::new("BTCUSDT").weight(), 20);
    assert_eq!(futures::AllOrders::default().weight(), 5);
    assert_eq!(futures::IncomeHistory::default().weight(), 30);
    assert_eq!(futures::IncomeHistoryPage::default().weight(), 30);
    assert_eq!(futures::UserTrades::new("BTCUSDT").weight(), 5);
    assert_eq!(futures::ForceOrders::default().weight(), 20);
    assert_eq!(
        futures::WalletTransfer {
            amount: d("1"),
            asset: "USDT".into(),
            client_tran_id: "t".into(),
            kind: TransferKind::FutureSpot
        }
        .weight(),
        50
    );
    assert_eq!(
        futures::PlaceStrategyOrder {
            strategy_type: StrategyType::Oto,
            sub_orders: vec![],
            client_strategy_id: None,
            builder: None,
            fee_rate: None
        }
        .weight(),
        50
    );
    assert_eq!(
        futures::StrategyOpenOrder(StrategyLookup {
            strategy_id: Some(1),
            client_strategy_id: None,
            strategy_type: StrategyType::Oto
        })
        .weight(),
        5
    );
    assert_eq!(
        futures::CountdownCancelAll {
            symbol: "BTCUSDT".into(),
            countdown_time: 1
        }
        .weight(),
        10
    );
    assert_eq!(futures::DirectAnnouncements::default().weight(), 1);

    let mut spot_klines = aster_client::spot::Klines::new("BTCUSDT", KlineInterval::Min3);
    spot_klines.start_time = Some(1);
    spot_klines.end_time = Some(2);
    spot_klines.limit = Some(3);
    assert_eq!(
        spot_klines.params().unwrap().pairs(),
        vec![
            ("endTime", "2".to_owned()),
            ("interval", "3m".to_owned()),
            ("limit", "3".to_owned()),
            ("startTime", "1".to_owned()),
            ("symbol", "BTCUSDT".to_owned()),
        ]
    );
    assert_eq!(
        aster_client::spot::Ticker24hr::symbol("BTCUSDT").weight(),
        1
    );
    assert_eq!(
        aster_client::spot::PriceTicker::symbol("BTCUSDT").weight(),
        1
    );
    assert_eq!(
        aster_client::spot::BookTicker::symbol("BTCUSDT").weight(),
        1
    );
    assert_eq!(aster_client::spot::AllSpotOrders::default().weight(), 5);
    assert_eq!(
        aster_client::spot::SpotUserTrades::new("BTCUSDT").weight(),
        5
    );
    assert_eq!(aster_client::spot::Trades::new("BTCUSDT").weight(), 1);
    assert_eq!(
        aster_client::spot::HistoricalTrades {
            symbol: "BTCUSDT".into(),
            limit: None,
            from_id: None
        }
        .weight(),
        1
    );
    assert_eq!(aster_client::spot::AggTrades::new("BTCUSDT").weight(), 1);
    assert_eq!(
        aster_client::spot::CommissionRateQuery {
            symbol: "BTCUSDT".into()
        }
        .weight(),
        1
    );
    assert_eq!(aster_client::spot::Noop.weight(), 1);
    assert_eq!(aster_client::spot::SpotAccount.weight(), 5);
}

#[test]
fn params_cover_all_scalar_types() {
    let mut p = Params::new();
    p.set("i8", &-1_i8);
    p.set("i16", &-2_i16);
    p.set("i32", &-3_i32);
    p.set("i64", &-4_i64);
    p.set("u8", &5_u8);
    p.set("u16", &6_u16);
    p.set("u32", &7_u32);
    p.set("u64", &8_u64);
    p.set("usize", &9_usize);
    p.set("decimal", &Decimal::new(1250, 2));
    p.set("bool", &false);
    p.set("none", &Option::<u32>::None);
    p.set("some", &Some(1_u32));
    p.set("owned", &"text".to_owned());
    p.set("borrowed", &"text");
    assert_eq!(p.pairs().len(), 14);
    assert_eq!(p.get("decimal"), Some("12.50"));
    let encoded = p.encode();
    assert!(encoded.starts_with("bool=false&borrowed=text&decimal=12.50"));
    assert!(encoded.contains("i8=-1") && encoded.contains("u64=8"));
    assert_eq!(
        p.encode_signed("0x1").split('&').next_back(),
        Some("signature=0x1")
    );
    assert_eq!(Params::new().encode_signed("0x1"), "signature=0x1");

    // `Param` is public so custom types can be sent.
    struct Custom(u8);
    impl Param for Custom {
        fn write(&self, key: &'static str, out: &mut Params) {
            out.set_str(key, self.0.to_string());
        }
    }
    let mut p = Params::new();
    Custom(7).write("custom", &mut p);
    assert_eq!(p.get("custom"), Some("7"));
}

#[test]
fn signing_error_paths() {
    assert!(EvmSigner::from_hex("0xzz").is_err());
    assert!(EvmSigner::from_hex("0x00").is_err());
    assert!(EvmSigner::from_bytes(&[0u8; 32]).is_err());

    let signer = EvmSigner::from_hex(KEY).unwrap();
    assert_eq!(signer.public_address(), signer.address());
    assert!(format!("{signer:?}").contains("EvmSigner"));

    let signature: Signature = signer
        .sign_digest(aster_client::auth::keccak(b"payload"))
        .unwrap();
    assert_eq!(signature.to_hex(), signature.to_string());
    assert!(Signature::from_bytes(*signature.as_bytes()).is_ok());

    // v = 27 is accepted as well.
    let mut bytes = *signature.as_bytes();
    bytes[64] = 27;
    assert!(Signature::from_bytes(bytes).is_ok());
}

#[test]
fn funding_extras() {
    use aster_client::auth::WithdrawChain;
    use aster_client::funding::*;

    let signer = EvmSigner::from_hex(KEY).unwrap();
    let signed = WithdrawIntent {
        chain: WithdrawChain::Ethereum,
        asset: "ETH".into(),
        amount: d("1"),
        fee: d("0.1"),
        receiver: signer.address(),
    }
    .sign(&signer, Some(5))
    .unwrap();

    let params = FuturesWithdraw(signed).params().unwrap();
    assert_eq!(params.get("chainId"), Some("1"));
    assert_eq!(params.get("asset"), Some("ETH"));

    let estimate = EstimateWithdrawFee {
        chain: WithdrawChain::Ethereum,
        asset: "USDT".into(),
    };
    assert_eq!(estimate.params().unwrap().get("chainId"), Some("1"));

    assert_eq!(WithdrawChain::Ethereum.as_str(), "ETH");
    assert_eq!(WithdrawChain::Arbitrum.chain_id(), 42_161);

    // Every vault and the native placeholder are distinct constants.
    assert_eq!(deposit::NATIVE_CURRENCY.as_slice()[0], 0xfd);
    assert_ne!(
        deposit::DepositChain::Ethereum.vault(),
        deposit::DepositChain::Arbitrum.vault()
    );
    assert_eq!(deposit::BROKER_SPOT, 1000);
    assert_eq!(
        deposit::deposit_for_calldata(deposit::NATIVE_CURRENCY, signer.address(), 1, 0).len(),
        132
    );
}

#[test]
fn rpc_configuration() {
    use aster_client::rpc::{MAINNET_RPC_URL, RpcClient};

    let mainnet = RpcClient::mainnet();
    assert!(format!("{mainnet:?}").contains("RpcClient"));
    let custom = RpcClient::new("http://localhost:8545");
    let _ = custom.transport();
    assert!(MAINNET_RPC_URL.starts_with("https://"));
}
