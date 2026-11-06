//! Funding: withdrawal signature, deposit calldata and Aster Chain RPC.
//!
//! ```sh
//! cargo run --example wallet
//! # optional on-chain read
//! ASTER_RPC_ADDRESS=0x… cargo run --example wallet
//! ```
//!
//! Withdrawals sign the `Action` struct in the `Aster` domain with the
//! **destination** chain id — not the 1666/714 used for trading. This example
//! signs with the public test-vector key; never use it with real funds.

use std::env;

use aster_client::Endpoint;
use aster_client::auth::{EvmSigner, WithdrawChain};
use aster_client::funding::{self, WithdrawIntent};
use aster_client::rpc::RpcClient;

const TEST_KEY: &str = "0x0123456789012345678901234567890123456789012345678901234567890123";

#[tokio::main]
async fn main() -> aster_client::Result<()> {
    let signer = EvmSigner::from_hex(
        &env::var("ASTER_WITHDRAW_KEY").unwrap_or_else(|_| TEST_KEY.to_owned()),
    )?;
    println!("account wallet: {}", signer.address());

    let intent = WithdrawIntent {
        chain: WithdrawChain::Bsc,
        asset: "USDT".to_owned(),
        amount: "10.123400".parse().unwrap(),
        fee: "1.234567891".parse().unwrap(),
        receiver: signer.address(),
    };
    let signed = intent.sign(&signer, None)?;
    println!("userNonce      {}", signed.nonce);
    println!("userSignature  {}", signed.signature);

    // What would be POSTed to both /fapi/v3/aster/user-withdraw and
    // /api/v3/aster/user-withdraw.
    let body = funding::SpotWithdraw(signed).params()?.encode();
    println!("request body   {body}");

    // EVM deposit: approve(vault, amount) then depositFor(token, user, amount, broker).
    // `BROKER_SPOT` credits the spot account; any other value credits futures.
    let amount = 1_000_000_000_000_000_000_u128; // 1 token, 18 decimals
    let calldata = funding::deposit::deposit_for_calldata(
        funding::deposit::NATIVE_CURRENCY,
        signer.address(),
        amount,
        funding::deposit::BROKER_SPOT,
    );
    println!("depositFor     0x{}", hex::encode(calldata));
    println!(
        "vault (BSC)    {}",
        funding::deposit::DepositChain::Bsc.vault()
    );

    if let Ok(address) = env::var("ASTER_RPC_ADDRESS") {
        let rpc = RpcClient::mainnet();
        let balance = rpc.get_balance(&address, "latest").await?;
        println!("rpc balance    {balance}");
    } else {
        println!("set ASTER_RPC_ADDRESS=0x… to query aster_getBalance");
    }
    Ok(())
}
