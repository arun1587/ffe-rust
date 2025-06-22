use super::types::{DirectionsResponse, GeoResponse};
use crate::sdk::routing::cache::Coord;
use crate::sdk::routing::error::{OrsErrorPayload, RoutingError};
use crate::sdk::routing::route::RouteSummary;
use crate::sdk::routing::service::RoutingProvider;
use reqwest::blocking::Client;
use serde_json::json;
use std::error::Error;
use std::time::Duration;

pub struct LocalOrsProvider {
    client: Client,
    base_url: String,
}

impl LocalOrsProvider {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(15))
                .build()
                .unwrap(),
            base_url,
        }
    }
}

impl RoutingProvider for LocalOrsProvider {
    fn geocode(&self, city: &str) -> Result<Coord, Box<dyn Error>> {
        log::debug!("[PROVIDER] Calling local geocode for city: \"{}\"", city);
        let url = format!("{}/pelias/v1/search?text={}", self.base_url, city);

        let response = self.client.get(&url).send()?;
        let text = response.text()?;

        let resp: GeoResponse = serde_json::from_str(&text).map_err(|e| {
            log::error!(
                "Failed to parse local GeoResponse. URL: {}\nError: {}. Body: {}",
                url,
                e,
                text
            );
            e
        })?;

        let coords = resp
            .features
            .first()
            .ok_or_else(|| RoutingError::Generic(format!("No geocode results for city: {}", city)))?
            .geometry
            .coordinates;
        Ok((coords[0], coords[1]))
    }

    fn reverse_geocode(&self, coord: Coord) -> Result<Vec<Coord>, Box<dyn Error>> {
        log::debug!(
            "[PROVIDER] Calling local reverse_geocode for coord: {:?}",
            coord
        );
        let url = format!(
            "{}/pelias/v1/reverse?point.lon={}&point.lat={}",
            self.base_url, coord.0, coord.1
        );

        let response = self.client.get(&url).send()?;
        let text = response.text()?;

        let body: GeoResponse = serde_json::from_str(&text).map_err(|e| {
            log::error!(
                "Failed to parse local GeoResponse. URL: {}\nError: {}. Body: {}",
                url,
                e,
                text
            );
            e
        })?;

        Ok(body
            .features
            .into_iter()
            .map(|f| (f.geometry.coordinates[0], f.geometry.coordinates[1]))
            .collect())
    }

    fn is_routable(&self, coord: Coord) -> Result<bool, Box<dyn Error>> {
        log::debug!(
            "[PROVIDER] Calling local is_routable for coord: {:?}",
            coord
        );
        let url = format!("{}/v2/directions/driving-car", self.base_url);
        let body = json!({ "coordinates": [[coord.0, coord.1], [coord.0, coord.1]] });

        let response = self.client.post(url).json(&body).send()?;
        Ok(response.status().is_success())
    }

    fn get_directions(&self, start: Coord, end: Coord) -> Result<RouteSummary, Box<dyn Error>> {
        if start == end {
            return Ok(RouteSummary {
                distance_km: 0.0,
                duration_hours: 0.0,
            });
        }

        log::debug!(
            "[PROVIDER] Calling local get_directions for {:?} -> {:?}",
            start,
            end
        );
        let url = format!("{}/v2/directions/driving-car", self.base_url);
        let body = json!({ "coordinates": [[start.0, start.1], [end.0, end.1]] });

        let response = match self.client.post(&url).json(&body).send() {
            Ok(resp) => resp,
            Err(e) => {
                log::error!(
                    "Failed to send POST request to local ORS. URL: {}\nError: {}",
                    url,
                    e
                );
                return Err(Box::new(e));
            }
        };

        let status = response.status();
        let text = response.text()?;

        if !status.is_success() {
            // Try to parse the structured error first
            if let Ok(payload) = serde_json::from_str::<OrsErrorPayload>(&text) {
                return Err(Box::new(RoutingError::ApiError {
                    code: payload.error.code,
                    message: payload.error.message,
                }));
            } else {
                // Fallback to a raw error if parsing fails
                log::error!(
                    "API returned non-success status: {}. Unparseable Body: {}",
                    status,
                    text
                );
                return Err(Box::new(RoutingError::RawApiError(text)));
            }
        }

        let route_response: DirectionsResponse = serde_json::from_str(&text).map_err(|e| {
            log::error!(
                "Failed to parse local DirectionsResponse. URL: {}\nError: {}. Body: {}",
                url,
                e,
                text
            );
            e
        })?;

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
