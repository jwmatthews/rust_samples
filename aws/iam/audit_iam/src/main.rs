fn main() {
    if let Err(e) = audit_iam::run() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
