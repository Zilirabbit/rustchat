use axum::extract::ws::Message;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientEvent {
    Ping,
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
    Error {
        message: String,
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

    #[test]
    fn ping_event_can_be_parsed() {
        let event = parse_client_event(r#"{"type":"ping"}"#).unwrap();
        assert_eq!(event, ClientEvent::Ping);
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
}
