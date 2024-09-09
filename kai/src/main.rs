mod yaml_parser;
use std::collections::HashMap;
use std::time::Instant;
use yaml_parser::parse_yaml;
use yaml_parser::Ruleset;

#[allow(dead_code)]
fn print_debug_demo_report() {
    let report = parse_yaml("samples/demo-output.yaml").unwrap();
   
    let start = Instant::now();
    let impacted_files = report.impacted_files();
    let duration = start.elapsed();
    println!("Impacted files: {:?}", impacted_files.keys()); 
    println!("# of impacted files: {:?}", impacted_files.len()); 
    println!("Test 'impacted_files_c' took: {:?}", duration);
    
    let expected_key = "file:///examples/customers-tomcat-legacy/pom.xml";
    let rulesets = impacted_files.get(expected_key).unwrap();
    println!("URI: `{}` Impacted rulesets: {:?}", expected_key, rulesets.len());

    let ruleset = rulesets.get("konveyor-analysis").unwrap();
    println!("URI: `{}` Ruleset: `{}` # Violations: {:?}", expected_key, ruleset.name, ruleset.violations.len());
    for (violation_name, violation) in ruleset.violations.iter() {
        println!("Violation: name {:?}", violation_name);
        println!("\tViolation: description {:?}", violation.description);
        println!("\tViolation: # incidents {:?}", violation.incidents.len());
    }   
}

fn main() {

   //print_debug_demo_report();

   match parse_yaml("samples/coolstore_analysis_output.yaml") {
    Ok(report) => {
        let impacted_files: HashMap<String, HashMap<String, Ruleset>> = report.impacted_files()
            .into_iter()
            .filter(|(uri, _impacted_ruleset)| {
                !uri.starts_with("file:///root/.m2")
            })
            .collect();
        println!("Parsed report has {:?} impacted files", impacted_files.len())
    },
    Err(e) => eprintln!("Error parsing YAML: {}", e),
}


}
