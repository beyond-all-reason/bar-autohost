use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::Serialize;
use thiserror::Error;

use super::http_client::HttpClient;

#[derive(Error, Debug)]
pub enum HttpRequestError {
    #[error("Serialization error")]
    Serialization(String),
    #[error("Request error")]
    Request(String),
}

pub async fn post<T: Serialize>(
    http_client: &(dyn HttpClient + Sync + Send),
    endpoint_url: &str,
    body: &T,
) -> Result<String, HttpRequestError> {
    let body = serde_json::to_string(body).map_err(|e| {
        HttpRequestError::Serialization(format!(
            "Error occured during request serialization: {:?}",
            e
        ))
    })?;

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    http_client
        .post(endpoint_url, body, headers)
        .await
        .map_err(|e| HttpRequestError::Request(format!("Request failed: {:?}", e)))
}
