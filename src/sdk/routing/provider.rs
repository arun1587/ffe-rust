use super::cache::Coord;
use super::error::RoutingError;
use super::route::RouteSummary;
use super::service::RoutingProvider;
use crate::sdk::util::rate_limit::Limiter;
use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::json;
use std::error::Error;
use std::time::Duration;

// --- Data Structures for parsing ORS responses ---
#[derive(Deserialize)]
struct GeoResponse {
    features: Vec<Feature>,
}
#[derive(Deserialize)]
struct Feature {
    geometry: Geometry,
}
#[derive(Deserialize)]
struct Geometry {
    coordinates: [f64; 2],
}

#[derive(Deserialize)]
struct DirectionsResponse {
    routes: Vec<Route>,
}
#[derive(Deserialize)]
struct Route {
    summary: DirectionsSummary,
}
#[derive(Deserialize, Clone, Copy)]
struct DirectionsSummary {
    distance: f64,
    duration: f64,
}

// --- Remote Provider Implementation ---
pub struct RemoteOrsProvider {
    client: Client,
    api_key: String,
    base_url: String,
    limiter: Limiter,
}

impl RemoteOrsProvider {
    pub fn new(api_key: String, limiter: Limiter) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(15))
                .build()
                .unwrap(),
            api_key,
            base_url: "https://api.openrouteservice.org".to_string(),
            limiter,
        }
    }
}

impl RoutingProvider for RemoteOrsProvider {
    fn geocode(&self, city: &str) -> Result<Coord, Box<dyn Error>> {
        self.limiter.wait();
        let url = format!(
            "{}/geocode/search?api_key={}&text={}",
            self.base_url, self.api_key, city
        );
        let resp: GeoResponse = self.client.get(&url).send()?.json()?;
        let coords = resp
            .features
            .first()
            .ok_or_else(|| RoutingError::Generic(format!("No geocode results for city: {}", city)))?
            .geometry
            .coordinates;
        Ok((coords[0], coords[1]))
    }

    fn reverse_geocode(&self, coord: Coord) -> Result<Vec<Coord>, Box<dyn Error>> {
        self.limiter.wait();
        let url = format!(
            "{}/geocode/reverse?point.lon={}&point.lat={}&api_key={}",
            self.base_url, coord.0, coord.1, self.api_key
        );
        let body: GeoResponse = self.client.get(&url).send()?.json()?;
        Ok(body
            .features
            .into_iter()
            .map(|f| (f.geometry.coordinates[0], f.geometry.coordinates[1]))
            .collect())
    }

    fn is_routable(&self, coord: Coord) -> Result<bool, Box<dyn Error>> {
        self.limiter.wait();
        let url = format!("{}/v2/directions/driving-car", self.base_url);
        let body = json!({ "coordinates": [[coord.0, coord.1], [coord.0, coord.1]] });
        let response = self
            .client
            .post(url)
            .header("Authorization", &self.api_key)
            .json(&body)
            .send()?;
        Ok(response.status().is_success())
    }

    fn get_directions(&self, start: Coord, end: Coord) -> Result<RouteSummary, Box<dyn Error>> {
        if start == end {
            log::debug!("Start and end coordinates are identical. Returning zero route.");
            return Ok(RouteSummary {
                distance_km: 0.0,
                duration_hours: 0.0,
            });
        }

        self.limiter.wait();
        let url = format!("{}/v2/directions/driving-car", self.base_url);
        let body = json!({ "coordinates": [[start.0, start.1], [end.0, end.1]] });
        let response = self
            .client
            .post(url)
            .header("Authorization", &self.api_key)
            .json(&body)
            .send()?;
        let status = response.status();
        let text = response.text()?;

        if !status.is_success() {
            if text.contains("2010") {
                return Err(Box::new(RoutingError::UnroutablePoint));
            }
            return Err(Box::new(RoutingError::ApiError(text)));
        }

        let route_response: DirectionsResponse = serde_json::from_str(&text)?;
        let summary = route_response
            .routes
            .first()
            .ok_or_else(|| RoutingError::Generic("No route found in success response".to_string()))?
            .summary;

        Ok(RouteSummary {
            distance_km: summary.distance / 1000.0,
            duration_hours: summary.duration / 3600.0,
        })
    }
}
