use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::HeaderValue;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tracing::info;

use crate::error::MiniMaxError;
use crate::types::*;

/// WebSocket TTS client — connects to wss://api.minimaxi.com/ws/v1/t2a_v2
pub struct WsTtsClient {
    ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl WsTtsClient {
    /// Connect to the WebSocket endpoint and wait for `connected_success`.
    pub async fn connect(base_url: &str, api_key: &str) -> Result<Self, MiniMaxError> {
        // Replace https:// with wss://
        let ws_base = base_url
            .replace("https://", "wss://")
            .replace("http://", "ws://");
        let url = format!("{}/ws/v1/t2a_v2", ws_base);

        let mut request = url.as_str().into_client_request().map_err(|e| {
            MiniMaxError::Api {
                code: -1,
                message: format!("invalid WebSocket request: {e}"),
                trace_id: None,
            }
        })?;

        request.headers_mut().insert(
            "Authorization",
            HeaderValue::from_str(&format!("Bearer {}", api_key)).map_err(|e| {
                MiniMaxError::Api {
                    code: -1,
                    message: format!("invalid header value: {e}"),
                    trace_id: None,
                }
            })?,
        );
        request.headers_mut().insert(
            "MM-API-Source",
            HeaderValue::from_static("Minimax-MCP"),
        );

        let (ws, _response) = connect_async(request).await.map_err(|e| MiniMaxError::Api {
            code: -1,
            message: format!("WebSocket connection failed: {e}"),
            trace_id: None,
        })?;

        let mut client = Self { ws };

        // Wait for connected_success
        let msg = tokio::time::timeout(Duration::from_secs(10), client.ws.next())
            .await
            .map_err(|_| MiniMaxError::Api {
                code: -1,
                message: "timeout waiting for connected_success".to_string(),
                trace_id: None,
            })?
            .ok_or_else(|| MiniMaxError::Api {
                code: -1,
                message: "WebSocket closed before connected_success".to_string(),
                trace_id: None,
            })?
            .map_err(|e| MiniMaxError::Api {
                code: -1,
                message: format!("WebSocket error: {e}"),
                trace_id: None,
            })?;

        if let Message::Text(text) = msg {
            let value: serde_json::Value = serde_json::from_str(&text).map_err(|e| {
                MiniMaxError::Api {
                    code: -1,
                    message: format!("parse error: {e}"),
                    trace_id: None,
                }
            })?;
            let event = value.get("event").and_then(|v| v.as_str()).unwrap_or("");
            if event != "connected_success" {
                return Err(MiniMaxError::Api {
                    code: -1,
                    message: format!("unexpected event on connect: {event}"),
                    trace_id: None,
                });
            }
            let code = value
                .get("base_resp")
                .and_then(|b| b.get("status_code"))
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            if code != 0 {
                let msg = value
                    .get("base_resp")
                    .and_then(|b| b.get("status_msg"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                return Err(MiniMaxError::Api {
                    code: code as i32,
                    message: msg.to_string(),
                    trace_id: None,
                });
            }
            info!("WebSocket connected: session_id={}",
                value.get("session_id").and_then(|v| v.as_str()).unwrap_or("N/A"));
        } else {
            return Err(MiniMaxError::Api {
                code: -1,
                message: "expected text message for connected_success".to_string(),
                trace_id: None,
            });
        }

        Ok(client)
    }

    /// Send `task_start` event.
    pub async fn task_start(&mut self, req: &WsTaskStart) -> Result<(), MiniMaxError> {
        let json = serde_json::to_string(req).map_err(|e| MiniMaxError::Api {
            code: -1,
            message: format!("serialize error: {e}"),
            trace_id: None,
        })?;
        self.ws.send(Message::Text(json.into())).await.map_err(|e| {
            MiniMaxError::Api {
                code: -1,
                message: format!("send error: {e}"),
                trace_id: None,
            }
        })?;

        // Wait for task_started
        let msg = self.read_text_msg("task_started").await?;
        let value: serde_json::Value = serde_json::from_str(&msg).map_err(|e| {
            MiniMaxError::Api {
                code: -1,
                message: format!("parse error: {e}"),
                trace_id: None,
            }
        })?;
        Self::check_event_status(&value, "task_started")?;
        Ok(())
    }

    /// Send `task_continue` event with text, collect audio chunks.
    pub async fn task_continue(
        &mut self,
        text: &str,
    ) -> Result<(Vec<u8>, bool), MiniMaxError> {
        let req = WsTaskContinue {
            event: "task_continue".to_string(),
            text: text.to_string(),
        };
        let json = serde_json::to_string(&req).map_err(|e| MiniMaxError::Api {
            code: -1,
            message: format!("serialize error: {e}"),
            trace_id: None,
        })?;
        self.ws.send(Message::Text(json.into())).await.map_err(|e| {
            MiniMaxError::Api {
                code: -1,
                message: format!("send error: {e}"),
                trace_id: None,
            }
        })?;

        let mut all_audio = Vec::new();
        let mut is_final = false;

        loop {
            let msg = self.read_text_msg("task_continued").await?;
            let value: serde_json::Value = serde_json::from_str(&msg).map_err(|e| {
                MiniMaxError::Api {
                    code: -1,
                    message: format!("parse error: {e}"),
                    trace_id: None,
                }
            })?;

            let event = value.get("event").and_then(|v| v.as_str()).unwrap_or("");

            if event == "task_failed" {
                let code = value
                    .get("base_resp")
                    .and_then(|b| b.get("status_code"))
                    .and_then(|v| v.as_i64())
                    .unwrap_or(-1);
                let msg = value
                    .get("base_resp")
                    .and_then(|b| b.get("status_msg"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                return Err(MiniMaxError::Api {
                    code: code as i32,
                    message: msg.to_string(),
                    trace_id: None,
                });
            }

            if event == "task_continued" {
                if let Some(audio) = value
                    .get("data")
                    .and_then(|d| d.get("audio"))
                    .and_then(|v| v.as_str())
                {
                    let bytes = hex::decode(audio).map_err(|e| MiniMaxError::Api {
                        code: -1,
                        message: format!("hex decode error: {e}"),
                        trace_id: None,
                    })?;
                    all_audio.extend(bytes);
                }

                is_final = value.get("is_final").and_then(|v| v.as_bool()).unwrap_or(false);
                if is_final {
                    break;
                }
            } else {
                break;
            }
        }

        Ok((all_audio, is_final))
    }

    /// Send `task_finish` event and close the connection.
    pub async fn task_finish(&mut self) -> Result<(), MiniMaxError> {
        let req = WsTaskFinish {
            event: "task_finish".to_string(),
        };
        let json = serde_json::to_string(&req).map_err(|e| MiniMaxError::Api {
            code: -1,
            message: format!("serialize error: {e}"),
            trace_id: None,
        })?;
        self.ws.send(Message::Text(json.into())).await.map_err(|e| {
            MiniMaxError::Api {
                code: -1,
                message: format!("send error: {e}"),
                trace_id: None,
            }
        })?;

        // Wait for task_finished
        let msg = self.read_text_msg("task_finished").await?;
        let value: serde_json::Value = serde_json::from_str(&msg).map_err(|e| {
            MiniMaxError::Api {
                code: -1,
                message: format!("parse error: {e}"),
                trace_id: None,
            }
        })?;
        Self::check_event_status(&value, "task_finished")?;

        // Close the WebSocket
        let _ = self.ws.close(None).await;
        Ok(())
    }

    // ============================================================
    // Internal helpers
    // ============================================================

    async fn read_text_msg(
        &mut self,
        expected_event: &str,
    ) -> Result<String, MiniMaxError> {
        let msg = tokio::time::timeout(Duration::from_secs(60), self.ws.next())
            .await
            .map_err(|_| MiniMaxError::Api {
                code: -1,
                message: format!("timeout waiting for {expected_event}"),
                trace_id: None,
            })?
            .ok_or_else(|| MiniMaxError::Api {
                code: -1,
                message: format!("WebSocket closed while waiting for {expected_event}"),
                trace_id: None,
            })?
            .map_err(|e| MiniMaxError::Api {
                code: -1,
                message: format!("WebSocket error: {e}"),
                trace_id: None,
            })?;

        match msg {
            Message::Text(text) => Ok(text.to_string()),
            Message::Close(_) => Err(MiniMaxError::Api {
                code: -1,
                message: "WebSocket closed by server".to_string(),
                trace_id: None,
            }),
            other => Err(MiniMaxError::Api {
                code: -1,
                message: format!("unexpected message type: {:?}", other),
                trace_id: None,
            }),
        }
    }

    fn check_event_status(
        value: &serde_json::Value,
        expected_event: &str,
    ) -> Result<(), MiniMaxError> {
        let event = value.get("event").and_then(|v| v.as_str()).unwrap_or("");
        if event == "task_failed" {
            let code = value
                .get("base_resp")
                .and_then(|b| b.get("status_code"))
                .and_then(|v| v.as_i64())
                .unwrap_or(-1);
            let msg = value
                .get("base_resp")
                .and_then(|b| b.get("status_msg"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            return Err(MiniMaxError::Api {
                code: code as i32,
                message: msg.to_string(),
                trace_id: None,
            });
        }
        if event != expected_event {
            return Err(MiniMaxError::Api {
                code: -1,
                message: format!("expected {expected_event}, got {event}"),
                trace_id: None,
            });
        }
        let code = value
            .get("base_resp")
            .and_then(|b| b.get("status_code"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        if code != 0 {
            let msg = value
                .get("base_resp")
                .and_then(|b| b.get("status_msg"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            return Err(MiniMaxError::Api {
                code: code as i32,
                message: msg.to_string(),
                trace_id: None,
            });
        }
        Ok(())
    }
}
