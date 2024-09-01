use std::fs::File;
use std::io::Read;
use serde::Deserialize;
use std::collections::HashMap;
use serde_json; 


#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct AnalysisReport {
    pub rulesets: Vec<Ruleset>,
}

impl AnalysisReport {
    /*pub fn get_impacted_file_names(&self) -> Vec<String> {
        self.rulesets.iter()
            .map(|ruleset| ruleset.violations.keys().cloned().collect())
        self.violations.iter()
            .map(|(file, _)| file.to_string())
            .collect()
    }*/
    pub fn load_from_file(&mut self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::open(file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
    
        let rulesets: Vec<Ruleset>= serde_yaml::from_str(&contents)?;
        self.rulesets = rulesets;
        Ok(())
    }
}


#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Ruleset {
    pub name: String,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub violations: HashMap<String, Violation>,
    
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub insights: HashMap<String, Insight>,
    
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub errors: HashMap<String, String>,
    
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unmatched: Vec<String>,
}



#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Violation {
    pub description: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub labels: Vec<String>,
    
    pub incidents: Vec<Incident>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<i32>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Insight {
    pub description: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub labels: Vec<String>,
    
    pub incidents: Vec<Incident>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Incident {
    pub uri: String,
    pub message: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_snip: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_number: Option<i32>,
    
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub variables: HashMap<String, serde_json::Value>,
}

pub fn parse_yaml(file_path: &str) -> Result<AnalysisReport, Box<dyn std::error::Error>> {
    let mut report = AnalysisReport::default();
    report.load_from_file(file_path)?;
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coolstore_analysis() {
        let mut report = AnalysisReport::default();
        let result = report.load_from_file("samples/coolstore_analysis_output.yaml");
        assert!(matches!(result, Ok(_))); 
        assert_eq!(report.rulesets.len(), 26, "The vector length did not match the expected value.");
        println!("Parsed report: {:?}", report);
    }

    #[test]
    fn test_demo_output_analysis() {
        let mut report = AnalysisReport::default();
        let result = report.load_from_file("samples/demo-output.yaml");
        assert!(matches!(result, Ok(_)));
        assert_eq!(report.rulesets.len(), 1, "The vector length did not match the expected value.");
        println!("Parsed report: {:?}", report);
    }

    /* 
    #[test]
    fn impacted_file_names() {
        let report = parse_yaml("samples/demo-output.yaml").unwrap();
        let impacted_files = report.impacted_file_names();
        assert_eq!(impacted_files, ["src/main.rs"]);
    }
    */
 
}