mod yaml_parser;

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
}
