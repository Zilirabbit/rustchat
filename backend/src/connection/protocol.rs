use axum::extract::ws::Message;
use serde::{Deserialize, Serialize};

use crate::message::dto::ChatMessagePayload;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientEvent {
    Ping,
    SendMessage {
        session_id: i64,
        content: String,
        client_message_id: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerEvent {
    Connected {
        user_id: i64,
        username: String,
        connection_id: u64,
    },
    Pong,
    MessageSent {
        message: ChatMessagePayload,
        #[serde(skip_serializing_if = "Option::is_none")]
        client_message_id: Option<String>,
    },
    ReceiveMessage {
        message: ChatMessagePayload,
    },
    Error {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        client_message_id: Option<String>,
    },
}

pub fn parse_client_event(payload: &str) -> Result<ClientEvent, serde_json::Error> {
    serde_json::from_str(payload)
}

pub fn server_event_message(event: &ServerEvent) -> Result<Message, serde_json::Error> {
    serde_json::to_string(event).map(|payload| Message::Text(payload.into()))
}

#[cfg(test)]
mod tests {
    use super::{ClientEvent, ServerEvent, parse_client_event, server_event_message};
    use crate::message::dto::ChatMessagePayload;

    #[test]
    fn ping_event_can_be_parsed() {
        let event = parse_client_event(r#"{"type":"ping"}"#).unwrap();
        assert_eq!(event, ClientEvent::Ping);
    }

    #[test]
    fn send_message_event_can_be_parsed() {
        let event =
            parse_client_event(r#"{"type":"send_message","session_id":12,"content":"hello"}"#)
                .unwrap();

        assert_eq!(
            event,
            ClientEvent::SendMessage {
                session_id: 12,
                content: "hello".to_string(),
                client_message_id: None,
            }
        );
    }

    #[test]
    fn send_message_event_can_include_client_message_id() {
        let event = parse_client_event(
            r#"{"type":"send_message","session_id":12,"content":"hello","client_message_id":"local-1"}"#,
        )
        .unwrap();

        assert_eq!(
            event,
            ClientEvent::SendMessage {
                session_id: 12,
                content: "hello".to_string(),
                client_message_id: Some("local-1".to_string()),
            }
        );
    }

    #[test]
    fn connected_event_uses_snake_case_protocol() {
        let message = server_event_message(&ServerEvent::Connected {
            user_id: 7,
            username: "alice".to_string(),
            connection_id: 99,
        })
        .unwrap();

        let text = message.into_text().unwrap();
        assert!(text.contains(r#""type":"connected""#));
        assert!(text.contains(r#""connection_id":99"#));
    }

    #[test]
    fn receive_message_event_uses_snake_case_protocol() {
        let message = server_event_message(&ServerEvent::ReceiveMessage {
            message: ChatMessagePayload {
                message_id: 3,
                session_id: 12,
                sender_id: 7,
                sender_username: "alice".to_string(),
                message_type: "text".to_string(),
                content: "hello".to_string(),
                created_at: "2026-05-03 12:00:00+00".to_string(),
                file_id: None,
                file_name: None,
                file_size: None,
                file_type: None,
            },
        })
        .unwrap();

        let text = message.into_text().unwrap();
        assert!(text.contains(r#""type":"receive_message""#));
        assert!(text.contains(r#""message_id":3"#));
    }

    #[test]
    fn message_sent_event_can_echo_client_message_id() {
        let message = server_event_message(&ServerEvent::MessageSent {
            message: ChatMessagePayload {
                message_id: 3,
                session_id: 12,
                sender_id: 7,
                sender_username: "alice".to_string(),
                message_type: "text".to_string(),
                content: "hello".to_string(),
                created_at: "2026-05-03 12:00:00+00".to_string(),
                file_id: None,
                file_name: None,
                file_size: None,
                file_type: None,
            },
            client_message_id: Some("local-1".to_string()),
        })
        .unwrap();

        let text = message.into_text().unwrap();
        assert!(text.contains(r#""type":"message_sent""#));
        assert!(text.contains(r#""client_message_id":"local-1""#));
    }

    #[test]
    fn file_message_event_includes_file_fields() {
        let message = server_event_message(&ServerEvent::ReceiveMessage {
            message: ChatMessagePayload {
                message_id: 10,
                session_id: 12,
                sender_id: 7,
                sender_username: "alice".to_string(),
                message_type: "file".to_string(),
                content: "report.pdf".to_string(),
                created_at: "2026-05-13 12:00:00+00".to_string(),
                file_id: Some(5),
                file_name: Some("report.pdf".to_string()),
                file_size: Some(2048000),
                file_type: Some("application/pdf".to_string()),
            },
        })
        .unwrap();

        let text = message.into_text().unwrap();
        assert!(text.contains(r#""type":"receive_message""#));
        assert!(text.contains(r#""message_type":"file""#));
        assert!(text.contains(r#""file_id":5"#));
        assert!(text.contains(r#""file_name":"report.pdf""#));
        assert!(text.contains(r#""file_size":2048000"#));
    }
}
