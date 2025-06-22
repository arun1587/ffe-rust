use csv::ReaderBuilder;
use std::{collections::HashMap, error::Error, fs::File, path::Path};

#[derive(Debug, Clone)]
pub struct DepartmentLookup {
    departments: HashMap<String, String>,
}

impl DepartmentLookup {
    /// Creates a new lookup table from a 2-column CSV file (number, name).
    pub fn new<P: AsRef<Path>>(csv_path: P) -> Result<Self, Box<dyn Error>> {
        let file = File::open(csv_path)?;
        let mut rdr = ReaderBuilder::new().delimiter(b',').from_reader(file);

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
