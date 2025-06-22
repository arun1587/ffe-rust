pub mod cache;
pub mod error;
pub mod geocode;
pub mod provider;
pub mod route;
pub mod service;

pub use cache::{Coord, GeoCache};
pub use error::RoutingError;
pub use geocode::{find_routable_coordinates, get_or_cache_geocode};
// This line works because providers/mod.rs re-exports them
pub use provider::{HybridOrsProvider, LocalOrsProvider, RemoteOrsProvider};
pub use route::{RouteSummary, get_road_distance};
pub use service::RoutingProvider;
