use std::env;

pub enum OrsConfig {
    Remote { api_key: String },
    Local { base_url: String },
}

impl OrsConfig {
    /// Creates configuration from environment variables.
    /// Priority:
    /// 1. If ORS_LOCAL_URL is set, use local provider.
    /// 2. Else, if ORS_API_KEY is set, use remote provider.
    /// 3. Else, return an error.
    pub fn from_env() -> Result<Self, String> {
        if let Ok(base_url) = env::var("ORS_LOCAL_URL") {
            Ok(OrsConfig::Local { base_url })
        } else if let Ok(api_key) = env::var("ORS_API_KEY") {
            Ok(OrsConfig::Remote { api_key })
        } else {
            Err(
                "Missing ORS configuration: Please set either ORS_LOCAL_URL or ORS_API_KEY"
                    .to_string(),
            )
        }
    }
}
