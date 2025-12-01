use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Request types that can be sent to the server
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Request {
    /// Get a specific environment variable
    GetEnv { name: String },
    /// Get all environment variables
    GetAllEnv,
    /// Set an environment variable (for PowerShell to set in its session)
    SetEnv { name: String, value: String },
    /// Execute arbitrary data transfer
    SendData { key: String, data: String },
    /// Ping to check if server is alive
    Ping,
}

/// Response types sent back from the server
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Response {
    /// Success with optional data
    Success { data: Option<String> },
    /// Success with environment variables
    EnvVars { vars: HashMap<String, String> },
    /// Error response
    Error { message: String },
    /// Pong response to ping
    Pong,
}

impl Response {
    pub fn success(data: Option<String>) -> Self {
        Response::Success { data }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Response::Error {
            message: message.into(),
        }
    }

    pub fn env_vars(vars: HashMap<String, String>) -> Self {
        Response::EnvVars { vars }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_serialization() {
        let request = Request::GetEnv {
            name: "PATH".to_string(),
        };
        let json = serde_json::to_string(&request).unwrap();
        let deserialized: Request = serde_json::from_str(&json).unwrap();

        match deserialized {
            Request::GetEnv { name } => assert_eq!(name, "PATH"),
            _ => panic!("Wrong request type"),
        }
    }

    #[test]
    fn test_response_serialization() {
        let response = Response::success(Some("test_value".to_string()));
        let json = serde_json::to_string(&response).unwrap();
        let deserialized: Response = serde_json::from_str(&json).unwrap();

        match deserialized {
            Response::Success { data } => assert_eq!(data, Some("test_value".to_string())),
            _ => panic!("Wrong response type"),
        }
    }
}
