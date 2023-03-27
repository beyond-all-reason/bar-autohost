use std::result::Result;

use reqwest::{header::HeaderMap, Client};

use async_trait::async_trait;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HttpClientError {
    #[error("Request failed")]
    RequestFailed(String),
}

#[async_trait]
pub trait HttpClient {
    async fn post(
        &self,
        url: &str,
        body: String,
        headers: HeaderMap,
    ) -> Result<String, HttpClientError>;
}

pub struct TeiHttpClient {
    client: Client,
}

impl TeiHttpClient {
    pub fn new() -> TeiHttpClient {
        TeiHttpClient {
            client: Client::new(),
        }
    }
}

#[async_trait]
impl HttpClient for TeiHttpClient {
    async fn post(
        &self,
        url: &str,
        body: String,
        headers: HeaderMap,
    ) -> Result<String, HttpClientError> {
        let response = self
            .client
            .post(url)
            .headers(headers)
            .body(body)
            .send()
            .await
            .map_err(|e| HttpClientError::RequestFailed(format!("{:?}", e)))?;

        let response = response
            .text()
            .await
            .map_err(|e| HttpClientError::RequestFailed(format!("{:?}", e)))?;
        Ok(response)
    }
}
