
#[::tokio::main]
async fn main() {
    if let Err(e) = audit_iam::run().await {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
