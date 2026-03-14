use csv::ReaderBuilder;
use std::{collections::HashMap, error::Error};

#[derive(Debug, Clone)]
pub struct DepartmentLookup {
    departments: HashMap<String, String>,
}

impl DepartmentLookup {
    /// Creates a new lookup table from the embedded CSV file.
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let csv_data = include_str!("../departments.csv");
        let mut rdr = ReaderBuilder::new().delimiter(b',').from_reader(csv_data.as_bytes());

        let mut departments = HashMap::new();
        for result in rdr.records() {
            let record = result?;
            // Using .get(index) is safer than unwrapping
            let number = record
                .get(0)
                .ok_or("Missing department number in CSV")?
                .trim()
                .to_string();
            let name = record
                .get(1)
                .ok_or("Missing department name in CSV")?
                .trim()
                .to_string();
            departments.insert(number, name);
        }

        Ok(DepartmentLookup { departments })
    }

    /// Gets the full name of a department from its number (e.g., "35" -> "Ille-et-Vilaine").
    pub fn get_name(&self, number: &str) -> Option<&String> {
        self.departments.get(number)
    }

    /// Checks if a department number is valid.
    pub fn is_valid_department(&self, number: &str) -> bool {
        self.departments.contains_key(number)
    }

    /// Builds a full location string suitable for geocoding (e.g., "City, Department, France").
    pub fn build_geocode_query(&self, city: &str, dept_code: &str) -> Option<String> {
        self.get_name(dept_code)
            .map(|dept_name| format!("{}, {}", city, dept_name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_department_lookup() {
        let lookup = DepartmentLookup::new().unwrap();
        assert!(lookup.is_valid_department("35"));
        assert_eq!(lookup.get_name("35").unwrap(), "Ille-et-Vilaine");
        
        // Zero-padding usually matches csv format if present
        assert!(lookup.is_valid_department("01"));
        assert_eq!(lookup.get_name("01").unwrap(), "Ain");
    }

    #[test]
    fn test_invalid_department() {
        let lookup = DepartmentLookup::new().unwrap();
        assert!(!lookup.is_valid_department("999"));
        assert!(lookup.get_name("999").is_none());
    }

    #[test]
    fn test_build_geocode_query() {
        let lookup = DepartmentLookup::new().unwrap();
        let query = lookup.build_geocode_query("Rennes", "35").unwrap();
        assert_eq!(query, "Rennes, Ille-et-Vilaine");
    }
}
