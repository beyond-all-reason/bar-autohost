use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct SuccessfulTokenResponse {
    #[allow(dead_code)]
    pub result: String,
    pub token_value: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ErrorResponse {
    #[allow(dead_code)]
    pub detail: String,
}
