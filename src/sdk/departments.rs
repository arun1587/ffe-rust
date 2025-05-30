use std::{collections::HashMap, error::Error, fs::File, path::Path};
use csv::ReaderBuilder;

#[derive(Debug, Clone)]
pub struct DepartmentLookup {
    departments: HashMap<String, String>,
}

impl DepartmentLookup {
    pub fn from_csv<P: AsRef<Path>>(csv_path: P) -> Result<Self, Box<dyn Error>> {
        let file = File::open(csv_path)?;
        let mut rdr = ReaderBuilder::new().delimiter(b',').from_reader(file);

        let mut departments = HashMap::new();
        for result in rdr.records() {
            let record = result?;
            let number = record.get(0).ok_or("Missing number")?.to_string();
            let name = record.get(1).ok_or("Missing name")?.to_string();
            departments.insert(number, name);
        }

        Ok(DepartmentLookup { departments })
    }

    pub fn get_name(&self, number: &str) -> Option<&String> {
        self.departments.get(number)
    }

    pub fn is_valid_department(&self, number: &str) -> bool {
        self.departments.contains_key(number)
    }

    pub fn origin_from(&self, city: &str, dept_code: &str) -> Option<String> {
        self.get_name(dept_code)
            .map(|dept_name| format!("{},{},France", city, dept_name))
    }
}
