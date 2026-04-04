//! LLM backend abstraction — allows swapping between cc-sdk, Codex, or other providers.
//!
//! The trait is intentionally minimal: single-turn, text-in/text-out.
//! Our agent loop controls all tool execution; the LLM only does text analysis.

use anyhow::{Context, Result};

/// Token usage reported by the backend (if available).
#[derive(Debug, Clone, Default)]
pub struct TokenUsage {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
}

impl TokenUsage {
    /// Accumulate usage from another response.
    pub fn accumulate(&mut self, other: &TokenUsage) {
        self.input_tokens = match (self.input_tokens, other.input_tokens) {
            (Some(a), Some(b)) => Some(a + b),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        };
        self.output_tokens = match (self.output_tokens, other.output_tokens) {
            (Some(a), Some(b)) => Some(a + b),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        };
    }

    /// Total tokens (input + output), or 0 if unknown.
    pub fn total(&self) -> u64 {
        self.input_tokens.unwrap_or(0) + self.output_tokens.unwrap_or(0)
    }
}

/// Response from an LLM completion.
#[derive(Debug, Clone)]
pub struct LlmResponse {
    pub text: String,
    pub usage: TokenUsage,
}

/// Trait for LLM backends. Intentionally minimal: single-turn, text-in/text-out.
///
/// We use `&dyn LlmBackend` (trait object) for simplicity — the dynamic dispatch
/// cost is negligible compared to LLM network latency. The return type is a boxed
/// future to enable dyn-compatibility.
pub trait LlmBackend: Send + Sync {
    /// Send a single-turn completion request.
    fn complete(
        &self,
        system: &str,
        user: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<LlmResponse>> + Send + '_>>;
}

// ---------------------------------------------------------------------------
// Backend: cc-sdk LLM Proxy
// ---------------------------------------------------------------------------

/// Uses `cc_sdk::llm::query()` — routes through Claude Code subscription.
/// No ANTHROPIC_API_KEY needed.
pub struct CcSdkBackend;

impl LlmBackend for CcSdkBackend {
    fn complete(
        &self,
        system: &str,
        user: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<LlmResponse>> + Send + '_>> {
        let system = system.to_string();
        let user = user.to_string();
        Box::pin(async move {
            let options = cc_sdk::llm::LlmOptions::builder()
                .system_prompt(&system)
                .build();

            let response = cc_sdk::llm::query(&user, Some(options))
                .await
                .context("cc-sdk LLM Proxy query failed")?;

            Ok(LlmResponse {
                text: response.text,
                usage: TokenUsage::default(),
            })
        })
    }
}

// ---------------------------------------------------------------------------
// Backend: Codex Responses API Proxy
// ---------------------------------------------------------------------------

/// Uses OpenAI Responses API (directly or via local proxy).
///
/// Two modes:
/// - **Direct**: `base_url = "https://api.openai.com"`, auth_token from `~/.codex/auth.json`
/// - **Proxy**: `base_url = "http://127.0.0.1:{port}"`, no auth needed (proxy handles it)
pub struct CodexBackend {
    client: reqwest::Client,
    base_url: String,
    auth_token: Option<String>,
    account_id: Option<String>,
    model: String,
}

impl CodexBackend {
    /// Create with explicit URL (for local proxy or custom endpoint).
    pub fn new(base_url: &str, auth_token: Option<String>, model: Option<&str>) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            auth_token,
            account_id: None,
            model: model.unwrap_or("o3").to_string(),
        }
    }

    /// Create a backend using Codex subscription auth from `~/.codex/auth.json`.
    ///
    /// Uses the ChatGPT backend endpoint (`chatgpt.com/backend-api/codex/responses`)
    /// which accepts OAuth tokens from `codex login`, NOT `api.openai.com` which
    /// requires API keys.
    pub fn from_codex_auth(model: Option<&str>) -> Result<Self> {
        let auth_path = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("cannot determine home directory"))?
            .join(".codex/auth.json");

        let auth_data: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(&auth_path)
                .with_context(|| format!("cannot read {}", auth_path.display()))?,
        )
        .context("invalid auth.json")?;

        let token = auth_data["tokens"]["access_token"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("no access_token in ~/.codex/auth.json"))?
            .to_string();

        let account_id = auth_data["tokens"]["account_id"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(Self {
            client: reqwest::Client::new(),
            // Codex subscription endpoint — full URL, NOT api.openai.com
            base_url: "https://chatgpt.com/backend-api/codex/responses".to_string(),
            auth_token: Some(token),
            account_id: Some(account_id),
            model: model.unwrap_or("o3").to_string(),
        })
    }
}

impl LlmBackend for CodexBackend {
    fn complete(
        &self,
        system: &str,
        user: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<LlmResponse>> + Send + '_>> {
        let system = system.to_string();
        let user = user.to_string();
        let model = self.model.clone();
        // If base_url already ends with /responses, use it directly;
        // otherwise append /v1/responses (for proxy mode like api.openai.com).
        let url = if self.base_url.ends_with("/responses") {
            self.base_url.clone()
        } else {
            format!("{}/v1/responses", self.base_url)
        };
        let client = self.client.clone();
        let auth_token = self.auth_token.clone();
        let account_id = self.account_id.clone();
        Box::pin(async move {
            let body = serde_json::json!({
                "model": model,
                "instructions": system,
                "input": [{
                    "type": "message",
                    "role": "user",
                    "content": [{"type": "input_text", "text": user}]
                }],
                "tools": [],
                "stream": true,
                "store": false
            });

            let mut req = client
                .post(&url)
                .header("Content-Type", "application/json")
                .header("Accept", "text/event-stream");

            if let Some(token) = &auth_token {
                req = req.header("Authorization", format!("Bearer {token}"));
            }
            // Required for Codex subscription endpoint (chatgpt.com backend)
            req = req.header("OpenAI-Beta", "responses=experimental");
            if let Some(acct) = &account_id {
                if !acct.is_empty() {
                    req = req.header("chatgpt-account-id", acct);
                }
            }

            let http_resp = req
                .json(&body)
                .send()
                .await
                .context("Codex Responses API request failed")?;

            let status = http_resp.status();
            if !status.is_success() {
                let error_body = http_resp.text().await.unwrap_or_default();
                tracing::warn!(status = %status, "Codex API error: {error_body}");
                anyhow::bail!(
                    "Codex Responses API returned {status}: {error_body}"
                );
            }

            // Parse SSE stream: collect text deltas until response.completed
            let mut text = String::new();
            let mut usage = TokenUsage::default();
            let mut event_type = String::new();
            let mut completed = false;

            let body = http_resp.text().await.context("Failed to read SSE body")?;

            for line in body.lines() {
                if let Some(ev) = line.strip_prefix("event: ") {
                    event_type = ev.trim().to_string();
                } else if let Some(data) = line.strip_prefix("data: ") {
                    match event_type.as_str() {
                        "response.output_text.delta" => {
                            if let Ok(val) = serde_json::from_str::<serde_json::Value>(data) {
                                if let Some(delta) = val["delta"].as_str() {
                                    text.push_str(delta);
                                }
                            }
                        }
                        "response.completed" => {
                            completed = true;
                            if let Ok(val) = serde_json::from_str::<serde_json::Value>(data) {
                                usage = TokenUsage {
                                    input_tokens: val["response"]["usage"]["input_tokens"].as_u64(),
                                    output_tokens: val["response"]["usage"]["output_tokens"].as_u64(),
                                };
                            }
                            break;
                        }
                        _ => {} // Skip other events
                    }
                }
            }

            if !completed {
                anyhow::bail!("Codex SSE stream ended without response.completed event (truncated?)");
            }

            Ok(LlmResponse { text, usage })
        })
    }
}

// ---------------------------------------------------------------------------
// Backend: cc-sdk WebSocket (persistent connection via ClaudeSDKClient)
// ---------------------------------------------------------------------------

/// Uses cc-sdk's WebSocket transport for persistent connections to remote CC instances.
pub struct CcSdkWsBackend {
    ws_url: String,
    auth_token: Option<String>,
}

impl CcSdkWsBackend {
    pub fn new(ws_url: &str, auth_token: Option<String>) -> Self {
        Self {
            ws_url: ws_url.to_string(),
            auth_token,
        }
    }
}

impl LlmBackend for CcSdkWsBackend {
    fn complete(
        &self,
        system: &str,
        user: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<LlmResponse>> + Send + '_>> {
        let system = system.to_string();
        let user = user.to_string();
        let ws_url = self.ws_url.clone();
        let auth_token = self.auth_token.clone();
        Box::pin(async move {
            use futures::StreamExt;

            // Build WebSocket transport
            let ws_config = cc_sdk::WebSocketConfig {
                auth_token,
                ..Default::default()
            };
            let transport = cc_sdk::WebSocketTransport::new(&ws_url, ws_config)
                .map_err(|e| anyhow::anyhow!("WebSocket transport error: {e}"))?;

            // Build options with system prompt, no tools
            let options = cc_sdk::ClaudeCodeOptions::builder()
                .system_prompt(&system)
                .permission_mode(cc_sdk::PermissionMode::DontAsk)
                .max_turns(1)
                .build();

            // Create client with WebSocket transport
            let mut client =
                cc_sdk::ClaudeSDKClient::with_transport(options, Box::new(transport));
            client
                .connect(Some(user))
                .await
                .map_err(|e| anyhow::anyhow!("WebSocket connect failed: {e}"))?;

            // Collect response — scope the stream borrow so we can disconnect after
            let mut text = String::new();
            {
                let stream = client.receive_response().await;
                tokio::pin!(stream);

                while let Some(msg_result) = stream.next().await {
                    let msg =
                        msg_result.map_err(|e| anyhow::anyhow!("WS message error: {e}"))?;
                    match msg {
                        cc_sdk::Message::Assistant { message } => {
                            for block in &message.content {
                                if let cc_sdk::ContentBlock::Text(t) = block {
                                    text.push_str(&t.text);
                                }
                            }
                        }
                        cc_sdk::Message::Result { is_error, result, .. } => {
                            if is_error {
                                let err = result.as_deref().unwrap_or("unknown error");
                                anyhow::bail!("cc-sdk WebSocket error: {err}");
                            }
                            break;
                        }
                        _ => {}
                    }
                }
            } // stream dropped here, releasing borrow on client

            client
                .disconnect()
                .await
                .map_err(|e| anyhow::anyhow!("WS disconnect error: {e}"))?;

            Ok(LlmResponse {
                text,
                usage: TokenUsage::default(),
            })
        })
    }
}

// ---------------------------------------------------------------------------
// Backend: Codex app-server WebSocket (JSON-RPC over WebSocket)
// ---------------------------------------------------------------------------

/// Connects to Codex app-server via WebSocket using JSON-RPC protocol.
/// Lightweight client — no dependency on codex_app_server_protocol crate.
pub struct CodexWsBackend {
    ws_url: String,
    auth_token: Option<String>,
    model: String,
}

impl CodexWsBackend {
    pub fn new(ws_url: &str, auth_token: Option<String>, model: Option<&str>) -> Self {
        Self {
            ws_url: ws_url.to_string(),
            auth_token,
            model: model.unwrap_or("o3").to_string(),
        }
    }
}

/// Helper: send a JSON-RPC message over WebSocket.
async fn ws_send(
    write: &mut (impl futures::SinkExt<tokio_tungstenite::tungstenite::Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin),
    msg: &serde_json::Value,
) -> Result<()> {
    write
        .send(tokio_tungstenite::tungstenite::Message::Text(
            msg.to_string().into(),
        ))
        .await
        .context("WebSocket send failed")
}

/// Helper: read WebSocket text messages, skipping non-text frames.
fn ws_text(msg: tokio_tungstenite::tungstenite::Message) -> Option<String> {
    match msg {
        tokio_tungstenite::tungstenite::Message::Text(t) => Some(t.to_string()),
        _ => None,
    }
}

impl LlmBackend for CodexWsBackend {
    fn complete(
        &self,
        system: &str,
        user: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<LlmResponse>> + Send + '_>> {
        let system = system.to_string();
        let user = user.to_string();
        let ws_url = self.ws_url.clone();
        let auth_token = self.auth_token.clone();
        let model = self.model.clone();
        Box::pin(async move {
            use futures::{SinkExt, StreamExt};

            // Build connection request with optional auth
            let mut request = tokio_tungstenite::tungstenite::http::Request::builder()
                .uri(&ws_url)
                .header("Sec-WebSocket-Version", "13")
                .header("Sec-WebSocket-Key", tokio_tungstenite::tungstenite::handshake::client::generate_key())
                .header("Host", tokio_tungstenite::tungstenite::http::Uri::try_from(&ws_url)
                    .ok()
                    .and_then(|u| u.host().map(|h| h.to_string()))
                    .unwrap_or_default())
                .header("Connection", "Upgrade")
                .header("Upgrade", "websocket");

            if let Some(token) = &auth_token {
                request = request.header("Authorization", format!("Bearer {token}"));
            }

            let request = request.body(()).context("failed to build WebSocket request")?;

            let (ws_stream, _) = tokio_tungstenite::connect_async(request)
                .await
                .context("Codex WebSocket connection failed")?;

            let (mut write, mut read) = ws_stream.split();

            // Step 1: Initialize
            ws_send(&mut write, &serde_json::json!({
                "jsonrpc": "2.0", "method": "initialize", "id": 1,
                "params": { "clientName": "code-review-agent", "clientVersion": "0.1.0" }
            })).await?;

            // Wait for init response
            let mut initialized = false;
            while let Some(Ok(msg)) = read.next().await {
                if let Some(text) = ws_text(msg) {
                    let val: serde_json::Value = serde_json::from_str(&text)?;
                    if val.get("id") == Some(&serde_json::json!(1)) {
                        initialized = true;
                        break;
                    }
                }
            }
            if !initialized {
                anyhow::bail!("Codex WebSocket initialization failed");
            }

            // Send initialized notification
            ws_send(&mut write, &serde_json::json!({
                "jsonrpc": "2.0", "method": "initialized"
            })).await?;

            // Step 2: Start thread
            ws_send(&mut write, &serde_json::json!({
                "jsonrpc": "2.0", "method": "thread/start", "id": 2,
                "params": { "model": model, "baseInstructions": system, "ephemeral": true }
            })).await?;

            let mut thread_id = String::new();
            while let Some(Ok(msg)) = read.next().await {
                if let Some(text) = ws_text(msg) {
                    let val: serde_json::Value = serde_json::from_str(&text)?;
                    if val.get("id") == Some(&serde_json::json!(2)) {
                        thread_id = val["result"]["thread"]["id"]
                            .as_str()
                            .unwrap_or("")
                            .to_string();
                        break;
                    }
                }
            }
            if thread_id.is_empty() {
                anyhow::bail!("Codex WebSocket: failed to start thread");
            }

            // Step 3: Start turn with user message
            ws_send(&mut write, &serde_json::json!({
                "jsonrpc": "2.0", "method": "turn/start", "id": 3,
                "params": {
                    "threadId": thread_id,
                    "input": [{ "type": "text", "text": user }]
                }
            })).await?;

            // Step 4: Collect response from notifications
            let mut response_text = String::new();
            let mut usage = TokenUsage::default();

            while let Some(Ok(msg)) = read.next().await {
                if let Some(text) = ws_text(msg) {
                    let val: serde_json::Value = serde_json::from_str(&text)?;
                    let method = val["method"].as_str().unwrap_or("");

                    match method {
                        "item/agentMessage/delta" => {
                            if let Some(delta) = val["params"]["delta"].as_str() {
                                response_text.push_str(delta);
                            }
                        }
                        "turn/completed" => {
                            if let Some(u) = val["params"].get("usage") {
                                usage = TokenUsage {
                                    input_tokens: u["inputTokens"].as_u64(),
                                    output_tokens: u["outputTokens"].as_u64(),
                                };
                            }
                            break;
                        }
                        _ => {}
                    }
                }
            }

            write.close().await.ok();

            Ok(LlmResponse { text: response_text, usage })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_usage_accumulate() {
        let mut total = TokenUsage::default();
        assert_eq!(total.total(), 0);

        total.accumulate(&TokenUsage {
            input_tokens: Some(100),
            output_tokens: Some(50),
        });
        assert_eq!(total.total(), 150);

        total.accumulate(&TokenUsage {
            input_tokens: Some(200),
            output_tokens: None,
        });
        assert_eq!(total.input_tokens, Some(300));
        assert_eq!(total.output_tokens, Some(50));
    }

    #[test]
    fn test_token_usage_default_is_zero() {
        let usage = TokenUsage::default();
        assert_eq!(usage.total(), 0);
        assert!(usage.input_tokens.is_none());
        assert!(usage.output_tokens.is_none());
    }
}
