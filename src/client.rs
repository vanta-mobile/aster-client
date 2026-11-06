//! HTTP transport, environment configuration and the generic request pipeline.

use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use alloy_primitives::Address;
use serde::de::DeserializeOwned;

use crate::auth::{
    Domain, MicrosecondNonce, NoSigner, NonceSource, Signature, Signer, message_digest, now_micros,
};
use crate::error::{ApiError, Error, Result};
use crate::limits::RateLimiter;
use crate::params::Params;

/// HTTP method used by an endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    /// HTTP GET.
    Get,
    /// HTTP POST.
    Post,
    /// HTTP PUT.
    Put,
    /// HTTP DELETE.
    Delete,
}

impl Method {
    /// Wire representation of this value.
    pub fn as_str(self) -> &'static str {
        match self {
            Method::Get => "GET",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Delete => "DELETE",
        }
    }
}

/// Mainnet or testnet. Selects hosts, WebSocket URLs and the request-signing
/// chain id (1666 / 714).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Environment {
    /// Mainnet.
    #[default]
    Mainnet,
    /// Testnet.
    Testnet,
}

impl Environment {
    /// Futures REST base URL.
    pub fn futures_rest(self) -> &'static str {
        match self {
            Environment::Mainnet => "https://fapi.asterdex.com",
            Environment::Testnet => "https://fapi.asterdex-testnet.com",
        }
    }

    /// Spot REST base URL.
    pub fn spot_rest(self) -> &'static str {
        match self {
            Environment::Mainnet => "https://sapi.asterdex.com",
            Environment::Testnet => "https://sapi.asterdex-testnet.com",
        }
    }

    /// Futures WebSocket base URL.
    pub fn futures_ws(self) -> &'static str {
        match self {
            Environment::Mainnet => "wss://fstream.asterdex.com",
            Environment::Testnet => "wss://fstream.asterdex-testnet.com",
        }
    }

    /// Spot WebSocket base URL.
    pub fn spot_ws(self) -> &'static str {
        match self {
            Environment::Mainnet => "wss://sstream.asterdex.com",
            Environment::Testnet => "wss://sstream.asterdex-testnet.com",
        }
    }

    /// Aster-Chain REST base URL (`/aster-chain/v3`).
    ///
    /// Differs from futures/spot: it is served by `chainapi.*`, not `fapi.*`.
    pub fn chain_rest(self) -> &'static str {
        match self {
            Environment::Mainnet => "https://chainapi.asterdex.com",
            Environment::Testnet => "https://chainapi.asterdex-testnet.com",
        }
    }

    /// EIP-712 chain id for request signatures.
    pub fn sign_chain_id(self) -> u64 {
        match self {
            Environment::Mainnet => 1666,
            Environment::Testnet => 714,
        }
    }
}

/// Which exchange surface an endpoint belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Venue {
    /// Futures (`/fapi/v3`).
    Futures,
    /// Spot (`/api/v3`).
    Spot,
    /// Aster-Chain (`/aster-chain/v3`), served by `chainapi.*`.
    Chain,
}

impl Venue {
    /// REST base URL for this venue.
    pub fn base_url(self, environment: Environment) -> &'static str {
        match self {
            Venue::Futures => environment.futures_rest(),
            Venue::Spot => environment.spot_rest(),
            Venue::Chain => environment.chain_rest(),
        }
    }
}

/// How a request is authenticated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Security {
    /// Public market data; no credentials required.
    None,
    /// Agent wallet: `signer` + `nonce`, signed with the agent key.
    Signer,
    /// Master wallet: `user` + `nonce`, signed with the master key.
    User,
}

/// A typed REST endpoint.
///
/// Implementations are tiny structs that own their parameters:
///
/// ```ignore
/// let depth = client.send(Depth { symbol: "BTCUSDT".into(), limit: Some(5) }).await?;
/// ```
pub trait Endpoint {
    /// Decoded success payload.
    type Response: DeserializeOwned;

    /// HTTP method.
    const METHOD: Method;
    /// Full path, e.g. `/fapi/v3/depth`.
    const PATH: &'static str;
    /// Exchange surface that serves the path.
    const VENUE: Venue;
    /// Authentication the endpoint requires.
    const SECURITY: Security = Security::None;

    /// Validate and encode the request parameters.
    fn params(&self) -> Result<Params> {
        Ok(Params::new())
    }

    /// Documented request weight, useful for local rate-limit accounting.
    fn weight(&self) -> u32 {
        1
    }

    /// Reuse a caller-supplied nonce instead of a fresh one. Guarded cancels
    /// must sign with the nonce the order was placed with.
    fn nonce_override(&self) -> Option<u64> {
        None
    }
}

/// A transport-level request, already encoded and signed.
#[derive(Debug, Clone)]
pub struct HttpRequest {
    /// HTTP method.
    pub method: Method,
    /// Request URL.
    pub url: String,
    /// Request URL.
    /// Body for POST/PUT/DELETE.
    pub body: Option<String>,
    /// Content Type.
    /// Encoded request body.
    /// Defaults to `application/x-www-form-urlencoded` when a body is present.
    pub content_type: Option<&'static str>,
}

/// Raw transport response.
#[derive(Debug, Clone)]
pub struct HttpResponse {
    /// HTTP status code.
    pub status: u16,
    /// Encoded request body.
    pub body: String,
    /// Server-provided backoff.
    pub retry_after: Option<Duration>,
}

/// Pluggable HTTP backend: implement this to run over a different stack
/// (proxy, wasm, replaying test harness).
pub trait Transport: Send + Sync {
    /// Execute one already-encoded, already-signed request.
    fn execute(&self, request: HttpRequest) -> impl Future<Output = Result<HttpResponse>> + Send;
}

/// Default `reqwest` transport (rustls).
#[derive(Debug, Clone)]
pub struct ReqwestTransport {
    client: reqwest::Client,
}

impl Default for ReqwestTransport {
    fn default() -> Self {
        Self::new(reqwest::Client::new())
    }
}

impl ReqwestTransport {
    /// Create a new value.
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

impl Transport for ReqwestTransport {
    async fn execute(&self, request: HttpRequest) -> Result<HttpResponse> {
        let method = match request.method {
            Method::Get => reqwest::Method::GET,
            Method::Post => reqwest::Method::POST,
            Method::Put => reqwest::Method::PUT,
            Method::Delete => reqwest::Method::DELETE,
        };
        let mut builder = self.client.request(method, &request.url).header(
            reqwest::header::USER_AGENT,
            concat!("aster-rs/", env!("CARGO_PKG_VERSION")),
        );
        if let Some(body) = request.body {
            builder = builder.header(
                reqwest::header::CONTENT_TYPE,
                request
                    .content_type
                    .unwrap_or("application/x-www-form-urlencoded"),
            );
            builder = builder.body(body);
        }
        let response = builder
            .send()
            .await
            .map_err(|e| Error::Transport(e.to_string()))?;
        let status = response.status().as_u16();
        let retry_after = response
            .headers()
            .get(reqwest::header::RETRY_AFTER)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.trim().parse::<u64>().ok())
            .map(Duration::from_secs);
        let body = response
            .text()
            .await
            .map_err(|e| Error::Transport(e.to_string()))?;
        Ok(HttpResponse {
            status,
            body,
            retry_after,
        })
    }
}

/// Aster REST client, generic over the transport and the signing key.
///
/// ```no_run
/// # use aster_client::{Client, Environment};
/// # async fn demo() -> aster_client::Result<()> {
/// let read_only = Client::new(Environment::Mainnet);
/// let trading = read_only
///     .signer(aster_client::auth::EvmSigner::from_hex("0x…")?)
///     .with_user("0x63DD5aCC6b1aa0f563956C0e534DD30B6dcF7C4e".parse().unwrap());
/// # Ok(()) }
/// ```
pub struct Client<T = ReqwestTransport, S = NoSigner> {
    transport: T,
    environment: Environment,
    signer: Option<S>,
    /// Master wallet address, sent as `user` by `Security::User` endpoints.
    user: Option<Address>,
    nonce: Arc<dyn NonceSource>,
    limiter: Option<Arc<RateLimiter>>,
}

impl Client<ReqwestTransport, NoSigner> {
    /// Public (unauthenticated) client for the given environment.
    pub fn new(environment: Environment) -> Self {
        Self::with_transport(environment, ReqwestTransport::default())
    }
}

impl<T: Transport> Client<T, NoSigner> {
    /// Public client over a custom transport.
    pub fn with_transport(environment: Environment, transport: T) -> Self {
        Client {
            transport,
            environment,
            signer: None,
            user: None,
            nonce: Arc::new(MicrosecondNonce::new()),
            limiter: None,
        }
    }
}

impl<T: Transport, S: Signer> Client<T, S> {
    /// Address of the attached signing key.
    pub fn signer_address(&self) -> Option<Address> {
        self.signer.as_ref().map(Signer::address)
    }

    /// Whether a signing key is attached.
    pub fn has_credentials(&self) -> bool {
        self.signer.is_some()
    }
}

impl<T: Transport, S> Client<T, S> {
    /// environment (mainnet or testnet).
    pub fn environment(&self) -> Environment {
        self.environment
    }

    /// hTTP transport.
    pub fn transport(&self) -> &T {
        &self.transport
    }

    /// Replace the request-signing source (defaults to wall-clock microseconds).
    pub fn nonce_source(mut self, nonce: Arc<dyn NonceSource>) -> Self {
        self.nonce = nonce;
        self
    }

    /// Throttle outgoing requests by their documented [`Endpoint::weight`].
    /// Build the limiter from `exchangeInfo.rateLimits`
    /// ([`RateLimiter::from_rate_limits`]); without one, requests are not paced.
    pub fn rate_limiter(mut self, limiter: Arc<RateLimiter>) -> Self {
        self.limiter = Some(limiter);
        self
    }

    /// Attach a signing key. The key's address becomes `signer`.
    pub fn signer<S2: Signer>(mut self, signer: S2) -> Client<T, S2> {
        Client {
            transport: self.transport,
            environment: self.environment,
            signer: Some(signer),
            user: self.user.take(),
            nonce: self.nonce,
            limiter: self.limiter.take(),
        }
    }

    /// Attach the master wallet address, sent as `user` by endpoints that
    /// authenticate with the master key. May be called before or after
    /// [`Client::signer`].
    pub fn with_user(mut self, user: Address) -> Client<T, S> {
        self.user = Some(user);
        self
    }

    /// Like [`Client::with_user`], for optional configuration.
    pub fn with_optional_user(mut self, user: Option<Address>) -> Client<T, S> {
        if user.is_some() {
            self.user = user;
        }
        self
    }
}

impl<T: Transport, S: Signer> Client<T, S> {
    /// Send a typed request.
    pub async fn send<E: Endpoint>(&self, endpoint: E) -> Result<E::Response> {
        if let Some(limiter) = &self.limiter {
            limiter.acquire(endpoint.weight()).await;
        }
        let mut params = endpoint.params()?;
        let signed = self.authorize::<E>(&mut params, endpoint.nonce_override())?;
        let request = self.build_request::<E>(params, signed);
        let response = self.transport.execute(request).await?;
        decode::<E::Response>(response)
    }

    /// Align request nonces with the exchange clock.
    ///
    /// Call once after startup and after a long background pause: a device
    /// clock more than 60s away from the server makes every signed request
    /// fail. Returns the offset applied to later nonces, in microseconds
    /// (exchange time minus local time).
    pub async fn calibrate_time(&self) -> Result<i64> {
        let before = now_micros() as i64;
        let server = self.send(crate::futures::ServerTime).await?.server_time;
        let after = now_micros() as i64;
        let offset = server
            .saturating_mul(1_000)
            .saturating_sub(before + (after - before) / 2);
        self.nonce.set_time_offset(offset);
        Ok(offset)
    }

    /// Send an endpoint whose signature is a standalone request parameter
    /// computed over an arbitrary EIP-712 struct (agent administration and
    /// any future management endpoint).
    ///
    /// `params` must already contain every field the signature covers — plus
    /// `nonce` and `user`/`signer` — because Aster verifies the signature
    /// against the typed data, not against the encoded parameter string.
    /// The signature is appended last.
    pub async fn send_typed_signed<U: DeserializeOwned>(
        &self,
        method: Method,
        venue: Venue,
        path: &'static str,
        mut params: Params,
        domain: &crate::auth::Domain,
        payload: &crate::auth::Struct<'_>,
    ) -> Result<U> {
        let signer = self.signer.as_ref().ok_or(Error::MissingCredentials)?;
        let signature = signer.sign_digest(payload.digest(domain))?;
        params.set_str("signature", signature.to_hex());
        let url = format!("{}{}", venue.base_url(self.environment), path);
        let encoded = params.encode();
        let request = match method {
            Method::Get => HttpRequest {
                method,
                url: format!("{url}?{encoded}"),
                body: None,
                content_type: None,
            },
            method => HttpRequest {
                method,
                url,
                body: Some(encoded),
                content_type: None,
            },
        };
        decode::<U>(self.transport.execute(request).await?)
    }

    /// Adds `nonce` plus `signer`/`user` to the params and signs the canonical
    /// parameter string.
    pub(crate) fn sign_payload(
        &self,
        params: &mut Params,
        security: Security,
        nonce_override: Option<u64>,
    ) -> Result<Signature> {
        let signer = self.signer.as_ref().ok_or(Error::MissingCredentials)?;
        params.set(
            "nonce",
            &nonce_override.unwrap_or_else(|| self.nonce.next()),
        );
        match security {
            Security::None => return Err(Error::MissingCredentials),
            Security::Signer => {
                params.set("signer", &signer.address());
            }
            Security::User => {
                let user = self.user.ok_or(Error::MissingCredentials)?;
                params.set("user", &user);
            }
        }
        let payload = params.encode();
        let domain = Domain::aster_sign(self.environment.sign_chain_id());
        signer.sign_digest(message_digest(&domain, &payload))
    }

    fn authorize<E: Endpoint>(
        &self,
        params: &mut Params,
        nonce_override: Option<u64>,
    ) -> Result<Option<Signature>> {
        match E::SECURITY {
            Security::None => Ok(None),
            security => self
                .sign_payload(params, security, nonce_override)
                .map(Some),
        }
    }

    fn build_request<E: Endpoint>(
        &self,
        params: Params,
        signature: Option<Signature>,
    ) -> HttpRequest {
        let mut url = String::with_capacity(96);
        url.push_str(E::VENUE.base_url(self.environment));
        url.push_str(E::PATH);

        let encoded = match signature {
            Some(signature) => params.encode_signed(&signature.to_hex()),
            None => params.encode(),
        };

        match E::METHOD {
            Method::Get => {
                if !encoded.is_empty() {
                    url.push('?');
                    url.push_str(&encoded);
                }
                HttpRequest {
                    method: E::METHOD,
                    url,
                    body: None,
                    content_type: None,
                }
            }
            method => HttpRequest {
                method,
                url,
                body: Some(encoded),
                content_type: None,
            },
        }
    }
}

/// Extract `{code, msg}` when `code` is not 200; `None` for success shapes.
fn api_error_in_body(body: &str) -> Option<ApiError> {
    let value: serde_json::Value = serde_json::from_str(body).ok()?;
    let code = value.get("code")?;
    let code = match code {
        serde_json::Value::Number(n) => n.as_i64()? as i32,
        serde_json::Value::String(s) => s.parse().ok()?,
        _ => return None,
    };
    if code == 200 {
        return None;
    }
    Some(ApiError::new(
        code,
        value
            .get("msg")
            .and_then(|m| m.as_str())
            .unwrap_or_default(),
    ))
}

fn decode<T: DeserializeOwned>(response: HttpResponse) -> Result<T> {
    let status = response.status;
    let body = if response.body.trim().is_empty() {
        "null".to_owned()
    } else {
        response.body
    };

    if (200..300).contains(&status) {
        // Aster reports business errors with HTTP 200 (and `-5050` may arrive
        // that way). A `code` other than 200 always wins over the success type.
        if let Some(api_error) = api_error_in_body(&body) {
            return Err(Error::Api(api_error));
        }
        match serde_json::from_str::<T>(&body) {
            Ok(value) => return Ok(value),
            Err(decode_error) => {
                if let Ok(api_error) = serde_json::from_str::<ApiError>(&body) {
                    return Err(Error::Api(api_error));
                }
                return Err(Error::Decode(decode_error));
            }
        }
    }

    match status {
        429 | 418 => Err(Error::RateLimited {
            status,
            retry_after: response.retry_after,
            message: body,
        }),
        500..=599 => Err(Error::UnknownState {
            status,
            message: body,
        }),
        _ => match serde_json::from_str::<ApiError>(&body) {
            Ok(api_error) => Err(Error::Api(api_error)),
            Err(_) => Err(Error::Http { status, body }),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeTransport {
        response: HttpResponse,
        seen: std::sync::Mutex<Vec<HttpRequest>>,
    }

    impl FakeTransport {
        fn new(body: &str) -> Self {
            Self {
                response: HttpResponse {
                    status: 200,
                    body: body.to_owned(),
                    retry_after: None,
                },
                seen: Default::default(),
            }
        }
    }

    impl Transport for FakeTransport {
        async fn execute(&self, request: HttpRequest) -> Result<HttpResponse> {
            self.seen.lock().unwrap().push(request);
            Ok(self.response.clone())
        }
    }

    struct Ping;
    impl Endpoint for Ping {
        type Response = serde_json::Value;
        const METHOD: Method = Method::Get;
        const PATH: &'static str = "/fapi/v3/ping";
        const VENUE: Venue = Venue::Futures;
    }

    #[tokio::test]
    async fn public_get_has_no_query_when_empty() {
        let client = Client::with_transport(Environment::Testnet, FakeTransport::new("{}"));
        let _ = client.send(Ping).await.unwrap();
        let seen = client.transport().seen.lock().unwrap();
        assert_eq!(
            seen[0].url,
            "https://fapi.asterdex-testnet.com/fapi/v3/ping"
        );
        assert!(seen[0].body.is_none());
    }
}
