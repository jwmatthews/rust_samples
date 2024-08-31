use std::fs::File;
use std::io::Read;
use serde::Deserialize;
use std::collections::HashMap;
use serde_json; 

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct AnalysisReport {
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

pub fn parse_yaml(file_path: &str) -> Result<Vec<AnalysisReport>, Box<dyn std::error::Error>> {
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let report: Vec<AnalysisReport>= serde_yaml::from_str(&contents)?;
    Ok(report)
}