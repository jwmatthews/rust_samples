
use aws_sdk_iam;
use aws_sdk_iam::error::SdkError;
use aws_sdk_iam::operation::list_users::ListUsersError;


//#[tokio::run]
pub async fn run() -> Result<(), SdkError<ListUsersError>> {
    // Create a client
    let shared_config = aws_config::load_from_env().await;
    let client = aws_sdk_iam::Client::new(&shared_config);

    // Call the list_users method
    match client.list_users().send().await {
        Ok(resp) => {
            // Iterate through the users and print their names
            for user in resp.users.unwrap_or_default() {
                println!("{}", user.user_name.unwrap_or_default());
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("Error: {:?}", e);
            Err(e)
        }
    }
}