
use aws_sdk_iam::{self, Client};
use aws_sdk_iam::error::SdkError;
use aws_sdk_iam::operation::list_users::{ListUsersError, ListUsersOutput};
//use aws_sdk_iam::operation::list_users::_list_users_output::ListUsersOutput;
use aws_types::sdk_config::SdkConfig;

pub async fn run() -> Result<(), SdkError<ListUsersError>> {
    // Create a client
    let config = aws_config::load_from_env().await;
    run_with_config(config).await
}

pub async fn run_with_config(config: SdkConfig) -> Result<(), SdkError<ListUsersError>> {
    let client = aws_sdk_iam::Client::new(&config);

    match list_users(client).await {
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



pub async fn list_users(client: Client) -> Result<ListUsersOutput, SdkError<ListUsersError>> {
    client.list_users().send().await

    // Call the list_users method
    /*match client.list_users().send().await {
        Ok(resp) => {
            // Iterate through the users and print their names
            for user in resp.users.clone().unwrap_or_default() {
                println!("{}", user.user_name.unwrap_or_default());
            }
            Ok(resp)
        }
        Err(e) => {
            eprintln!("Error: {:?}", e);
            Err(e)
        }
    }
     */
}