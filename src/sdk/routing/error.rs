use serde::Deserialize;
use thiserror::Error;

// Helper structs to parse the JSON error response from ORS
#[derive(Deserialize, Debug)]
pub struct OrsErrorDetail {
    pub code: u32,
    pub message: String,
}
#[derive(Deserialize, Debug)]
pub struct OrsErrorPayload {
    pub error: OrsErrorDetail,
}

#[derive(Error, Debug)]
pub enum RoutingError {
    #[error("A point was not routable on the road network")]
    UnroutablePoint,

    // This variant hold the structured error from the API
    #[error("API Error (Code {code}): {message}")]
    ApiError { code: u32, message: String },

    // A fallback for when we get an error that isn't in the expected JSON format
    #[error("Unstructured API Error: {0}")]
    RawApiError(String),

    #[error("Underlying request failed: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Failed to parse JSON response: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("Generic error: {0}")]
    Generic(String),
}
