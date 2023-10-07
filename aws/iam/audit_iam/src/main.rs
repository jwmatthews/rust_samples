#[::tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

   // Read for starter on using tracing:  https://tokio.rs/tokio/topics/tracing
    // Start configuring a `fmt` subscriber
    let subscriber = tracing_subscriber::fmt()
        // Use a more compact, abbreviated log format
        .compact()
        // Display source code file paths
        .with_file(true)
        // Display source code line numbers
        .with_line_number(true)
        // Display the thread ID an event was recorded on
        .with_thread_ids(true)
        // Don't display the event's target (module path)
        .with_target(false)
        // Build the subscriber
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    let entries = audit_iam::run().await;
    match entries {
        Ok(entries) => {
            for entry in entries {
                println!("User: {}", entry.user.user_name.unwrap_or_default());
                for key in entry.keys {
                    println!("  Key: id {}, status {}, create_date {:?}",
                             key.access_key_id.unwrap_or_default(),
                             key.status.expect("No status").as_str(),
                             key.create_date.expect("No create date")
                    )
                }
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1)
        }
    }
}
