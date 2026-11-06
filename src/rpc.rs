//! Aster Chain JSON-RPC (read-only), e.g. `https://tapi.asterdex.com/info`.
//!
//! Only the documented methods are wrapped; responses are returned as JSON
//! values because the provider does not publish their exact schema.

use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::client::{HttpRequest, Method, ReqwestTransport, Transport};
use crate::error::{Error, Result};

/// Default public read-only endpoint.
pub const MAINNET_RPC_URL: &str = "https://tapi.asterdex.com/info";

/// JSON-RPC 2.0 client for Aster Chain.
#[derive(Debug, Clone)]
pub struct RpcClient<T = ReqwestTransport> {
    transport: T,
    url: String,
    next_id: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

impl RpcClient<ReqwestTransport> {
    /// Create a new value.
    pub fn new(url: impl Into<String>) -> Self {
        Self::with_transport(url, ReqwestTransport::default())
    }

    /// Client for the public mainnet RPC endpoint.
    pub fn mainnet() -> Self {
        Self::new(MAINNET_RPC_URL)
    }
}

impl<T: Transport> RpcClient<T> {
    /// Client over a custom transport.
    pub fn with_transport(url: impl Into<String>, transport: T) -> Self {
        Self {
            transport,
            url: url.into(),
            next_id: Default::default(),
        }
    }

    /// Transport backing this client.
    pub fn transport(&self) -> &T {
        &self.transport
    }

    /// Raw JSON-RPC call.
    pub async fn call(&self, method: &str, params: Value) -> Result<Value> {
        let id = self
            .next_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            + 1;
        let payload = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });
        let request = HttpRequest {
            method: Method::Post,
            url: self.url.clone(),
            body: Some(serde_json::to_string(&payload)?),
            content_type: Some("application/json"),
        };
        let response = self.transport.execute(request).await?;
        if response.status != 200 {
            return Err(Error::Transport(format!(
                "rpc http {}: {}",
                response.status, response.body
            )));
        }
        let envelope: RpcEnvelope = serde_json::from_str(&response.body)?;
        if let Some(error) = envelope.error {
            return Err(Error::Api(crate::error::ApiError::new(
                error.code as i32,
                error.message,
            )));
        }
        envelope
            .result
            .ok_or_else(|| Error::Transport("empty rpc result".to_owned()))
    }

    /// Typed variant of [`RpcClient::call`].
    pub async fn call_as<U: DeserializeOwned>(&self, method: &str, params: Value) -> Result<U> {
        let value = self.call(method, params).await?;
        Ok(serde_json::from_value(value)?)
    }

    /// `aster_getBalance(address, blockTag)`.
    pub async fn get_balance(&self, address: &str, block_tag: &str) -> Result<Value> {
        self.call("aster_getBalance", serde_json::json!([address, block_tag]))
            .await
    }

    /// `aster_getOpenOrders(address, blockTag)`.
    pub async fn get_open_orders(&self, address: &str, block_tag: &str) -> Result<Value> {
        self.call(
            "aster_getOpenOrders",
            serde_json::json!([address, block_tag]),
        )
        .await
    }

    /// `aster_getSpotOpenOrders(address, blockTag)`.
    pub async fn get_spot_open_orders(&self, address: &str, block_tag: &str) -> Result<Value> {
        self.call(
            "aster_getSpotOpenOrders",
            serde_json::json!([address, block_tag]),
        )
        .await
    }

    /// `aster_getUserFuturesFills(address, blockTag)`.
    pub async fn get_user_futures_fills(&self, address: &str, block_tag: &str) -> Result<Value> {
        self.call(
            "aster_getUserFuturesFills",
            serde_json::json!([address, block_tag]),
        )
        .await
    }

    /// `aster_getUserSpotFills(address, blockTag)`.
    pub async fn get_user_spot_fills(&self, address: &str, block_tag: &str) -> Result<Value> {
        self.call(
            "aster_getUserSpotFills",
            serde_json::json!([address, block_tag]),
        )
        .await
    }
}

#[derive(serde::Deserialize)]
struct RpcEnvelope {
    result: Option<Value>,
    error: Option<RpcError>,
}

#[derive(serde::Deserialize)]
struct RpcError {
    code: i64,
    message: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::{HttpRequest, HttpResponse, Transport};
    use std::sync::Mutex;

    struct Fake {
        status: u16,
        body: String,
        seen: Mutex<Vec<HttpRequest>>,
    }

    impl Fake {
        fn ok(result: serde_json::Value) -> Self {
            Self {
                status: 200,
                body: serde_json::json!({"jsonrpc":"2.0","id":1,"result":result}).to_string(),
                seen: Mutex::new(Vec::new()),
            }
        }
    }

    impl Transport for Fake {
        async fn execute(&self, request: HttpRequest) -> Result<HttpResponse> {
            self.seen.lock().unwrap().push(request);
            Ok(HttpResponse {
                status: self.status,
                body: self.body.clone(),
                retry_after: None,
            })
        }
    }

    #[tokio::test]
    async fn call_sends_json_and_unwraps_result() {
        let fake = Fake::ok(serde_json::json!({"balance":"1"}));
        let client = RpcClient::with_transport("http://rpc.local", fake);
        let value = client
            .call("aster_getBalance", serde_json::json!(["0xabc", "latest"]))
            .await
            .unwrap();
        assert_eq!(value["balance"], "1");

        let seen = client.transport().seen.lock().unwrap();
        assert_eq!(seen[0].method, Method::Post);
        assert_eq!(seen[0].content_type, Some("application/json"));
        assert!(
            seen[0]
                .body
                .as_deref()
                .unwrap()
                .contains("aster_getBalance")
        );
    }

    #[tokio::test]
    async fn typed_calls_cover_all_documented_methods() {
        let client = RpcClient::with_transport("http://rpc.local", Fake::ok(serde_json::json!([])));
        assert!(client.get_balance("0xa", "latest").await.is_ok());
        assert!(client.get_open_orders("0xa", "latest").await.is_ok());
        assert!(client.get_spot_open_orders("0xa", "latest").await.is_ok());
        assert!(client.get_user_futures_fills("0xa", "latest").await.is_ok());
        assert!(client.get_user_spot_fills("0xa", "latest").await.is_ok());
        let ids: Vec<u32> = client
            .call_as("aster_x", serde_json::json!([]))
            .await
            .unwrap();
        assert!(ids.is_empty());
    }

    #[tokio::test]
    async fn rpc_error_and_http_status_are_errors() {
        let error_body = serde_json::json!({"jsonrpc":"2.0","id":1,"error":{"code":-32601,"message":"method not found"}});
        let client = RpcClient::with_transport(
            "http://rpc.local",
            Fake {
                status: 200,
                body: error_body.to_string(),
                seen: Mutex::new(Vec::new()),
            },
        );
        let error = client
            .call("aster_nope", serde_json::json!([]))
            .await
            .unwrap_err();
        assert_eq!(error.code(), Some(-32601));

        let client = RpcClient::with_transport(
            "http://rpc.local",
            Fake {
                status: 502,
                body: "bad gateway".to_owned(),
                seen: Mutex::new(Vec::new()),
            },
        );
        assert!(matches!(
            client.get_balance("0xa", "latest").await,
            Err(Error::Transport(_))
        ));

        // Empty result envelopes are rejected as well.
        let client = RpcClient::with_transport(
            "http://rpc.local",
            Fake {
                status: 200,
                body: "{\"jsonrpc\":\"2.0\",\"id\":1}".to_owned(),
                seen: Mutex::new(Vec::new()),
            },
        );
        assert!(
            client
                .call("aster_empty", serde_json::json!([]))
                .await
                .is_err()
        );
    }
}
