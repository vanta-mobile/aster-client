# aster-client：Aster DEX V3 Rust 客户端

基于 [Aster 官方 API 文档](https://github.com/asterdex/api-docs) 实现，覆盖合约（`/fapi/v3`）、现货（`/api/v3`）、出入金、Builder/Agent 管理、WebSocket 与 Aster Chain JSON-RPC。

```toml
[dependencies]
aster-client = "0.1"
```

```rust
use aster_client::{Client, Environment};
```

API docs: <https://docs.rs/aster-client>. Every public item is documented and
`missing_docs` is enforced (`cargo clippy -- -D warnings`).

## 设计

| 模块 | 内容 |
|---|---|
| `client` | `Client<T, S>`：泛型 HTTP 传输 + 泛型签名器；`Endpoint` trait 描述每个接口（method / path / venue / security / 参数 / 响应类型） |
| `auth` | EIP-712（`Message` 与动态 struct）、secp256k1 签名、单调微秒 nonce、Builder/提现 typed-data |
| `params` | 参数收集与规范编码；**签名串 == 发送串**（ASCII 排序 + form-urlencoded），`signature` 只追加在末尾 |
| `futures` | 行情、下单/改单/撤单/批量/追单/策略单、账户、持仓、Agent、Builder 查询 |
| `spot` | 行情、下单/撤单/查询、账户、划转、Builder 授权 |
| `funding` | 提现签名与提交、估费、出入金历史、充值合约 calldata |
| `ws` | 组合流 URL 构建、连接、强类型事件解码、重连监督器与 listenKey 生命周期 |
| `limits` | 按 `Endpoint::weight()` 的令牌桶，客户端可选接入，提前避开 429 |
| `rpc` | `aster_getBalance` 等只读 JSON-RPC |

关键类型：

- `Client<ReqwestTransport>`：只读；`.signer(EvmSigner)` 后具备签名能力；`.with_user(Address)` 配置主钱包（顺序无关）。
- 交易类 `Endpoint` 的 `SECURITY = Signer`，只读客户端调用会得到 `Error::MissingCredentials`。
- 金额/数量用 `rust_decimal::Decimal`；下单构造器接受 `Decimal`、整数或字符串（`NewOrder::limit(..., "1.5", "200.25")?`），解析失败返回 `Error::InvalidParam`。
- `SymbolInfo::tick_size()/step_size()/min_notional()/quantize_price()/quantize_qty()` 直接读 `exchangeInfo.filters`，不猜 `pricePrecision`。

## 快速开始

```rust,no_run
use aster_client::{Client, Environment, futures, model::Side, model::TimeInForce};

# async fn demo() -> aster_client::Result<()> {
// 只读
let public = Client::new(Environment::Mainnet);
let klines = public
    .send(futures::Klines::new("BTCUSDT", aster_client::model::KlineInterval::Min1))
    .await?;

// 签名（Agent 私钥 + 主钱包地址）
let trading = public
    .signer(aster_client::auth::EvmSigner::from_hex("0x<agent-private-key>")?)
    .with_user("0x<master-wallet>".parse().unwrap());

let order = trading
    .send(
        futures::NewOrder::limit("AAPLUSDT", Side::Buy, "1.5", "200.25")?
            .time_in_force(TimeInForce::Gtc)
            .client_order_id("a1b2"),
    )
    .await?;
println!("{} {:?}", order.order_id, order.status);
# Ok(()) }
```

`send` 只接受实现 `Endpoint` 的类型；每个结构体自带参数并做本地前置校验（例如 `LIMIT` 缺 `timeInForce`、`closePosition` 与 `quantity` 冲突、`builder` 无 `feeRate`、`newClientOrderId` 字符集），把 `-1102/-1106` 一类错误挡在发送前。

## 签名

- 普通请求：`EIP-712(domain=AsterSignTransaction, chainId=1666/714, type=Message(string msg))`，`msg` 是最终 form-encoded 串（不含 `signature`）。
- 提现：`domain=Aster, chainId=目标链`，`type=Action`（字段含空格，如 `destination Chain`）。
- Builder 授权：`ApproveBuilder/UpdateBuilder/DelBuilder`，域 `chainId=56`。
- `registerAndApproveAgent`：`Message(msg)`，域 `chainId=signatureChainId`（56/101）。
- nonce：微秒、按 signer 严格递增（`MicrosecondNonce`，原子 CAS）；多实例需自行实现 `NonceSource` 共享递增源。
- 冷启动与时钟偏差：`PersistentNonce<S: NonceStore>` 落盘高水位（默认每 1s 进度写一次，`flush()` 可强制），`Client::calibrate_time()` 拉 `/fapi/v3/time` 并把偏移注入 nonce（手机时钟偏差 >60s 时签名必失败）。
- 限流：`RateLimiter::from_rate_limits(&exchange_info.rate_limits)` + `Client::rate_limiter(Arc<..>)`，`send` 前按端点权重等待。
- WebSocket：`Supervisor` 掉线后下一次 `next()` 自动重连；掉线先返回一次 `Err`，用于触发盘口/账户重同步。
  - 每次连接（含首次失败后的重试）都先等待退避：1s 起、翻倍至 60s、再加 0–25% 抖动；单次连接 10s 超时。Aster 对重连过快的 IP 直接 `Connection reset`，所以不存在任何紧循环重试路径。
  - 23h 主动轮换；无事件 3min 发 WebSocket Ping 保活；`close()` 走优雅关闭。
  - `subscribe()/unsubscribe()` 在已连接时发 `SUBSCRIBE/UNSUBSCRIBE`（切交易对不用重连），并同步进重连 URL，掉线后自动恢复；单连接上限 200 流。固定 URL 模式（`Supervisor::new`）拒绝这类命令。
  - `UserStream` 在其上做 listenKey 30min 保活 + `listenKeyExpired` 换 key 重连。
- Agent/Builder 管理类其它字段（如 `approveAgent/updateAgent/delAgent`）的 typed-data 布局文档未给出，用 `Client::send_typed_signed` 传入 `auth::Struct` 即可，不做猜测。

签名测试向量（与 Python `eth_account` 交叉验证）在 `tests/signing.rs`；`cargo test` 同时校验参考向量 `0x014c…481c`。

## 错误处理

```rust,no_run
# async fn demo(c: aster_client::Client) {
match c.send(futures::CancelOrder::by_client_id("BTCUSDT", "a1b2")).await {
    Ok(order) => { /* ... */ }
    Err(e) if e.is_execution_unknown() => { /* HTTP 5xx：用 clientOrderId 查单，不要盲目重试 */ }
    Err(e) if e.is_rate_limited() => { /* 429/418：按 retry_after 退避 */ }
    Err(e) => eprintln!("{e} / code={:?}", e.code()),
}
# }
```

- `503`/5xx → `Error::UnknownState { status, .. }`：执行状态未知，先查单。
- `429`/`418` → `Error::RateLimited { retry_after, .. }`。
- 业务错误（含 HTTP 200 的 `failureCode`）→ `Error::Api(ApiError { code, msg })`；策略单还需检查 `StrategyOrderResult::rejection()`。

## 覆盖与边界

已覆盖：

- 合约：全部行情、下单/改单/撤单/守卫撤单/批量/全撤/倒计时全撤/追单/策略单、账户/持仓/杠杆/保证金/保证金历史、划转、`noop`、listenKey、公告、Agent 注册与查询、Builder 授权与 9 个 Builder 维度查询。
- 现货：行情、下单/撤单/查询、账户、划转、listenKey、Builder 授权/查询。
- 资金：提现估费/提交（两套签名域）、出入金历史、EVM 充值 calldata（`depositFor`/`approve`）、Aster Chain 只读 RPC。
- WebSocket：组合/原始流 URL、市场事件与用户数据事件（`ACCOUNT_UPDATE`、`ORDER_TRADE_UPDATE`、`MARGIN_CALL`、`listenKeyExpired`、公告等）。

明确未做（文档缺失或不属于 API 封装）：

- `approveAgent/updateAgent/delAgent` 的 typed-data 字段布局（用 `send_typed_signed` 自行传入）。
- 子账户 5 个端点（`bind`/`createSubAccount`/`getSubAccountList`/`updateSubAccount`/`subAccountTransfer`）：需要双签名与文档字面顺序的签名串，`Client` 单签名 + `Params::encode()` 排序都不满足。
- 现货 `POST /api/v3/batchOrders`（官方文档未给出参数）。
- 本地订单簿对齐：重连后需调用方拉 `depth` 快照并按 `U/u/pu` 对齐，`Supervisor` 只负责连接。
- 美股/商品交易时段日历：API 不返回该信息。
- `ORDERS`（1200 单/分钟）预算：`RateLimiter` 只按 weight 计，下单次数需调用方自己控。

## 示例（`examples/`）

| 示例 | 说明 | 运行 |
|---|---|---|
| `public_market_data` | 只读行情：`exchangeInfo` 过滤器、量化、盘口、K 线、ticker、现货盘口 | `cargo run --example public_market_data` |
| `trade_screen` | Pro 交易页的全部冷启动数据 + 页面同款 WS 流 URL | `cargo run --example trade_screen` |
| `signed_order` | 下单参数本地校验、规范签名串；默认 dry-run，`ASTER_SEND=1` 才提交 | `cargo run --example signed_order` |
| `builder_fees` | Builder 归属下单（`builder`+`feeRate`+`user`）与 9 个 Builder 查询 | `cargo run --example builder_fees` |
| `websocket` | 组合流 URL、连接、强类型事件解码（打印 10 条后退出） | `cargo run --example websocket` |
| `wallet` | 提现 `Action` 签名、出入金 calldata、可选 Aster Chain RPC | `cargo run --example wallet` |
| `error_handling` | 503 未知状态 / HTTP 200 业务错误 / 本地拒绝 / 429 退避（离线，注入 `Transport`） | `cargo run --example error_handling` |

除需要真实密钥的写操作外，示例均可直接运行；签名示例默认使用文档中的公开测试私钥，**不可用于真实资金**。

## 测试与发布检查

```sh
cargo test                                     # 单元 + 签名向量 + 文档示例
cargo test --test live -- --ignored            # 主网只读冒烟（ping/time/exchangeInfo/depth/klines/ticker/spot）
cargo clippy --all-targets -- -D warnings      # 含 missing_docs
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps # 无失效 intra-doc 链接
cargo package                                  # 打包并在临时目录构建校验
```

## 许可

MIT OR Apache-2.0，见 `LICENSE-MIT`、`LICENSE-APACHE`。
