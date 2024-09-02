mod yaml_parser;

use std::time::Instant;
use yaml_parser::parse_yaml;


fn main() {
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
}
