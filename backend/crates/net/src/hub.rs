use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

/// Registro de conexões WebSocket autenticadas.
#[derive(Clone, Default)]
pub struct ConnectionHub {
    senders: Arc<Mutex<HashMap<Uuid, UnboundedSender<Vec<u8>>>>>,
}

impl ConnectionHub {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&self, user_id: Uuid, sender: UnboundedSender<Vec<u8>>) {
        if let Ok(mut senders) = self.senders.lock() {
            senders.insert(user_id, sender);
        }
    }

    pub fn unregister(&self, user_id: Uuid) {
        if let Ok(mut senders) = self.senders.lock() {
            senders.remove(&user_id);
        }
    }

    pub fn send_user(&self, user_id: Uuid, payload: Vec<u8>) -> bool {
        let sender = self
            .senders
            .lock()
            .ok()
            .and_then(|senders| senders.get(&user_id).cloned());
        sender.is_some_and(|sender| sender.send(payload).is_ok())
    }

    pub fn broadcast(&self, payload: &[u8], except: Option<Uuid>) {
        let senders: Vec<UnboundedSender<Vec<u8>>> = self
            .senders
            .lock()
            .map(|senders| {
                senders
                    .iter()
                    .filter(|(user_id, _)| Some(**user_id) != except)
                    .map(|(_, sender)| sender.clone())
                    .collect()
            })
            .unwrap_or_default();
        for sender in senders {
            let _ = sender.send(payload.to_vec());
        }
    }

    pub fn online(&self) -> usize {
        self.senders
            .lock()
            .map(|senders| senders.len())
            .unwrap_or(0)
    }

    pub fn is_online(&self, user_id: Uuid) -> bool {
        self.senders
            .lock()
            .map(|senders| senders.contains_key(&user_id))
            .unwrap_or(false)
    }
}
