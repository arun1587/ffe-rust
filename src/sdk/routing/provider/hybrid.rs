use crate::sdk::routing::cache::Coord;
use crate::sdk::routing::route::RouteSummary;
use crate::sdk::routing::service::RoutingProvider;
use crate::sdk::util::rate_limit::Limiter;
use std::error::Error;

use super::local::LocalOrsProvider;
use super::remote::RemoteOrsProvider;

pub struct HybridOrsProvider {
    remote: RemoteOrsProvider,
    local: LocalOrsProvider,
}

impl HybridOrsProvider {
    pub fn new(api_key: String, limiter: Limiter, local_base_url: String) -> Self {
        Self {
            remote: RemoteOrsProvider::new(api_key, limiter),
            local: LocalOrsProvider::new(local_base_url),
        }
    }
}

impl RoutingProvider for HybridOrsProvider {
    fn geocode(&self, city: &str) -> Result<Coord, Box<dyn Error>> {
        log::debug!("[Hybrid Provider] Using REMOTE for geocode");
        self.remote.geocode(city)
    }

    fn reverse_geocode(&self, coord: Coord) -> Result<Vec<Coord>, Box<dyn Error>> {
        log::debug!("[Hybrid Provider] Using REMOTE for reverse_geocode");
        self.remote.reverse_geocode(coord)
    }

    fn is_routable(&self, coord: Coord) -> Result<bool, Box<dyn Error>> {
        log::debug!("[Hybrid Provider] Using LOCAL for is_routable");
        self.local.is_routable(coord)
    }

    fn get_directions(&self, start: Coord, end: Coord) -> Result<RouteSummary, Box<dyn Error>> {
        log::debug!("[Hybrid Provider] Using LOCAL for get_directions");
        self.local.get_directions(start, end)
    }
}
