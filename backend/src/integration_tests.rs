use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::task::JoinHandle;
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};
use tower::util::ServiceExt;

use crate::{
    app::AppState,
    common::config::{AppConfig, DatabaseConfig, JwtConfig},
    router::create_router,
};

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL to point at the rustchat_test database"]
async fn real_database_private_chat_flow_works_end_to_end() {
    dotenvy::from_filename(".env.test").ok();

    let state = real_database_state().await;
    reset_test_data(&state).await;
    let app = create_router(state.clone());

    let suffix = unique_suffix();
    let alice_username = format!("alice_it_{suffix}");
    let bob_username = format!("bob_it_{suffix}");
    let password = "secret123";

    let alice = register_user(app.clone(), &alice_username, password).await;
    let bob = register_user(app.clone(), &bob_username, password).await;

    let alice_token = login_user(app.clone(), &alice_username, password).await;
    let bob_token = login_user(app.clone(), &bob_username, password).await;

    let me = get_json(app.clone(), "/api/me", Some(&alice_token)).await;
    assert_eq!(me["data"]["user_id"], alice["data"]["user_id"]);
    assert_eq!(me["data"]["username"], alice_username);

    let search = get_json(
        app.clone(),
        &format!("/api/users?keyword={bob_username}"),
        Some(&alice_token),
    )
    .await;
    assert_eq!(search["message"], "users searched");
    let searched_bob = search["data"]
        .as_array()
        .expect("search data should be an array")
        .iter()
        .find(|item| item["user_id"] == bob["data"]["user_id"])
        .expect("alice should find bob by username");
    assert_eq!(searched_bob["username"], bob_username);

    let private_session = post_json(
        app.clone(),
        "/api/sessions/private",
        Some(&alice_token),
        json!({ "target_user_id": searched_bob["user_id"] }),
    )
    .await;
    let session_id = private_session["data"]["session_id"]
        .as_i64()
        .expect("session id should be returned");
    assert_eq!(private_session["data"]["session_type"], "private");
    assert_eq!(
        private_session["data"]["peer_user_id"],
        bob["data"]["user_id"]
    );

    let empty_read = post_json(
        app.clone(),
        &format!("/api/sessions/{session_id}/read"),
        Some(&alice_token),
        json!({}),
    )
    .await;
    assert_eq!(empty_read["message"], "session marked as read");
    assert_eq!(empty_read["data"]["session_id"], session_id);
    assert!(empty_read["data"]["last_read_message_id"].is_null());

    let (addr, server) = spawn_server(state).await;
    let (mut alice_ws, _) = connect_async(format!("ws://{addr}/ws?token={alice_token}"))
        .await
        .expect("alice websocket should connect");
    let (mut bob_ws, _) = connect_async(format!("ws://{addr}/ws?token={bob_token}"))
        .await
        .expect("bob websocket should connect");

    assert_ws_event_type(&mut alice_ws, "connected").await;
    assert_ws_event_type(&mut bob_ws, "connected").await;

    alice_ws
        .send(WsMessage::Text(
            json!({
                "type": "send_message",
                "session_id": session_id,
                "content": "hello from integration test"
            })
            .to_string()
            .into(),
        ))
        .await
        .expect("alice should send websocket message");

    let ack = next_ws_json(&mut alice_ws).await;
    assert_eq!(ack["type"], "message_sent");
    assert_eq!(ack["message"]["session_id"], session_id);
    assert_eq!(ack["message"]["sender_id"], alice["data"]["user_id"]);
    assert_eq!(ack["message"]["content"], "hello from integration test");

    let pushed = next_ws_json(&mut bob_ws).await;
    assert_eq!(pushed["type"], "receive_message");
    assert_eq!(pushed["message"], ack["message"]);

    alice_ws.close(None).await.ok();
    bob_ws.close(None).await.ok();
    server.abort();

    let conversations = get_json(app.clone(), "/api/conversations", Some(&bob_token)).await;
    let conversation = conversations["data"]
        .as_array()
        .expect("conversations data should be an array")
        .iter()
        .find(|item| item["session_id"].as_i64() == Some(session_id))
        .expect("bob should see the private conversation");
    assert_eq!(conversation["session_name"], alice_username);
    assert_eq!(conversation["last_message"], "hello from integration test");
    assert_eq!(conversation["unread_count"], 1);

    let history = get_json(
        app.clone(),
        &format!("/api/messages?session_id={session_id}&limit=20"),
        Some(&bob_token),
    )
    .await;
    assert_eq!(history["data"]["session_id"], session_id);
    assert_eq!(
        history["data"]["messages"][0]["content"],
        "hello from integration test"
    );
    assert_eq!(
        history["data"]["messages"][0]["sender_username"],
        alice_username
    );

    let read = post_json(
        app.clone(),
        &format!("/api/sessions/{session_id}/read"),
        Some(&bob_token),
        json!({}),
    )
    .await;
    assert_eq!(read["message"], "session marked as read");
    assert_eq!(read["data"]["session_id"], session_id);
    assert_eq!(
        read["data"]["last_read_message_id"],
        history["data"]["messages"][0]["message_id"]
    );

    let conversations_after_read = get_json(app, "/api/conversations", Some(&bob_token)).await;
    let conversation_after_read = conversations_after_read["data"]
        .as_array()
        .expect("conversations data should be an array")
        .iter()
        .find(|item| item["session_id"].as_i64() == Some(session_id))
        .expect("bob should still see the private conversation");
    assert_eq!(conversation_after_read["unread_count"], 0);
}

async fn real_database_state() -> AppState {
    let url = std::env::var("TEST_DATABASE_URL")
        .expect("TEST_DATABASE_URL must be set for real database integration tests");

    AppState::build(AppConfig {
        app_host: "127.0.0.1".to_string(),
        app_port: 0,
        log_level: "debug".to_string(),
        database: Some(DatabaseConfig {
            url,
            max_connections: 5,
            min_connections: 1,
            acquire_timeout_secs: 5,
        }),
        jwt: JwtConfig {
            secret: "rustchat-integration-test-secret".to_string(),
            expires_in_secs: 3_600,
            issuer: "rustchat-integration-test".to_string(),
        },
    })
    .await
    .expect("app state should build with the real test database")
}

async fn reset_test_data(state: &AppState) {
    let storage = state
        .storage
        .as_ref()
        .expect("integration state should include storage");

    sqlx::query(
        r#"
        TRUNCATE TABLE
            user_session_read_state,
            messages,
            session_members,
            sessions,
            users
        RESTART IDENTITY CASCADE
        "#,
    )
    .execute(storage.pool())
    .await
    .expect("test database should be reset");
}

async fn spawn_server(state: AppState) -> (std::net::SocketAddr, JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("test server should bind");
    let addr = listener.local_addr().expect("test server should have addr");
    let server = tokio::spawn(async move {
        axum::serve(listener, create_router(state))
            .await
            .expect("test websocket server should run");
    });

    (addr, server)
}

async fn register_user(app: Router, username: &str, password: &str) -> Value {
    let response = post_json(
        app,
        "/api/register",
        None,
        json!({ "username": username, "password": password }),
    )
    .await;
    assert_eq!(response["message"], "user registered");
    response
}

async fn login_user(app: Router, username: &str, password: &str) -> String {
    let response = post_json(
        app,
        "/api/login",
        None,
        json!({ "username": username, "password": password }),
    )
    .await;
    assert_eq!(response["message"], "login succeeded");
    response["data"]["token"]
        .as_str()
        .expect("login should return token")
        .to_string()
}

async fn post_json(app: Router, uri: &str, token: Option<&str>, payload: Value) -> Value {
    let mut builder = Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(token) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }

    request_json(app, builder.body(Body::from(payload.to_string())).unwrap()).await
}

async fn get_json(app: Router, uri: &str, token: Option<&str>) -> Value {
    let mut builder = Request::builder().method("GET").uri(uri);
    if let Some(token) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }

    request_json(app, builder.body(Body::empty()).unwrap()).await
}

async fn request_json(app: Router, request: Request<Body>) -> Value {
    let response = app.oneshot(request).await.expect("request should complete");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    serde_json::from_slice(&body).expect("response body should be json")
}

async fn assert_ws_event_type<S>(socket: &mut S, expected_type: &str)
where
    S: StreamExt<Item = Result<WsMessage, tokio_tungstenite::tungstenite::Error>> + Unpin,
{
    let event = next_ws_json(socket).await;
    assert_eq!(event["type"], expected_type);
}

async fn next_ws_json<S>(socket: &mut S) -> Value
where
    S: StreamExt<Item = Result<WsMessage, tokio_tungstenite::tungstenite::Error>> + Unpin,
{
    let message = socket
        .next()
        .await
        .expect("websocket should yield a message")
        .expect("websocket message should be valid");
    let text = message
        .into_text()
        .expect("websocket message should be text");
    serde_json::from_str(&text).expect("websocket text should be json")
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_millis()
        % 1_000_000_000_000
}
