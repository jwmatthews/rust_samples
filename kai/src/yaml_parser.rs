use std::fs::File;
use std::io::Read;
use serde::Deserialize;
use std::collections::HashMap;
use std::collections::HashSet;
use serde_json; 

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct AnalysisReport {
    pub rulesets: Vec<Ruleset>,
}

impl AnalysisReport {

    /// This method returns the impacted file names from the analysis report.
    ///
    /// # Returns
    /// * A `HashMap<String, Vec<Ruleset>>` where the key is the uri and the value is a Vector of Rulsets. 
    ///
    pub fn impacted_file_names(&self) -> Vec<String> {
        let mut uris = HashSet::new();
        for ruleset in &self.rulesets {
            for (_violation_name, violation) in &ruleset.violations {
                for incident in &violation.incidents {
                    uris.insert(incident.uri.clone());
                }
            }
        }
        let vec: Vec<String> = uris.into_iter().collect();
        vec
    }

    /// This method returns the impacted files with their associated violations.
    ///
    /// # Returns
    /// * A `HashMap<String, Vec<Ruleset>>` where the key is the uri and the value is a Vector of Rulsets. 
    ///
    pub fn impacted_files(&self) -> HashMap<String, Vec<Ruleset>> {
        let mut impacted_files = HashMap::<String, Vec<Ruleset>>::new();
      
        // First we get the list of impacted file names
        let uris = self.impacted_file_names();

        for uri in uris {
            // We iterate through all the rulesets
            for ruleset in &self.rulesets {
                // We clone the ruleset and remove the violation data 
                //so we can readd just the data related to this uri
                let mut stripped_ruleset = ruleset.clone();
                stripped_ruleset.violations = HashMap::new();
                // We iterate over all the violations in this ruleset
                for (violation_key, violation) in &ruleset.violations {
                    // We clone the violation and remove the incidents 
                    let mut stripped_violation = violation.clone();
                    stripped_violation.incidents = Vec::new();
                    // We iterate over all the incidents in this violation
                    for incident in &violation.incidents {
                        if uri == incident.uri {
                            // We add the incident to the stripped violation
                            stripped_violation.incidents.push(incident.clone());
                            // We add the stripped violation to the stripped ruleset
                        }
                    }
                    if stripped_violation.incidents.len() > 0 {
                        stripped_ruleset.violations.insert(violation_key.clone(), stripped_violation.clone());
                    }
                }
                if stripped_ruleset.violations.len() > 0 {
                    impacted_files.entry(uri.clone()).or_insert(Vec::new()).push(stripped_ruleset.clone());
                } 
            }
        }
        impacted_files
    }

    // Exploring an alternative implementation of impacted files, trying to avoid the 
    //outer loop of uris like in original implementation
    pub fn impacted_files_b(&self) -> HashMap<String, Vec<Ruleset>> {
        let uris = self.impacted_file_names();
        let uri_set: HashSet<String> = uris.into_iter().collect(); // Convert to HashSet for faster lookups
    
        let mut impacted_files = HashMap::new();
    
        for ruleset in &self.rulesets {
            let mut stripped_ruleset = None;
    
            for (violation_key, violation) in &ruleset.violations {
                let mut stripped_violation = None;
    
                for incident in &violation.incidents {
                    if uri_set.contains(&incident.uri) {
                        let stripped_violation = stripped_violation.get_or_insert_with(|| {
                            let mut v = violation.clone();
                            v.incidents = Vec::with_capacity(violation.incidents.len());
                            v
                        });
                        stripped_violation.incidents.push(incident.clone());
                    }
                }
    
                if let Some(stripped_violation) = stripped_violation {
                    let stripped_ruleset = stripped_ruleset.get_or_insert_with(|| {
                        let mut r = ruleset.clone();
                        r.violations = HashMap::new();
                        r
                    });
                    stripped_ruleset.violations.insert(violation_key.clone(), stripped_violation);
                }
            }
    
            if let Some(stripped_ruleset) = stripped_ruleset {
                for incident in stripped_ruleset.violations.values().flat_map(|v| &v.incidents) {
                    impacted_files.entry(incident.uri.clone())
                        .or_insert_with(Vec::new)
                        .push(stripped_ruleset.clone());
                }
            }
        }
    
        impacted_files
    }

    pub fn load_from_file(&mut self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::open(file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
    
        let rulesets: Vec<Ruleset>= serde_yaml::from_str(&contents)?;
        self.rulesets = rulesets;
        Ok(())
    }
}


#[derive(Clone, Debug, Default, Deserialize)]
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


#[derive(Clone, Debug, Default, Deserialize)]
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

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct Insight {
    pub description: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub labels: Vec<String>,
    
    pub incidents: Vec<Incident>,
}

#[derive(Clone, Debug, Default, Deserialize)]
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

    #[test]
    fn impacted_file_names() {
        let report = parse_yaml("samples/demo-output.yaml").unwrap();
        let mut impacted_files = report.impacted_file_names();

        let mut expected_impacted_files = 
            vec![
                "file:///examples/java/dummy/pom.xml", 
                "file:///examples/golang/dummy/test_functions.go", 
                "file:///examples/customers-tomcat-legacy/pom.xml", 
                "file:///examples/java/example/pom.xml", 
                "file:///examples/java-project/pom.xml", 
                "file:///examples/golang/go.mod", 
                "file:///examples/python/file_a.py", 
                "file:///examples/golang/main.go", 
                "file:///examples/java/example/src/main/java/com/example/apps/App.java", 
                "file:///examples/builtin/inclusion_tests/dir-0/inclusion-test.xml", 
                "file:///examples/gradle-multi-project-example/template-server/src/main/java/io/jeffchao/template/server/Server.java", 
                "file:///examples/yaml/k8s.yaml", 
                "file:///examples/inclusion-tests/src/main/java/io/konveyor/util/FileReader.java", 
                "file:///examples/java/beans.xml", 
                "file:///examples/gradle-multi-project-example/build.gradle", 
                "file:///examples/java/jboss-app.xml", 
                "file:///examples/java/example/src/main/java/com/example/apps/Bean.java", 
                "file:///examples/customers-tomcat-legacy/Dockerfile", 
                "file:///examples/java/pom.xml", 
                "file:///examples/builtin/inclusion_tests/dir-0/inclusion-test.json"];
        assert_eq!(impacted_files.sort(), expected_impacted_files.sort());
    }

    #[test]
    fn impacted_files() {
        let report = parse_yaml("samples/demo-output.yaml").unwrap();
        let impacted_files = report.impacted_files();
        assert_eq!(impacted_files.len(), 20);

        let report = parse_yaml("samples/coolstore_analysis_output.yaml").unwrap();
        let impacted_files = report.impacted_files();
        assert_eq!(impacted_files.len(), 424);
    }

      #[test]
    fn impacted_files_b() {
        let report = parse_yaml("samples/demo-output.yaml").unwrap();
        let impacted_files = report.impacted_files_b();
        assert_eq!(impacted_files.len(), 20);

        let report = parse_yaml("samples/coolstore_analysis_output.yaml").unwrap();
        let impacted_files = report.impacted_files_b();
        assert_eq!(impacted_files.len(), 424);
    }


}