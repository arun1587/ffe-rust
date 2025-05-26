use std::env;

pub fn get_ors_api_key() -> Result<String, Box<dyn std::error::Error>> {
    env::var("ORS_API_KEY").map_err(|e| e.into())
}
