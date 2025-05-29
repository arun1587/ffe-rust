use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    hash::Hash,
    io::Result as IoResult,
    path::Path,
};
use std::fmt;
use std::str::FromStr;

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct CityPairKey {
    pub origin: String,
    pub destination: String,
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

mod city_pair_map {
    use super::CityPairKey;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::collections::HashMap;
    use std::str::FromStr;

    pub fn serialize<S>(
        map: &HashMap<CityPairKey, (f64, f64)>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let string_map: HashMap<String, &(f64, f64)> = map
            .iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();
        string_map.serialize(serializer)
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<HashMap<CityPairKey, (f64, f64)>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string_map: HashMap<String, (f64, f64)> = HashMap::deserialize(deserializer)?;
        let mut map = HashMap::new();
        for (k, v) in string_map {
            let key = CityPairKey::from_str(&k).map_err(serde::de::Error::custom)?;
            map.insert(key, v);
        }
        Ok(map)
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct GeoCache {
    pub geocodes: HashMap<String, (f64, f64)>,
    #[serde(with = "city_pair_map")]
    pub routes: HashMap<CityPairKey, (f64, f64)>,
}

impl GeoCache {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> IoResult<Self> {
        if path.as_ref().exists() {
            let data = fs::read_to_string(path)?;
            let cache = serde_json::from_str(&data)?;
            Ok(cache)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> IoResult<()> {
        let data = serde_json::to_string_pretty(self)?;
        fs::write(path, data)
    }
}
