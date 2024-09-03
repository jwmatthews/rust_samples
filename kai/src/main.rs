mod yaml_parser;

use std::time::Instant;
use yaml_parser::parse_yaml;


fn main() {
    /* 
    match parse_yaml("samples/coolstore_analysis_output.yaml") {
        Ok(report) => println!("Parsed report: {:?}", report),
        Err(e) => eprintln!("Error parsing YAML: {}", e),
    }

    match parse_yaml("samples/demo-output.yaml") {
        Ok(report) => println!("Parsed report: {:?}", report),
        Err(e) => eprintln!("Error parsing YAML: {}", e),
    } 
    
    let report = parse_yaml("samples/coolstore_analysis_output.yaml").unwrap();
    
    let mut start = Instant::now();
    let impacted_files = report.impacted_files();
    let mut duration = start.elapsed();
    println!("# of impacted files: {:?}", impacted_files.len()); 
    println!("Test 'impacted_files' took: {:?}", duration);
    //println!("Impacted files: {:?}", impacted_files.keys());

    start = Instant::now();
    let impacted_files_b = report.impacted_files_b();
    duration = start.elapsed();
    println!("# of impacted files: {:?}", impacted_files_b.len()); 
    println!("Test 'impacted_files_b' took: {:?}", duration);
    */
   
    let report = parse_yaml("samples/demo-output.yaml").unwrap();
   

    let start = Instant::now();
    let impacted_files_c = report.impacted_files_c();
    let duration = start.elapsed();
    println!("Impacted files: {:?}", impacted_files_c.keys()); 
    println!("# of impacted files: {:?}", impacted_files_c.len()); 
    println!("Test 'impacted_files_c' took: {:?}", duration);
    
    let expected_key = "file:///examples/customers-tomcat-legacy/pom.xml";
    let rulesets = impacted_files_c.get(expected_key).unwrap();
    println!("URI: `{}` Impacted rulesets: {:?}", expected_key, rulesets.len());

    let ruleset = rulesets.get("konveyor-analysis").unwrap();
    println!("URI: `{}` Ruleset: `{}` # Violations: {:?}", expected_key, ruleset.name, ruleset.violations.len());
    for (violation_name, violation) in ruleset.violations.iter() {
        println!("Violation: name {:?}", violation_name);
        println!("\tViolation: description {:?}", violation.description);
        println!("\tViolation: # incidents {:?}", violation.incidents.len());
    }   
   // println!("URI: `{}` Impacted rulesets: {:?}", expected_key, impacted_rulesets.len());


}
