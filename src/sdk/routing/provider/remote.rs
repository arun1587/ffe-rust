use super::types::{DirectionsResponse, GeoResponse};
use crate::sdk::routing::cache::Coord;
use crate::sdk::routing::error::{OrsErrorPayload, RoutingError};
use crate::sdk::routing::route::RouteSummary;
use crate::sdk::routing::service::RoutingProvider;
use crate::sdk::util::rate_limit::Limiter;
use reqwest::blocking::Client;
use serde_json::json;
use std::error::Error;
use std::time::Duration;

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
        log::debug!("[PROVIDER] Calling remote geocode for city: \"{}\"", city);

        let response = self.client.get(&url).send()?;
        let text = response.text()?;

        let resp: GeoResponse = serde_json::from_str(&text).map_err(|e| {
            log::error!(
                "Failed to parse GeoResponse. URL: {}\nError: {}. Body: {}",
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
        self.limiter.wait();
        let url = format!(
            "{}/geocode/reverse?point.lon={}&point.lat={}&api_key={}",
            self.base_url, coord.0, coord.1, self.api_key
        );
        log::debug!(
            "[PROVIDER] Calling remote reverse_geocode for coord: {:?}",
            coord
        );

        let response = self.client.get(&url).send()?;
        let text = response.text()?;

        let body: GeoResponse = serde_json::from_str(&text).map_err(|e| {
            log::error!(
                "Failed to parse GeoResponse. URL: {}\nError: {}. Body: {}",
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
        self.limiter.wait();
        log::debug!(
            "[PROVIDER] Calling remote is_routable for coord: {:?}",
            coord
        );
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
            return Ok(RouteSummary {
                distance_km: 0.0,
                duration_hours: 0.0,
            });
        }

        self.limiter.wait();
        log::debug!(
            "[PROVIDER] Calling remote get_directions for {:?} -> {:?}",
            start,
            end
        );
        let url = format!("{}/v2/directions/driving-car", self.base_url);
        let body = json!({ "coordinates": [[start.0, start.1], [end.0, end.1]] });

        let response = match self
            .client
            .post(&url)
            .header("Authorization", &self.api_key)
            .json(&body)
            .send()
        {
            Ok(resp) => resp,
            Err(e) => {
                log::error!(
                    "Failed to send POST request. URL: {}\nBody: {}\nError: {}",
                    url,
                    serde_json::to_string_pretty(&body).unwrap_or_default(),
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
                "Failed to parse DirectionsResponse. URL: {}\nError: {}. Body: {}",
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
