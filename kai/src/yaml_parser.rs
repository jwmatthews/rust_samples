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

   
    // Exploring an alternative implementation of impacted files, trying to avoid the 
    // outer loop of uris like in original implementation
    pub fn impacted_files_ugly(&self) -> HashMap<String, HashMap<String, Ruleset>> {
        // key: uri:
        //  key: ruleset_name
        //     violations:
        //       key: violation_name: 
        //           incidents:
        //             - incident data //stripped to just that uri
        let mut impacted_files = HashMap::<String, HashMap<String, Ruleset>>::new();
        for ruleset in &self.rulesets {
            for (violation_key, violation) in &ruleset.violations {
                for incident in &violation.incidents {

                    if impacted_files.contains_key(&incident.uri) {
                        let impacted_rulesets = impacted_files.get_mut(&incident.uri).unwrap();
                        if impacted_rulesets.contains_key(&ruleset.name) {
                            let impacted_ruleset = impacted_rulesets.get_mut(&ruleset.name).unwrap();
                            if impacted_ruleset.violations.contains_key(violation_key) {
                                let violations = impacted_ruleset.violations.get_mut(violation_key).unwrap();
                                violations.incidents.push(incident.clone());
                            }
                            else {
                                let mut stripped_violation = violation.clone();
                                stripped_violation.incidents = Vec::new();
                                stripped_violation.incidents.push(incident.clone());
                                impacted_ruleset.violations.insert(violation_key.clone(), stripped_violation);
                            }
                        } 
                        else {
                            let mut stripped_violation = violation.clone();
                            stripped_violation.incidents = Vec::new();
                            stripped_violation.incidents.push(incident.clone());

                            let mut stripped_ruleset = ruleset.clone();
                            stripped_ruleset.violations = HashMap::new();
                            stripped_ruleset.tags = Vec::new();
                            stripped_ruleset.insights = HashMap::new();
                            stripped_ruleset.errors = HashMap::new();
                            stripped_ruleset.unmatched = Vec::new();
                            stripped_ruleset.violations.insert(violation_key.clone(), stripped_violation);

                            impacted_rulesets.insert(ruleset.name.clone(), stripped_ruleset);
                        }
                    } else {

                        let mut stripped_violation = violation.clone();
                        stripped_violation.incidents = Vec::new();
                        stripped_violation.incidents.push(incident.clone());

                        let mut stripped_ruleset = ruleset.clone();
                        stripped_ruleset.violations = HashMap::new();
                        stripped_ruleset.tags = Vec::new();
                        stripped_ruleset.insights = HashMap::new();
                        stripped_ruleset.errors = HashMap::new();
                        stripped_ruleset.unmatched = Vec::new();
                        stripped_ruleset.violations.insert(violation_key.clone(), stripped_violation);
                        
                        let mut uri_rulesets = HashMap::<String, Ruleset>::new();
                        uri_rulesets.insert(ruleset.name.clone(), stripped_ruleset);
                        impacted_files.insert(incident.uri.clone(), uri_rulesets);
                    } 

                }
            }
        }
        impacted_files
    }
           
  
    pub fn impacted_files(&self) -> HashMap<String, HashMap<String, Ruleset>> {
        // key: uri:
        //  key: ruleset_name
        //     violations:
        //       key: violation_name: 
        //           incidents:
        //             - incident data //stripped to just that uri
        let mut impacted_files = HashMap::<String, HashMap<String, Ruleset>>::new();
        
        for ruleset in &self.rulesets {
            for (violation_key, violation) in &ruleset.violations {
                for incident in &violation.incidents {

                    impacted_files
                        .entry(incident.uri.clone()) // Entry for the URI
                        .or_insert_with(HashMap::new) // Insert a new ruleset HashMap if missing
                        .entry(ruleset.name.clone()) // Entry for the ruleset name
                        .or_insert_with(|| {
                            let mut stripped_ruleset = ruleset.clone();
                            stripped_ruleset.violations = HashMap::new();
                            stripped_ruleset.tags.clear();
                            stripped_ruleset.insights.clear();
                            stripped_ruleset.errors.clear();
                            stripped_ruleset.unmatched.clear();
                            stripped_ruleset
                        })
                        .violations
                        .entry(violation_key.clone()) // Entry for the violation
                        .or_insert_with(|| {
                            let mut stripped_violation = violation.clone();
                            stripped_violation.incidents = Vec::new();
                            stripped_violation
                        })
                        .incidents
                        .push(incident.clone()); // Add the incident to the stripped violation
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

// Key: uri, Value: HashMap<String, Ruleset>
//      Key: ruleset name, Value: Vec<Ruleset>  
//type ImpactedRuleset = HashMap<String, HashMap<String, Ruleset>>;

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
        let report = parse_yaml("samples/coolstore_analysis_output.yaml").unwrap();
        let impacted_files = report.impacted_files();
        assert_eq!(impacted_files.len(), 424);
        
        let report = parse_yaml("samples/demo-output.yaml").unwrap();
        let impacted_files = report.impacted_files();
        assert_eq!(impacted_files.len(), 20);
        
        let expected_key = "file:///examples/customers-tomcat-legacy/pom.xml";
        assert!(impacted_files.contains_key(expected_key), "The key '{}' should exist in the impacted_files", &expected_key);

        let impacted_rulesets = impacted_files.get(expected_key).unwrap();
        assert_eq!(impacted_rulesets.len(), 1);

        let expected_ruleset_name = "konveyor-analysis";
        assert!(impacted_rulesets.contains_key(expected_ruleset_name), "The key '{}' should exist in the impacted_rulesets", &expected_ruleset_name);
         
        let ruleset = impacted_rulesets.get("konveyor-analysis").unwrap();
        assert_eq!(ruleset.violations.len(), 2);

        let violation_name = "xml-pom-001";
        let violation = ruleset.violations.get(violation_name).unwrap();
        assert_eq!(violation.incidents.len(), 17);

        let violation_name = "chain-pom-001";
        let violation = ruleset.violations.get(violation_name).unwrap();
        assert_eq!(violation.incidents.len(), 17);


        /* 
        let uri = "file:///examples/builtin/inclusion_tests/dir-0/inclusion-test.json";
        let impacted_rulesets = impacted_files.get(uri).unwrap();
        assert_eq!(impacted_rulesets.len(), 1);
        let expected_violations = impacted_rulesets.get("konveyor-analysis").unwrap();
        assert_eq!(expected_violations.len(), 1);   
        let expected_violation = expected_violations.get("builtin-inclusion-test-json").unwrap();
        assert_eq!(expected_violation.len(), 3);
        */

    }



}