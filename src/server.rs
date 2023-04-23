use std::result::Result;

use async_trait::async_trait;
use json::object;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tungstenite::connect;
use urlencoding::encode;

use crate::config::Config;
use crate::http_client::{HttpClient, HttpClientError};

const ENDPOINT_BASE: &str = "teiserver/api";
const TOKEN_REQUEST_ENDPOINT: &str = "request_token";
const DISCONNECT_ENDPOINT: &str = "disconnect";
const CLIENT_NAME: &str = "bar-autohost";
const CLIENT_HASH: &str = "ef37ced34460ba9db08eeacc323f07386ad68402"; // sha1 hash
const TOKEN_TTL: u64 = 60 * 60 * 24;
const NOT_FOUND_RESPONSE: &str = "Not Found";

#[derive(Deserialize, Serialize)]
struct SuccessfulTokenResponse {
    #[allow(dead_code)]
    result: String,
    token_value: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct ErrorResponse {
    #[allow(dead_code)]
    detail: String,
}

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("Session start error")]
    SessionStart(String),
    #[error("Session end error")]
    SessionEnd(String),
    #[error("Response deserialization error")]
    Deserialization(#[from] serde_json::Error),
    #[error("Socket connection error")]
    Socket(#[from] tungstenite::Error),
}

#[async_trait]
pub trait Server {
    async fn start_session(&mut self) -> Result<(), ServerError>;
    async fn end_session(&mut self) -> Result<(), ServerError>;
}

pub struct TeiServer<'a> {
    config: &'a (dyn Config + Sync + Send),
    http_client: &'a (dyn HttpClient + Sync + Send),
    token: String,
}

impl<'a> TeiServer<'_> {
    pub fn new(
        config: &'a (dyn Config + Sync + Send),
        http_client: &'a (dyn HttpClient + Sync + Send),
    ) -> TeiServer<'a> {
        TeiServer {
            config,
            http_client,
            token: String::new(),
        }
    }

    async fn fetch_token(&mut self) -> Result<String, ServerError> {
        let token_request_url = format!(
            "https://{}/{}/{}",
            self.config.get_server_domain(),
            ENDPOINT_BASE,
            TOKEN_REQUEST_ENDPOINT,
        );

        let body = json::stringify(object! {
            "cmd": "c.auth.get_token",
            "email": self.config.get_server_login_email(),
            "password": self.config.get_server_login_password(),
            "ttl": TOKEN_TTL.to_string(),
        });

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let response = self
            .http_client
            .post(&token_request_url, body, headers)
            .await.map_err(|e| ServerError::SessionStart(format!("Token request failed: {:?}", e)))?;

        if let Ok(response) = serde_json::from_str::<SuccessfulTokenResponse>(&response) {
            Ok(response.token_value)
        } else if let Ok(error_response) = serde_json::from_str::<ErrorResponse>(&response) {
            Err(ServerError::SessionStart(format!("Error received for token request: {:?}", error_response)))
        } else {
            Err(ServerError::SessionStart(format!("Unknown response for token request: {:?}", response)))
        }

    }

    async fn disconnect(&mut self) -> Result<(), ServerError> {
        // TODO: Find out what is expected here. Currently returns "Not Found"
        let disconnect_url = format!(
            "https://{}/{}/{}",
            self.config.get_server_domain(),
            ENDPOINT_BASE,
            DISCONNECT_ENDPOINT,
        );

        let body = json::stringify(object! {
            "command": "disconnect",
        });

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let response = self
            .http_client
            .post(&disconnect_url, body, headers)
            .await.map_err(|e| ServerError::SessionEnd(format!("Disconnect request failed: {:?}", e)))?;

        Ok(())
    }
}

#[async_trait]
impl Server for TeiServer<'_> {
    async fn start_session(&mut self) -> Result<(), ServerError> {
        if self.token.is_empty() {
            self.token = self.fetch_token().await?;
        }

        let websock_server_url = format!(
            "wss://{}/tachyon/websocket/?token={}&client_hash={}&client_name={}",
            self.config.get_server_domain(),
            encode(&self.token),
            CLIENT_HASH,
            CLIENT_NAME,
        );

        let (_socket, response) = connect(websock_server_url)?;
        println!("{:?}", response);

        Ok(())
    }

    async fn end_session(&mut self) -> Result<(), ServerError> {
        if !self.token.is_empty() {
            self.disconnect().await?;
            self.token.clear();
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use async_trait::async_trait;

    use super::*;

    struct FakeConfig {
        pub spring_relative_path: String,
        pub start_script_relative_path: String,
        pub write_dir_relative_path: String,
        pub server_domain: String,
        pub server_login_email: String,
        pub server_login_password: String,
    }

    impl FakeConfig {
        fn new() -> FakeConfig {
            FakeConfig {
                spring_relative_path: "fake_string_relative_path".to_string(),
                start_script_relative_path: "fake_start_script_relative_path".to_string(),
                write_dir_relative_path: "fake_write_dir_relative_path".to_string(),
                server_domain: "fake_string_server_domain".to_string(),
                server_login_email: "fake_string_server_login_email".to_string(),
                server_login_password: "fake_server_login_password".to_string(),
            }
        }
    }

    impl Config for FakeConfig {
        fn get_spring_relative_path(&self) -> &str {
            &self.spring_relative_path
        }

        fn get_start_script_relative_path(&self) -> &str {
            &self.start_script_relative_path
        }

        fn get_write_dir_relative_path(&self) -> &str {
            &self.write_dir_relative_path
        }

        fn get_server_domain(&self) -> &str {
            &self.server_domain
        }

        fn get_server_login_email(&self) -> &str {
            &self.server_login_email
        }

        fn get_server_login_password(&self) -> &str {
            &self.server_login_password
        }
    }

    struct FakeHttpClient {
        response: Option<String>,
    }

    impl FakeHttpClient {
        fn build_with_successful_response(response: String) -> Self {
            FakeHttpClient {
                response: Some(response),
            }
        }

        fn build_with_failed_response() -> Self {
            FakeHttpClient { response: None }
        }
    }

    #[async_trait]
    impl HttpClient for FakeHttpClient {
        async fn post(
            &self,
            _url: &str,
            _body: String,
            _headers: HeaderMap,
        ) -> Result<String, HttpClientError> {
            if let Some(response) = &self.response {
                Ok(response.clone())
            } else {
                Err(HttpClientError::RequestFailed("Oh noes!".to_string()))
            }
        }
    }

    #[tokio::test]
    async fn test_session_start_fails_when_http_client_returns_error() {
        let config = FakeConfig::new();

        let http_client = FakeHttpClient::build_with_failed_response();

        let mut server = TeiServer::new(&config, &http_client);

        let result = server.start_session().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_session_start_succeeds() {
        let config = FakeConfig::new();

        let successful_server_response = SuccessfulTokenResponse {
            token_value: "fake_token".to_string(),
            result: "fake_result".to_string(),
        };

        let successful_server_response =
            serde_json::to_string(&successful_server_response).unwrap();

        let http_client =
            FakeHttpClient::build_with_successful_response(successful_server_response);

        let mut server = TeiServer::new(&config, &http_client);

        let result = server.start_session().await;
        assert!(result.is_ok());
    }
}
