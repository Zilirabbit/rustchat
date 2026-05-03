use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use axum::extract::ws::Message;
use tokio::sync::{RwLock, mpsc};

use super::protocol::{ServerEvent, server_event_message};

#[derive(Clone, Default)]
pub struct ConnectionManager {
    inner: Arc<ConnectionManagerInner>,
}

#[derive(Default)]
struct ConnectionManagerInner {
    next_connection_id: AtomicU64,
    state: RwLock<ConnectionState>,
}

#[derive(Default)]
struct ConnectionState {
    user_to_connection: HashMap<i64, ConnectionHandle>,
    connection_to_user: HashMap<u64, i64>,
}

#[derive(Clone)]
struct ConnectionHandle {
    connection_id: u64,
    sender: mpsc::UnboundedSender<Message>,
}

pub struct RegisteredConnection {
    pub connection_id: u64,
    sender: mpsc::UnboundedSender<Message>,
    receiver: mpsc::UnboundedReceiver<Message>,
}

impl RegisteredConnection {
    pub fn sender(&self) -> mpsc::UnboundedSender<Message> {
        self.sender.clone()
    }

    pub fn into_receiver(self) -> mpsc::UnboundedReceiver<Message> {
        self.receiver
    }
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn register(&self, user_id: i64) -> RegisteredConnection {
        let connection_id = self
            .inner
            .next_connection_id
            .fetch_add(1, Ordering::Relaxed)
            .saturating_add(1);
        let (sender, receiver) = mpsc::unbounded_channel();

        let mut state = self.inner.state.write().await;
        if let Some(previous) = state.user_to_connection.insert(
            user_id,
            ConnectionHandle {
                connection_id,
                sender: sender.clone(),
            },
        ) {
            state.connection_to_user.remove(&previous.connection_id);
        }
        state.connection_to_user.insert(connection_id, user_id);

        RegisteredConnection {
            connection_id,
            sender,
            receiver,
        }
    }

    pub async fn unregister(&self, user_id: i64, connection_id: u64) {
        let mut state = self.inner.state.write().await;

        if state
            .user_to_connection
            .get(&user_id)
            .is_some_and(|connection| connection.connection_id == connection_id)
        {
            state.user_to_connection.remove(&user_id);
        }

        if state
            .connection_to_user
            .get(&connection_id)
            .is_some_and(|mapped_user_id| *mapped_user_id == user_id)
        {
            state.connection_to_user.remove(&connection_id);
        }
    }

    pub async fn send_to_user(&self, user_id: i64, event: &ServerEvent) -> bool {
        let sender = {
            let state = self.inner.state.read().await;
            state
                .user_to_connection
                .get(&user_id)
                .map(|connection| connection.sender.clone())
        };

        match (sender, server_event_message(event)) {
            (Some(sender), Ok(message)) => sender.send(message).is_ok(),
            (Some(_), Err(error)) => {
                tracing::warn!(error = %error, "failed to serialize websocket event");
                false
            }
            (None, _) => false,
        }
    }

    #[cfg(test)]
    pub async fn is_online(&self, user_id: i64) -> bool {
        let state = self.inner.state.read().await;
        state.user_to_connection.contains_key(&user_id)
    }

    #[cfg(test)]
    pub async fn connection_id_for_user(&self, user_id: i64) -> Option<u64> {
        let state = self.inner.state.read().await;
        state
            .user_to_connection
            .get(&user_id)
            .map(|connection| connection.connection_id)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::ConnectionManager;
    use crate::connection::protocol::ServerEvent;

    #[tokio::test]
    async fn register_marks_user_online_and_unregister_cleans_up() {
        let manager = ConnectionManager::new();
        let registered = manager.register(7).await;

        assert!(manager.is_online(7).await);
        assert_eq!(
            manager.connection_id_for_user(7).await,
            Some(registered.connection_id)
        );

        manager.unregister(7, registered.connection_id).await;

        assert!(!manager.is_online(7).await);
        assert_eq!(manager.connection_id_for_user(7).await, None);
    }

    #[tokio::test]
    async fn stale_disconnect_does_not_remove_newer_connection() {
        let manager = ConnectionManager::new();
        let first = manager.register(7).await;
        let second = manager.register(7).await;

        manager.unregister(7, first.connection_id).await;

        assert!(manager.is_online(7).await);
        assert_eq!(
            manager.connection_id_for_user(7).await,
            Some(second.connection_id)
        );

        manager.unregister(7, second.connection_id).await;
        assert!(!manager.is_online(7).await);
    }

    #[tokio::test]
    async fn send_to_user_pushes_protocol_message() {
        let manager = ConnectionManager::new();
        let registered = manager.register(7).await;
        let connection_id = registered.connection_id;
        let mut receiver = registered.into_receiver();

        assert!(
            manager
                .send_to_user(
                    7,
                    &ServerEvent::Connected {
                        user_id: 7,
                        username: "alice".to_string(),
                        connection_id,
                    },
                )
                .await
        );

        let message = receiver.recv().await.unwrap();
        let text = message.into_text().unwrap();
        let body: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(body["type"], "connected");
        assert_eq!(body["connection_id"], connection_id);
    }
}
