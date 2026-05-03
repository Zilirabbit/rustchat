use axum::{
    extract::{
        Query, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::{HeaderMap, header},
    response::IntoResponse,
};
use serde::Deserialize;

use crate::{
    app::AppState,
    auth::types::CurrentUser,
    common::error::{AppError, AppResult},
    middleware::auth::extract_bearer_token,
};

use super::{
    manager::ConnectionManager,
    protocol::{ClientEvent, ServerEvent, parse_client_event},
};

#[derive(Debug, Deserialize, Default)]
pub struct WsConnectQuery {
    token: Option<String>,
}

pub async fn ws_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<WsConnectQuery>,
    ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, AppError> {
    let token = extract_ws_token(&headers, query.token.as_deref())?;
    let claims = state.auth.jwt.decode_token(&token)?;
    let current_user = CurrentUser::from(claims);
    let connection_manager = state.connections.clone();

    Ok(ws.on_upgrade(move |socket| handle_socket(connection_manager, current_user, socket)))
}

fn extract_ws_token(headers: &HeaderMap, query_token: Option<&str>) -> AppResult<String> {
    if headers.contains_key(header::AUTHORIZATION) {
        return extract_bearer_token(headers);
    }

    let token = query_token
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .ok_or_else(|| AppError::Unauthorized("missing websocket token".to_string()))?;

    Ok(token.to_string())
}

async fn handle_socket(
    connection_manager: ConnectionManager,
    current_user: CurrentUser,
    mut socket: WebSocket,
) {
    let registered = connection_manager.register(current_user.user_id).await;
    let connection_id = registered.connection_id;
    let outbound_sender = registered.sender();
    let mut outbound = registered.into_receiver();
    let _ = connection_manager
        .send_to_user(
            current_user.user_id,
            &ServerEvent::Connected {
                user_id: current_user.user_id,
                username: current_user.username.clone(),
                connection_id,
            },
        )
        .await;

    tracing::info!(
        user_id = current_user.user_id,
        connection_id,
        "websocket connected"
    );

    loop {
        tokio::select! {
            outbound_message = outbound.recv() => {
                match outbound_message {
                    Some(message) => {
                        if socket.send(message).await.is_err() {
                            break;
                        }
                    }
                    None => break,
                }
            }
            inbound_message = socket.recv() => {
                match inbound_message {
                    Some(Ok(message)) => {
                        if !handle_inbound_message(&outbound_sender, message).await {
                            break;
                        }
                    }
                    Some(Err(error)) => {
                        tracing::warn!(
                            user_id = current_user.user_id,
                            connection_id,
                            error = %error,
                            "websocket receive failed"
                        );
                        break;
                    }
                    None => break,
                }
            }
        }
    }

    connection_manager
        .unregister(current_user.user_id, connection_id)
        .await;

    tracing::info!(
        user_id = current_user.user_id,
        connection_id,
        "websocket disconnected"
    );
}

async fn handle_inbound_message(
    outbound_sender: &tokio::sync::mpsc::UnboundedSender<Message>,
    message: Message,
) -> bool {
    match message {
        Message::Text(payload) => match parse_client_event(payload.as_ref()) {
            Ok(ClientEvent::Ping) => enqueue_event(outbound_sender, &ServerEvent::Pong),
            Err(_) => enqueue_event(
                outbound_sender,
                &ServerEvent::Error {
                    message: "invalid websocket message".to_string(),
                },
            ),
        },
        Message::Binary(_) => enqueue_event(
            outbound_sender,
            &ServerEvent::Error {
                message: "binary websocket message is not supported".to_string(),
            },
        ),
        Message::Ping(payload) => outbound_sender.send(Message::Pong(payload)).is_ok(),
        Message::Pong(_) => true,
        Message::Close(_) => false,
    }
}

fn enqueue_event(
    outbound_sender: &tokio::sync::mpsc::UnboundedSender<Message>,
    event: &ServerEvent,
) -> bool {
    match super::protocol::server_event_message(event) {
        Ok(message) => outbound_sender.send(message).is_ok(),
        Err(error) => {
            tracing::warn!(error = %error, "failed to serialize websocket event");
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{Router, http::StatusCode, routing::get};
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpStream,
        task::JoinHandle,
    };

    use super::ws_handler;
    use crate::{
        app::AppState, auth::jwt::JwtService, common::config::JwtConfig,
        user::service::UnavailableUserService,
    };

    fn test_state() -> AppState {
        AppState::new(
            None,
            JwtService::new(JwtConfig {
                secret: "connection-handler-test-secret".to_string(),
                expires_in_secs: 3_600,
                issuer: "rustchat-test".to_string(),
            }),
            Arc::new(UnavailableUserService),
        )
    }

    async fn spawn_test_server(state: AppState) -> (std::net::SocketAddr, JoinHandle<()>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let app = Router::new()
            .route("/ws", get(ws_handler))
            .with_state(state);
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        (addr, server)
    }

    async fn send_websocket_handshake(
        addr: std::net::SocketAddr,
        path: &str,
        authorization: Option<&str>,
    ) -> String {
        let mut stream = TcpStream::connect(addr).await.unwrap();
        let mut request = format!(
            "GET {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: Upgrade\r\nUpgrade: websocket\r\nSec-WebSocket-Version: 13\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n"
        );

        if let Some(authorization) = authorization {
            request.push_str(&format!("Authorization: {authorization}\r\n"));
        }

        request.push_str("\r\n");
        stream.write_all(request.as_bytes()).await.unwrap();

        let mut buffer = vec![0; 2048];
        let bytes_read = stream.read(&mut buffer).await.unwrap();
        String::from_utf8_lossy(&buffer[..bytes_read]).to_string()
    }

    #[tokio::test]
    async fn websocket_upgrade_accepts_valid_authorization_header() {
        let state = test_state();
        let token = state.auth.jwt.issue_token(7, "alice").unwrap();
        let (addr, server) = spawn_test_server(state).await;
        let response =
            send_websocket_handshake(addr, "/ws", Some(&format!("Bearer {token}"))).await;
        server.abort();

        assert!(response.starts_with("HTTP/1.1 101"));
    }

    #[tokio::test]
    async fn websocket_upgrade_accepts_valid_query_token() {
        let state = test_state();
        let token = state.auth.jwt.issue_token(7, "alice").unwrap();
        let (addr, server) = spawn_test_server(state).await;
        let response = send_websocket_handshake(addr, &format!("/ws?token={token}"), None).await;
        server.abort();

        assert!(response.starts_with("HTTP/1.1 101"));
    }

    #[tokio::test]
    async fn websocket_upgrade_rejects_missing_token() {
        let (addr, server) = spawn_test_server(test_state()).await;
        let response = send_websocket_handshake(addr, "/ws", None).await;
        server.abort();

        assert!(response.starts_with(&format!("HTTP/1.1 {}", StatusCode::UNAUTHORIZED.as_u16())));
        assert!(response.contains("missing websocket token"));
    }

    #[tokio::test]
    async fn websocket_upgrade_rejects_invalid_token() {
        let (addr, server) = spawn_test_server(test_state()).await;
        let response = send_websocket_handshake(addr, "/ws?token=invalid-token", None).await;
        server.abort();

        assert!(response.starts_with(&format!("HTTP/1.1 {}", StatusCode::UNAUTHORIZED.as_u16())));
    }
}
