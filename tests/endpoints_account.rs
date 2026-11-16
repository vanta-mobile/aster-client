//! Contract tests for account, agent, builder and funding endpoints.

use aster_client::auth::{AsterChain, EvmSigner, WithdrawChain};
use aster_client::client::{Endpoint, Method, Security, Venue};
use aster_client::model::*;
use rust_decimal::Decimal;

fn d(value: &str) -> Decimal {
    value.parse().unwrap()
}

const GET: Method = Method::Get;
const POST: Method = Method::Post;
const DELETE: Method = Method::Delete;
const FUT: Venue = Venue::Futures;
const SPOT: Venue = Venue::Spot;
const SIGNER: Security = Security::Signer;
const NONE: Security = Security::None;

const KEY: &str = "0x0123456789012345678901234567890123456789012345678901234567890123";

fn signer() -> EvmSigner {
    EvmSigner::from_hex(KEY).unwrap()
}

fn meta<E: Endpoint>(
    endpoint: &E,
    method: Method,
    path: &'static str,
    venue: Venue,
    security: Security,
) {
    assert_eq!(E::METHOD, method, "method for {path}");
    assert_eq!(E::PATH, path, "path");
    assert_eq!(E::VENUE, venue, "venue for {path}");
    assert_eq!(E::SECURITY, security, "security for {path}");
    let _ = endpoint.weight();
}

fn check<E: Endpoint>(
    endpoint: E,
    method: Method,
    path: &'static str,
    venue: Venue,
    security: Security,
    expected: &[(&'static str, &str)],
) {
    meta(&endpoint, method, path, venue, security);
    let mut want: Vec<(&'static str, String)> = expected
        .iter()
        .map(|(k, v)| (*k, (*v).to_owned()))
        .collect();
    want.sort_by(|a, b| a.0.cmp(b.0));
    assert_eq!(
        endpoint.params().unwrap().pairs(),
        want,
        "params for {path}"
    );
}

#[test]
fn futures_account_endpoints() {
    use aster_client::futures::*;

    check(
        QueryOrder(OrderLookup::by_id("BTCUSDT", 7)),
        GET,
        "/fapi/v3/order",
        FUT,
        SIGNER,
        &[("symbol", "BTCUSDT"), ("orderId", "7")],
    );
    check(
        QueryOpenOrder(OrderLookup::by_client_id("BTCUSDT", "a1")),
        GET,
        "/fapi/v3/openOrder",
        FUT,
        SIGNER,
        &[("symbol", "BTCUSDT"), ("origClientOrderId", "a1")],
    );
    assert!(
        QueryOrder(OrderLookup {
            symbol: "BTCUSDT".into(),
            order_id: None,
            orig_client_order_id: None
        })
        .params()
        .is_err()
    );

    check(
        OpenOrders::symbol("BTCUSDT"),
        GET,
        "/fapi/v3/openOrders",
        FUT,
        SIGNER,
        &[("symbol", "BTCUSDT")],
    );
    assert_eq!(OpenOrders::default().weight(), 40);
    assert_eq!(OpenOrders::symbol("BTCUSDT").weight(), 1);
    check(
        AllOrders {
            symbol: Some("BTCUSDT".into()),
            order_id: Some(1),
            start_time: Some(2),
            end_time: Some(3),
            limit: Some(4),
        },
        GET,
        "/fapi/v3/allOrders",
        FUT,
        SIGNER,
        &[
            ("symbol", "BTCUSDT"),
            ("orderId", "1"),
            ("startTime", "2"),
            ("endTime", "3"),
            ("limit", "4"),
        ],
    );

    check(Balance, GET, "/fapi/v3/balance", FUT, SIGNER, &[]);
    check(Account, GET, "/fapi/v3/account", FUT, SIGNER, &[]);
    check(
        AccountWithJoinMargin,
        GET,
        "/fapi/v3/accountWithJoinMargin",
        FUT,
        SIGNER,
        &[],
    );
    check(
        Positions::symbol("BTCUSDT"),
        GET,
        "/fapi/v3/positionRisk",
        FUT,
        SIGNER,
        &[("symbol", "BTCUSDT")],
    );
    check(
        UserTrades {
            symbol: "BTCUSDT".into(),
            order_id: Some(1),
            start_time: Some(2),
            end_time: Some(3),
            from_id: Some(4),
            limit: Some(5),
        },
        GET,
        "/fapi/v3/userTrades",
        FUT,
        SIGNER,
        &[
            ("symbol", "BTCUSDT"),
            ("orderId", "1"),
            ("startTime", "2"),
            ("endTime", "3"),
            ("fromId", "4"),
            ("limit", "5"),
        ],
    );
    check(
        IncomeHistory {
            symbol: Some("BTCUSDT".into()),
            income_type: Some("FUNDING_FEE".into()),
            start_time: Some(2),
            end_time: Some(3),
            limit: Some(4),
        },
        GET,
        "/fapi/v3/income",
        FUT,
        SIGNER,
        &[
            ("symbol", "BTCUSDT"),
            ("incomeType", "FUNDING_FEE"),
            ("startTime", "2"),
            ("endTime", "3"),
            ("limit", "4"),
        ],
    );
    check(
        IncomeHistoryPage {
            symbol: Some("BTCUSDT".into()),
            income_type: Some("COMMISSION".into()),
            start_time: Some(2),
            end_time: Some(3),
            page: Some(1),
            limit: Some(100),
        },
        GET,
        "/fapi/v3/incomePage",
        FUT,
        SIGNER,
        &[
            ("symbol", "BTCUSDT"),
            ("incomeType", "COMMISSION"),
            ("startTime", "2"),
            ("endTime", "3"),
            ("page", "1"),
            ("limit", "100"),
        ],
    );
    check(
        LeverageBrackets {
            symbol: Some("BTCUSDT".into()),
        },
        GET,
        "/fapi/v3/leverageBrackets",
        FUT,
        SIGNER,
        &[("symbol", "BTCUSDT")],
    );
    check(
        ChangeLeverage {
            symbol: "BTCUSDT".into(),
            leverage: 20,
        },
        POST,
        "/fapi/v3/leverage",
        FUT,
        SIGNER,
        &[("symbol", "BTCUSDT"), ("leverage", "20")],
    );
    assert!(
        ChangeLeverage {
            symbol: "BTCUSDT".into(),
            leverage: 0
        }
        .params()
        .is_err()
    );
    assert!(
        ChangeLeverage {
            symbol: "BTCUSDT".into(),
            leverage: 126
        }
        .params()
        .is_err()
    );
    check(
        ChangeMarginType {
            symbol: "BTCUSDT".into(),
            margin_type: MarginType::Isolated,
        },
        POST,
        "/fapi/v3/marginType",
        FUT,
        SIGNER,
        &[("symbol", "BTCUSDT"), ("marginType", "ISOLATED")],
    );
    check(
        ModifyPositionMargin {
            symbol: "BTCUSDT".into(),
            amount: d("12.5"),
            position_side: Some(PositionSide::Long),
            change_type: 1,
            client_tran_id: Some("t1".into()),
        },
        POST,
        "/fapi/v3/positionMargin",
        FUT,
        SIGNER,
        &[
            ("symbol", "BTCUSDT"),
            ("amount", "12.5"),
            ("positionSide", "LONG"),
            ("type", "1"),
            ("clientTranId", "t1"),
        ],
    );
    assert!(
        ModifyPositionMargin {
            symbol: "BTCUSDT".into(),
            amount: d("1"),
            position_side: None,
            change_type: 3,
            client_tran_id: None,
        }
        .params()
        .is_err()
    );
    check(
        PositionMarginHistory {
            symbol: "BTCUSDT".into(),
            change_type: Some(2),
            start_time: Some(1),
            end_time: Some(2),
            limit: Some(3),
        },
        GET,
        "/fapi/v3/positionMargin/history",
        FUT,
        SIGNER,
        &[
            ("symbol", "BTCUSDT"),
            ("type", "2"),
            ("startTime", "1"),
            ("endTime", "2"),
            ("limit", "3"),
        ],
    );
    check(
        PositionMode,
        GET,
        "/fapi/v3/positionSide/dual",
        FUT,
        SIGNER,
        &[],
    );
    check(
        ChangePositionMode {
            dual_side_position: true,
        },
        POST,
        "/fapi/v3/positionSide/dual",
        FUT,
        SIGNER,
        &[("dualSidePosition", "true")],
    );
    check(StpModeQuery, GET, "/fapi/v3/stpMode", FUT, SIGNER, &[]);
    check(
        ChangeStpMode {
            stp_mode: StpMode::ExpireBoth,
        },
        POST,
        "/fapi/v3/stpMode",
        FUT,
        SIGNER,
        &[("stpMode", "EXPIRE_BOTH")],
    );
    check(
        MultiAssetsMode,
        GET,
        "/fapi/v3/multiAssetsMargin",
        FUT,
        SIGNER,
        &[],
    );
    check(
        ChangeMultiAssetsMode {
            multi_assets_margin: false,
        },
        POST,
        "/fapi/v3/multiAssetsMargin",
        FUT,
        SIGNER,
        &[("multiAssetsMargin", "false")],
    );
    check(
        AssetExchange,
        POST,
        "/fapi/v3/assetExchange",
        FUT,
        SIGNER,
        &[],
    );
    check(
        AdlQuantileQuery {
            symbol: Some("BTCUSDT".into()),
        },
        GET,
        "/fapi/v3/adlQuantile",
        FUT,
        SIGNER,
        &[("symbol", "BTCUSDT")],
    );
    check(
        ForceOrders {
            symbol: Some("BTCUSDT".into()),
            auto_close_type: Some("LIQUIDATION".into()),
            start_time: Some(1),
            end_time: Some(2),
            limit: Some(3),
        },
        GET,
        "/fapi/v3/forceOrders",
        FUT,
        SIGNER,
        &[
            ("symbol", "BTCUSDT"),
            ("autoCloseType", "LIQUIDATION"),
            ("startTime", "1"),
            ("endTime", "2"),
            ("limit", "3"),
        ],
    );
    check(
        CommissionRateQuery {
            symbol: "BTCUSDT".into(),
        },
        GET,
        "/fapi/v3/commissionRate",
        FUT,
        SIGNER,
        &[("symbol", "BTCUSDT")],
    );
    check(
        WalletTransfer {
            amount: d("10"),
            asset: "USDT".into(),
            client_tran_id: "t1".into(),
            kind: TransferKind::SpotFuture,
        },
        POST,
        "/fapi/v3/asset/wallet/transfer",
        FUT,
        SIGNER,
        &[
            ("amount", "10"),
            ("asset", "USDT"),
            ("clientTranId", "t1"),
            ("kindType", "SPOT_FUTURE"),
        ],
    );
    assert!(
        WalletTransfer {
            amount: d("10"),
            asset: "USDT".into(),
            client_tran_id: String::new(),
            kind: TransferKind::SpotFuture,
        }
        .params()
        .is_err()
    );
    check(
        DirectAnnouncements {
            page: Some(1),
            size: Some(20),
        },
        GET,
        "/fapi/v3/announcement/direct",
        FUT,
        SIGNER,
        &[("page", "1"), ("size", "20")],
    );
    check(
        DirectAnnouncementById { id: 9 },
        GET,
        "/fapi/v3/announcement/directById",
        FUT,
        SIGNER,
        &[("id", "9")],
    );
}

#[test]
fn agent_and_builder_admin_endpoints() {
    use aster_client::futures::*;

    check(AgentList, GET, "/fapi/v3/agent", FUT, SIGNER, &[]);

    let request = RegisterAgentRequest {
        user: "0x63DD5aCC6b1aa0f563956C0e534DD30B6dcF7C4e"
            .parse()
            .unwrap(),
        agent_name: "bot".into(),
        agent_address: "0x14791697260e4c9a71f18484c9f997b308e59325"
            .parse()
            .unwrap(),
        expired: 1_900_000_000_000,
        can_spot_trade: true,
        can_perp_trade: true,
        can_withdraw: false,
        ip_whitelist: Some("10.0.0.0/24".into()),
        agent_code: Some("code".into()),
        signature_chain_id: 56,
    };
    let registration = RegisterAndApproveAgent::sign(&signer(), &request).unwrap();
    meta(
        &registration,
        POST,
        "/fapi/v3/registerAndApproveAgent",
        FUT,
        NONE,
    );
    let params = registration.params().unwrap();
    assert_eq!(params.get("agentName"), Some("bot"));
    assert_eq!(params.get("canSpotTrade"), Some("true"));
    assert_eq!(params.get("ipWhitelist"), Some("10.0.0.0/24"));
    assert_eq!(params.get("signatureChainId"), Some("56"));
    assert!(params.get("signature").unwrap().starts_with("0x"));

    let mut risky = request.clone();
    risky.can_withdraw = true;
    risky.ip_whitelist = None;
    assert!(RegisterAndApproveAgent::sign(&signer(), &risky).is_err());

    let approve = ApproveBuilder::sign(
        &signer(),
        "0xbuilder",
        "0.0001",
        "acme",
        AsterChain::Mainnet,
        7,
    )
    .unwrap();
    meta(&approve, POST, "/fapi/v3/approveBuilder", FUT, NONE);
    assert_eq!(approve.params().unwrap().get("maxFeeRate"), Some("0.0001"));
    assert_eq!(approve.params().unwrap().get("signatureChainId"), Some("56"));

    let update =
        UpdateBuilder::sign(&signer(), "0xbuilder", "0.0002", AsterChain::Mainnet, 8).unwrap();
    meta(&update, POST, "/fapi/v3/updateBuilder", FUT, NONE);
    assert_eq!(update.params().unwrap().get("builder"), Some("0xbuilder"));
    assert_eq!(update.params().unwrap().get("signatureChainId"), Some("56"));

    let del = DelBuilder::sign(&signer(), "0xbuilder", AsterChain::Mainnet, 9).unwrap();
    meta(&del, DELETE, "/fapi/v3/builder", FUT, NONE);
    assert_eq!(del.params().unwrap().get("nonce"), Some("9"));
    assert_eq!(del.params().unwrap().get("signatureChainId"), Some("56"));
}

#[test]
fn builder_query_endpoints() {
    use aster_client::futures::*;

    let query = || BuilderQuery {
        user_addresses: vec!["0xa".into(), "0xb".into()],
        start_time: Some(1),
        end_time: Some(2),
        page: Some(3),
        limit: Some(4),
    };
    let common = [
        ("userAddresses", "0xa,0xb"),
        ("startTime", "1"),
        ("endTime", "2"),
        ("page", "3"),
        ("limit", "4"),
    ];

    let accounts = BuilderUserAccounts {
        query: query(),
        symbol: Some("BTCUSDT".into()),
    };
    meta(&accounts, GET, "/fapi/v3/builder/userAccounts", FUT, SIGNER);
    let params = accounts.params().unwrap();
    assert_eq!(params.get("symbol"), Some("BTCUSDT"));
    assert_eq!(params.get("userAddresses"), Some("0xa,0xb"));

    check(
        BuilderUserOpenOrders { query: query() },
        GET,
        "/fapi/v3/builder/userOpenOrders",
        FUT,
        SIGNER,
        &common,
    );
    check(
        BuilderUserBalances { query: query() },
        GET,
        "/fapi/v3/builder/userBalances",
        FUT,
        SIGNER,
        &common,
    );
    check(
        BuilderUserPositions { query: query() },
        GET,
        "/fapi/v3/builder/userPositionRisk",
        FUT,
        SIGNER,
        &common,
    );
    check(
        BuilderUserCommissionRates { query: query() },
        GET,
        "/fapi/v3/builder/userCommissionRates",
        FUT,
        SIGNER,
        &common,
    );
    check(
        BuilderUserTrades { query: query() },
        GET,
        "/fapi/v3/builder/userTrades",
        FUT,
        SIGNER,
        &common,
    );
    check(
        BuilderUserCommissions { query: query() },
        GET,
        "/fapi/v3/builder/userCommissions",
        FUT,
        SIGNER,
        &common,
    );
    check(
        BuilderUserAllOrders { query: query() },
        GET,
        "/fapi/v3/builder/userAllOrders",
        FUT,
        SIGNER,
        &common,
    );
    check(
        BuilderApprovedUsers { query: query() },
        GET,
        "/fapi/v3/builder/approvedUserList",
        FUT,
        SIGNER,
        &common,
    );

    // Empty query: no parameters, still a valid request.
    check(
        BuilderUserCommissionRates::default(),
        GET,
        "/fapi/v3/builder/userCommissionRates",
        FUT,
        SIGNER,
        &[],
    );
}

#[test]
fn funding_endpoints() {
    use aster_client::funding::*;

    check(
        EstimateWithdrawFee {
            chain: WithdrawChain::Bsc,
            asset: "USDT".into(),
        },
        GET,
        "/api/v3/aster/withdraw/estimateFee",
        SPOT,
        NONE,
        &[("chainId", "56"), ("asset", "USDT")],
    );

    let intent = WithdrawIntent {
        chain: WithdrawChain::Arbitrum,
        asset: "USDT".into(),
        amount: d("10.5"),
        fee: d("1"),
        receiver: "0xD9cA6952F1b1349d27f91E4fa6FB8ef67b89F02d"
            .parse()
            .unwrap(),
    };
    let signed = intent.sign(&signer(), Some(99)).unwrap();
    let signature = signed.signature.to_hex();

    let spot = SpotWithdraw(signed.clone());
    check(
        spot,
        POST,
        "/api/v3/aster/user-withdraw",
        SPOT,
        NONE,
        &[
            ("chainId", "42161"),
            ("asset", "USDT"),
            ("amount", "10.5"),
            ("fee", "1"),
            ("receiver", "0xd9ca6952f1b1349d27f91e4fa6fb8ef67b89f02d"),
            ("userNonce", "99"),
            ("userSignature", &signature),
        ],
    );
    meta(
        &FuturesWithdraw(signed),
        POST,
        "/fapi/v3/aster/user-withdraw",
        FUT,
        NONE,
    );
    check(
        DepositWithdrawHistory,
        POST,
        "/fapi/v3/aster/deposit-withdraw-history",
        FUT,
        SIGNER,
        &[],
    );
    check(
        WithdrawInfo,
        POST,
        "/fapi/v3/aster/user-withdraw-info",
        FUT,
        SIGNER,
        &[],
    );

    // Chain metadata and calldata.
    assert_eq!(WithdrawChain::Ethereum.chain_id(), 1);
    assert_eq!(WithdrawChain::Bsc.as_str(), "BSC");
    assert_eq!(deposit::DepositChain::Ethereum.chain_id(), 1);
    assert_eq!(deposit::DepositChain::Bsc.chain_id(), 56);
    assert_eq!(deposit::DepositChain::Arbitrum.chain_id(), 42_161);
    assert_ne!(
        deposit::DepositChain::Ethereum.vault(),
        deposit::DepositChain::Bsc.vault()
    );
    assert_eq!(
        deposit::approve_calldata(alloy_primitives::Address::ZERO, 5).len(),
        4 + 64
    );
    assert_eq!(AsterChain::Testnet.to_string(), "Testnet");
    assert_eq!(aster_client::futures::QuantityUnit::Base.as_str(), "BASE");
    assert_eq!(
        aster_client::futures::ChaseOffsetType::Percentage.as_str(),
        "PERCENTAGE"
    );
}
