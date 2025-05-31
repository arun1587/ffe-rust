use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use crate::sdk::routing::cache::{GeoCache, CityPairKey};
use crate::sdk::routing::geocode::{geocode_city, get_routable_coordinates};
use crate::sdk::util::rate_limit::DIRECTIONS_LIMITER;


#[derive(Debug, Clone)]
pub struct RouteSummary {
    pub distance_km: f64,
    pub duration_hours: f64,
}

#[derive(Deserialize)]
struct Response {
    routes: Vec<Route>,
}

#[derive(Deserialize)]
struct Route {
    summary: Summary,
}

#[derive(Deserialize)]
struct Summary {
    distance: f64,
    duration: f64,
}

pub async fn get_road_distance(
    city1: &str,
    city2: &str,
    api_key: &str,
    cache: &mut GeoCache,
    client: &Client,
) -> Result<RouteSummary> {
    let key = CityPairKey {
        origin: city1.to_string(),
        destination: city2.to_string(),
    };

    if let Some(&(dist, dur)) = cache.routes.get(&key) {
        log::debug!("[CACHE HIT] {} → {}", city1, city2);
        return Ok(RouteSummary {
            distance_km: dist,
            duration_hours: dur,
        });
    }


    let (mut lon1, mut lat1) = geocode_city(city1, api_key, cache, client).await?;
    let (mut lon2, mut lat2) = geocode_city(city2, api_key, cache, client).await?;

    let url = "https://api.openrouteservice.org/v2/directions/driving-car";

    let body = serde_json::json!({
        "coordinates": [
            [lon1, lat1],
            [lon2, lat2]
        ]
    });

    let mut attempt = 0;
    let max_attempts = 2;

    while attempt < max_attempts {
        log::info!("Waiting for directions limiter before making API cal get_road_distance...");

        DIRECTIONS_LIMITER.until_ready().await;

        let resp = client
            .post(url)
            .header("Authorization", api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to send request to ORS")?;

        let text = resp
            .text()
            .await
            .context("Failed to read response text")?;

        if !text.contains("\"error\"") {
            let route: Response = serde_json::from_str(&text)
                .context("Failed to parse route response")?;

            let summary = &route.routes[0].summary;

            let result = RouteSummary {
                distance_km: summary.distance / 1000.0,
                duration_hours: summary.duration / 3600.0,
            };

            cache.routes.insert(key, (result.distance_km, result.duration_hours));
            return Ok(result);
        }

        if attempt == 0 && text.contains("Could not find routable point") {
            log::warn!(
                "Unroutable point detected. Trying snapped coordinates for {} → {}\nlon1={}, lat1={}, lon2={}, lat2={}",
                city1, city2, lon1, lat1, lon2, lat2
            );

            (lon1, lat1) = get_routable_coordinates(lon1, lat1, api_key, client).await?;
            (lon2, lat2) = get_routable_coordinates(lon2, lat2, api_key, client).await?;
        } else {
            log::error!(
                "Routing failed for {} → {}\n[RESPONSE] {}\n[Cords] [{},{}] → [{},{}]",
                city1, city2, text, lon1, lat1, lon2, lat2
            );
            return Err(anyhow::anyhow!("Routing failed for {} → {}", city1, city2));
        }

        attempt += 1;
    }

    Err(anyhow::anyhow!(
        "Routing failed after retry with snapped coordinates."
    ))
}
