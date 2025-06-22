use serde::Deserialize;

// --- Data Structures for parsing ORS responses, now public ---

#[derive(Deserialize)]
pub struct GeoResponse {
    pub features: Vec<Feature>,
}
#[derive(Deserialize)]
pub struct Feature {
    pub geometry: Geometry,
}
#[derive(Deserialize)]
pub struct Geometry {
    pub coordinates: [f64; 2],
}

#[derive(Deserialize)]
pub struct DirectionsResponse {
    pub routes: Vec<Route>,
}
#[derive(Deserialize)]
pub struct Route {
    pub summary: DirectionsSummary,
}
#[derive(Deserialize, Clone, Copy)]
pub struct DirectionsSummary {
    pub distance: f64,
    pub duration: f64,
}
