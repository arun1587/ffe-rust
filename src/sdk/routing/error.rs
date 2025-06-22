use thiserror::Error;

#[derive(Error, Debug)]
pub enum RoutingError {
    #[error("A point was not routable on the road network")]
    UnroutablePoint,

    #[error("Underlying request failed: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Failed to parse JSON response: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("API Error: {0}")]
    ApiError(String),

    #[error("Generic error: {0}")]
    Generic(String),
}
