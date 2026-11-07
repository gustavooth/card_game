use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Mutex;
use std::time::Instant;

use axum::Json;
use axum::extract::{ConnectInfo, Request, State};
use axum::http::{HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use crate::state::AppState;

struct Bucket {
    tokens: f32,
    last: Instant,
}

/// Token buckets em memória, por escopo + chave (IP ou usuário).
pub struct RateLimiter {
    buckets: Mutex<HashMap<String, Bucket>>,
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            buckets: Mutex::new(HashMap::new()),
        }
    }

    pub fn check(&self, key: &str, capacity: f32, refill_per_sec: f32) -> bool {
        let now = Instant::now();
        let Ok(mut buckets) = self.buckets.lock() else {
            return true;
        };
        if buckets.len() > 20_000 {
            buckets.retain(|_, bucket| now.duration_since(bucket.last).as_secs() < 900);
        }
        let bucket = buckets.entry(key.to_string()).or_insert(Bucket {
            tokens: capacity,
            last: now,
        });
        let elapsed = now.duration_since(bucket.last).as_secs_f32();
        bucket.tokens = (bucket.tokens + elapsed * refill_per_sec).min(capacity);
        bucket.last = now;
        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

pub fn client_ip(headers: &HeaderMap, connect: Option<ConnectInfo<SocketAddr>>) -> String {
    headers
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| connect.map(|ConnectInfo(addr)| addr.ip().to_string()))
        .unwrap_or_else(|| "unknown".to_string())
}

pub async fn global_middleware(
    State(state): State<AppState>,
    connect: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Response {
    let path = request.uri().path().to_string();
    if path.starts_with("/api") && path != "/api/health" {
        let ip = client_ip(&headers, Some(connect));
        if !state.limiter().check(&format!("global:{ip}"), 180.0, 4.0) {
            return (
                StatusCode::TOO_MANY_REQUESTS,
                Json(serde_json::json!({
                    "code": "rate_limited",
                    "message": "Muitas requisições. Tente novamente em instantes."
                })),
            )
                .into_response();
        }
    }
    next.run(request).await
}
