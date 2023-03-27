use std::result::Result;

use async_trait::async_trait;
use json::object;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::config::Config;
use crate::http_client::{HttpClient, HttpClientError};

const TOKEN_REQUEST_ENDPOINT_BASE: &str = "teiserver/api/request_token";

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("Session start error")]
    SessionStart(#[from] HttpClientError),
    #[error("Response deserialization error")]
    Deserialization(#[from] serde_json::Error),
}

#[derive(Deserialize, Serialize)]
struct SuccessfulTokenResponse {
    #[allow(dead_code)]
    result: String,
    token_value: String,
}

#[async_trait]
pub trait Server {
    async fn start_session(&mut self) -> Result<(), ServerError>;
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
            "https://{}/{}",
            self.config.get_server_domain(),
            TOKEN_REQUEST_ENDPOINT_BASE
        );

        let body = json::stringify(object! {
            "email": self.config.get_server_login_email(),
            "password": self.config.get_server_login_password(),
        });

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let response = self
            .http_client
            .post(&token_request_url, body, headers)
            .await?;

        let response: SuccessfulTokenResponse = serde_json::from_str(&response)?;

        Ok(response.token_value)
    }
}

#[async_trait]
impl Server for TeiServer<'_> {
    async fn start_session(&mut self) -> Result<(), ServerError> {
        if self.token.is_empty() {
            self.token = self.fetch_token().await?;
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
