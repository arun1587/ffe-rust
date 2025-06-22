use super::cache::Coord;
use super::route::RouteSummary;
use std::error::Error;

pub trait RoutingProvider: Send + Sync {
    /// Geocodes a city name to a coordinate.
    fn geocode(&self, city: &str) -> Result<Coord, Box<dyn Error>>;

    /// Finds potential coordinates near a given point.
    fn reverse_geocode(&self, coord: Coord) -> Result<Vec<Coord>, Box<dyn Error>>;

    /// Checks if a specific coordinate is on the routable road network.
    fn is_routable(&self, coord: Coord) -> Result<bool, Box<dyn Error>>;

    /// Gets directions between two points.
    fn get_directions(&self, start: Coord, end: Coord) -> Result<RouteSummary, Box<dyn Error>>;
}
