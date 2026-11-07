use std::net::SocketAddr;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{ConnectInfo, State};
use axum::http::HeaderMap;
use axum::response::Response;
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use royale_db::models::User;
use royale_shared::{ChatMsg, ErrorMsg, GameCommand, PongMsg};

use crate::error::ApiError;
use crate::extractors::AuthUser;
use crate::rate_limit::client_ip;
use crate::state::AppState;

pub async fn upgrade(
    State(state): State<AppState>,
    connect: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    user: AuthUser,
    ws: WebSocketUpgrade,
) -> Result<Response, ApiError> {
    let ip = client_ip(&headers, Some(connect));
    if !state
        .limiter()
        .check(&format!("ws:{ip}"), 20.0, 20.0 / 60.0)
    {
        return Err(ApiError::RateLimited("Muitas conexões. Aguarde."));
    }
    let username = user.user.username.clone();
    Ok(ws.on_upgrade(move |socket| handle(socket, state, user.user, username)))
}

async fn handle(socket: WebSocket, state: AppState, user: User, username: String) {
    let (mut sink, mut stream) = socket.split();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();
    state.hub().register(user.id, tx);

    let writer = tokio::spawn(async move {
        while let Some(payload) = rx.recv().await {
            if sink.send(Message::Binary(payload.into())).await.is_err() {
                break;
            }
        }
    });

    let _ = state
        .game_tx()
        .send(GameCommand::Reconnect { user_id: user.id });
    tracing::info!(user = %username, "WebSocket conectado");

    while let Some(message) = stream.next().await {
        let Ok(message) = message else {
            break;
        };
        match message {
            Message::Binary(bytes) => match rmpv::decode::read_value(&mut bytes.as_ref()) {
                Ok(value) => handle_command(&state, &user, &username, value).await,
                Err(_) => send_error(&state, user.id, "invalid_message", "Pacote inválido"),
            },
            Message::Text(text) => {
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) {
                    handle_command(&state, &user, &username, json_to_rmpv(value)).await;
                } else {
                    send_error(&state, user.id, "invalid_message", "Texto inválido");
                }
            }
            Message::Ping(_) | Message::Pong(_) => {}
            Message::Close(_) => break,
        }
    }

    state.hub().unregister(user.id);
    let _ = state
        .game_tx()
        .send(GameCommand::Disconnect { user_id: user.id });
    writer.abort();
    tracing::info!(user = %username, "WebSocket desconectado");
}

async fn handle_command(state: &AppState, user: &User, username: &str, value: rmpv::Value) {
    let Some(kind) = value
        .as_map()
        .and_then(|_| map_get(&value, "t"))
        .and_then(|value| value.as_str())
        .map(str::to_string)
    else {
        send_error(state, user.id, "invalid_message", "Comando sem tipo");
        return;
    };

    match kind.as_str() {
        "q" => {
            if !state
                .limiter()
                .check(&format!("queue:{}", user.id), 4.0, 4.0 / 10.0)
            {
                send_error(
                    state,
                    user.id,
                    "rate_limited",
                    "Aguarde para entrar na fila",
                );
                return;
            }
            let _ = state.game_tx().send(GameCommand::JoinQueue {
                user_id: user.id,
                username: username.to_string(),
            });
        }
        "lq" => {
            let _ = state
                .game_tx()
                .send(GameCommand::LeaveQueue { user_id: user.id });
        }
        "d" => {
            if !state
                .limiter()
                .check(&format!("deploy:{}", user.id), 30.0, 12.0)
            {
                send_error(
                    state,
                    user.id,
                    "rate_limited",
                    "Muitos comandos por segundo",
                );
                return;
            }
            let slot = map_get(&value, "s")
                .and_then(|value| value.as_u64())
                .unwrap_or(255) as u8;
            let x = map_get(&value, "x")
                .and_then(|value| value.as_f64())
                .unwrap_or(f64::NAN) as f32;
            let y = map_get(&value, "y")
                .and_then(|value| value.as_f64())
                .unwrap_or(f64::NAN) as f32;
            if !x.is_finite() || !y.is_finite() {
                send_error(state, user.id, "invalid_command", "Coordenadas inválidas");
                return;
            }
            let _ = state.game_tx().send(GameCommand::Deploy {
                user_id: user.id,
                slot,
                x,
                y,
            });
        }
        "c" => {
            if !state
                .limiter()
                .check(&format!("chat:{}", user.id), 3.0, 0.4)
            {
                send_error(
                    state,
                    user.id,
                    "rate_limited",
                    "Aguarde para enviar outra mensagem",
                );
                return;
            }
            let Some(text) = map_get(&value, "m").and_then(|value| value.as_str()) else {
                return;
            };
            let text = text.trim();
            if text.is_empty() {
                return;
            }
            let text = text
                .chars()
                .take(royale_shared::CHAT_MAX_CHARS)
                .collect::<String>();
            let payload = royale_shared::encode_message(&ChatMsg {
                tag: "chat",
                user: username.to_string(),
                text,
                ts: Utc::now().timestamp_millis(),
            });
            state.hub().broadcast(&payload, None);
        }
        "p" => {
            let ts = map_get(&value, "ts")
                .and_then(|value| value.as_u64())
                .unwrap_or(0);
            let payload = royale_shared::encode_message(&PongMsg { tag: "pong", ts });
            state.hub().send_user(user.id, payload);
        }
        _ => {
            send_error(state, user.id, "unknown_command", "Comando desconhecido");
        }
    }
}

fn send_error(state: &AppState, user_id: uuid::Uuid, code: &'static str, message: &str) {
    let payload = royale_shared::encode_message(&ErrorMsg {
        tag: "err",
        code,
        message: message.to_string(),
    });
    state.hub().send_user(user_id, payload);
}

fn map_get<'a>(value: &'a rmpv::Value, key: &str) -> Option<&'a rmpv::Value> {
    value.as_map()?.iter().find_map(|(k, v)| {
        if k.as_str() == Some(key) {
            Some(v)
        } else {
            None
        }
    })
}

fn json_to_rmpv(value: serde_json::Value) -> rmpv::Value {
    match value {
        serde_json::Value::Null => rmpv::Value::Nil,
        serde_json::Value::Bool(value) => rmpv::Value::Boolean(value),
        serde_json::Value::Number(number) => {
            if let Some(value) = number.as_i64() {
                rmpv::Value::from(value)
            } else if let Some(value) = number.as_u64() {
                rmpv::Value::from(value)
            } else {
                rmpv::Value::F64(number.as_f64().unwrap_or(0.0))
            }
        }
        serde_json::Value::String(value) => rmpv::Value::from(value),
        serde_json::Value::Array(values) => {
            rmpv::Value::Array(values.into_iter().map(json_to_rmpv).collect())
        }
        serde_json::Value::Object(object) => rmpv::Value::Map(
            object
                .into_iter()
                .map(|(key, value)| (rmpv::Value::from(key), json_to_rmpv(value)))
                .collect(),
        ),
    }
}
