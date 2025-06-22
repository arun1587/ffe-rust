use super::route::RouteSummary;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt, fs, io::Result as IoResult, path::Path, str::FromStr};

pub type Coord = (f64, f64);

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Clone, Debug)]
pub struct CityPairKey {
    pub origin: String,
    pub destination: String,
}

impl CityPairKey {
    /// Creates a new, canonical CityPairKey where cities are always sorted alphabetically.
    pub fn new(city1: &str, city2: &str) -> Self {
        let mut cities = [city1, city2];
        cities.sort_unstable();
        Self {
            origin: cities[0].to_string(),
            destination: cities[1].to_string(),
        }
    }
}

impl fmt::Display for CityPairKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}::{}", self.origin, self.destination)
    }
}

impl FromStr for CityPairKey {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split("::").collect();
        if parts.len() == 2 {
            Ok(CityPairKey {
                origin: parts[0].to_string(),
                destination: parts[1].to_string(),
            })
        } else {
            Err("Invalid CityPairKey format")
        }
    }
}

// --- Serde Helper for Complex Key ---
mod city_pair_map {
    use super::{CityPairKey, RouteSummary};
    use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error};
    use std::{collections::HashMap, str::FromStr};

    pub fn serialize<S: Serializer>(
        map: &HashMap<CityPairKey, RouteSummary>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let string_map: HashMap<String, &RouteSummary> =
            map.iter().map(|(k, v)| (k.to_string(), v)).collect();
        string_map.serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<HashMap<CityPairKey, RouteSummary>, D::Error> {
        // This line now works because the `Deserialize` trait is in scope
        let string_map = HashMap::<String, RouteSummary>::deserialize(deserializer)?;
        string_map
            .into_iter()
            .map(|(k, v)| Ok((CityPairKey::from_str(&k).map_err(Error::custom)?, v)))
            .collect()
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct GeoCache {
    geocodes: HashMap<String, Coord>,
    #[serde(with = "city_pair_map")]
    routes: HashMap<CityPairKey, RouteSummary>,
}

impl GeoCache {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> IoResult<Self> {
        if path.as_ref().exists() {
            let data = fs::read_to_string(path)?;
            Ok(serde_json::from_str(&data)?)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> IoResult<()> {
        let data = serde_json::to_string_pretty(self)?;
        fs::write(path, data)
    }

    pub fn get_geocode(&self, city: &str) -> Option<Coord> {
        self.geocodes.get(city).copied()
    }

    pub fn insert_geocode(&mut self, city: &str, coord: Coord) {
        self.geocodes.insert(city.to_string(), coord);
    }

    pub fn get_route(&self, key: &CityPairKey) -> Option<RouteSummary> {
        self.routes.get(key).copied()
    }

    pub fn insert_route(&mut self, key: CityPairKey, summary: RouteSummary) {
        self.routes.insert(key, summary);
    }
}
