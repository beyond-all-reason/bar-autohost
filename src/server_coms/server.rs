use std::result::Result;

use async_trait::async_trait;
use serde::Serialize;
use urlencoding::encode;

use crate::utils::config::Config;
use crate::utils::http_client::HttpClient;
use crate::utils::http_request;
use crate::utils::websocket_client::WebsocketClient;

use super::responses::{ErrorResponse, SuccessfulTokenResponse};
use super::server_error::ServerError;

const ENDPOINT_BASE: &str = "teiserver/api";
const TOKEN_REQUEST_ENDPOINT: &str = "request_token";
const DISCONNECT_ENDPOINT: &str = "disconnect";
const CLIENT_NAME: &str = "bar-autohost";
const CLIENT_HASH: &str = "ef37ced34460ba9db08eeacc323f07386ad68402"; // sha1 hash
const TOKEN_TTL: u64 = 60 * 60 * 24;

#[derive(Serialize)]
struct Authenticate<'a> {
    pub cmd: &'a str,
    pub email: &'a str,
    pub password: &'a str,
    pub ttl: &'a str,
}

#[derive(Serialize)]
struct Disconnect {
    command: String,
}

#[async_trait]
pub trait Server {
    async fn start_session(&mut self) -> Result<(), ServerError>;
    async fn end_session(&mut self) -> Result<(), ServerError>;
}

pub struct TeiServer<'a> {
    config: &'a (dyn Config + Sync + Send),
    http_client: &'a (dyn HttpClient + Sync + Send),
    socket_client: &'a mut (dyn WebsocketClient + Sync + Send),
    token: String,
}

impl<'a> TeiServer<'_> {
    pub fn new(
        config: &'a (dyn Config + Sync + Send),
        http_client: &'a (dyn HttpClient + Sync + Send),
        socket_client: &'a mut (dyn WebsocketClient + Sync + Send),
    ) -> TeiServer<'a> {
        TeiServer {
            config,
            http_client,
            socket_client,
            token: String::new(),
        }
    }

    async fn fetch_token(&mut self) -> Result<String, ServerError> {
        let authenticate_endpoint_url = format!(
            "https://{}/{}/{}",
            self.config.get_server_domain(),
            ENDPOINT_BASE,
            TOKEN_REQUEST_ENDPOINT,
        );

        let authenticate_request = Authenticate {
            cmd: "c.auth.get_token",
            email: self.config.get_server_login_email(),
            password: self.config.get_server_login_password(),
            ttl: &TOKEN_TTL.to_string(),
        };

        let response = http_request::post(
            self.http_client,
            &authenticate_endpoint_url,
            &authenticate_request,
        )
        .await
        .map_err(|e| ServerError::SessionStart(format!("Token request failed: {:?}", e)))?;

        if let Ok(response) = serde_json::from_str::<SuccessfulTokenResponse>(&response) {
            Ok(response.token_value)
        } else if let Ok(error_response) = serde_json::from_str::<ErrorResponse>(&response) {
            Err(ServerError::SessionStart(format!(
                "Error received for token request: {:?}",
                error_response
            )))
        } else {
            Err(ServerError::SessionStart(format!(
                "Unknown response for token request: {:?}",
                response
            )))
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

        let disconnect_request = Disconnect {
            command: "disconnect".to_string(),
        };

        http_request::post(self.http_client, &disconnect_url, &disconnect_request)
            .await
            .map_err(|e| {
                ServerError::SessionEnd(format!("Error occured while ending session: {:?}", e))
            })?;

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

        self.socket_client
            .connect(&websock_server_url)
            .map_err(|e| {
                ServerError::SessionStart(format!("Error occured while starting session: {:?}", e))
            })?;

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
    use reqwest::header::HeaderMap;

    use crate::utils::http_client::HttpClientError;
    use crate::utils::websocket_client::WebsocketError;

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

    struct FakeWebsocketClient {
        should_connect: bool,
    }

    impl FakeWebsocketClient {
        fn build(should_connect: bool) -> Self {
            FakeWebsocketClient { should_connect }
        }
    }

    impl WebsocketClient for FakeWebsocketClient {
        fn connect(&mut self, _server_url: &str) -> Result<(), WebsocketError> {
            if self.should_connect {
                Ok(())
            } else {
                Err(WebsocketError::Connection(
                    "Something went wrong!".to_string(),
                ))
            }
        }
    }

    #[tokio::test]
    async fn test_session_start_fails_when_http_client_returns_error() {
        let config = FakeConfig::new();

        let http_client = FakeHttpClient::build_with_failed_response();

        let mut websock_client = FakeWebsocketClient::build(false);

        let mut server = TeiServer::new(&config, &http_client, &mut websock_client);

        let result = server.start_session().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_session_start_fails_when_websock_client_returns_error() {
        let config = FakeConfig::new();

        let successful_server_response = SuccessfulTokenResponse {
            token_value: "fake_token".to_string(),
            result: "fake_result".to_string(),
        };

        let successful_server_response =
            serde_json::to_string(&successful_server_response).unwrap();

        let http_client =
            FakeHttpClient::build_with_successful_response(successful_server_response);

        let mut websock_client = FakeWebsocketClient::build(false);

        let mut server = TeiServer::new(&config, &http_client, &mut websock_client);

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

        let mut websock_client = FakeWebsocketClient::build(true);

        let mut server = TeiServer::new(&config, &http_client, &mut websock_client);

        let result = server.start_session().await;
        assert!(result.is_ok());
    }
}
