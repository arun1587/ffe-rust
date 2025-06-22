use std::env;

pub enum OrsConfig {
    Remote {
        api_key: String,
    },
    Local {
        base_url: String,
    },
    Hybrid {
        api_key: String,
        local_base_url: String,
    },
}

impl OrsConfig {
    /// Creates configuration from environment variables.
    /// Priority:
    /// 1. If ORS_LOCAL_URL is set, use local provider.
    /// 2. Else, if ORS_API_KEY is set, use remote provider.
    /// 3. Else, return an error.
    pub fn from_env() -> Result<Self, String> {
        let local_url = env::var("ORS_LOCAL_URL");
        let api_key = env::var("ORS_API_KEY");

        match (local_url, api_key) {
            // 1. If BOTH are set, use Hybrid mode.
            (Ok(local_base_url), Ok(api_key)) => {
                log::info!("Using Hybrid mode: LOCAL for routing, REMOTE for geocoding.");
                Ok(OrsConfig::Hybrid {
                    api_key,
                    local_base_url,
                })
            }
            // 2. If only local is set, use Local mode.
            (Ok(base_url), Err(_)) => {
                log::info!("Using local-only OpenRouteService instance at {}", base_url);
                Ok(OrsConfig::Local { base_url })
            }
            // 3. If only remote is set, use Remote mode.
            (Err(_), Ok(api_key)) => {
                log::info!("Using remote-only OpenRouteService API");
                Ok(OrsConfig::Remote { api_key })
            }
            // 4. If neither is set, error out.
            (Err(_), Err(_)) => Err(
                "Missing ORS configuration: Please set ORS_LOCAL_URL and/or ORS_API_KEY"
                    .to_string(),
            ),
        }
    }
}
