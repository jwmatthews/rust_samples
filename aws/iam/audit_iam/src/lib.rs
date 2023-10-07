
use aws_sdk_iam::{self, Client};
use aws_sdk_iam::types::{AccessKeyMetadata, User};
use aws_sdk_iam::error::SdkError;
use aws_sdk_iam::operation::list_access_keys::ListAccessKeysError;
use aws_sdk_iam::operation::list_users::{ListUsersError};
use aws_sdk_iam::operation::get_access_key_last_used::GetAccessKeyLastUsedOutput;
use aws_sdk_iam::operation::get_access_key_last_used::GetAccessKeyLastUsedError;
use aws_sdk_iam::primitives::DateTime;
use aws_types::sdk_config::SdkConfig;
use tokio_stream::StreamExt;
use tracing::{error, info};

/*
Main idea:  Find user accounts that are left over from automated provisioning steps, they are
likely from provisioning OpenShift clusters and subsequently were not deleted from openshift-install.
We have some automation that will delete forgotten clusters, but we lack the terraform state when the clusters
were created, so we need to recreate the various cleanup steps ourself.

We want to write the data to be delete to a yaml file, so we can customize it and verify before running.


Workflow for getting a list of users
1. Fetch a list of users from AWS IAM and filter for specific patterns.
2. Find the associated access keys
3. Find the most recent access key usage per user
4. Find the users who have not used their access key for a given period of time
5. Write the information to a yaml file

Workflow for deleting users
1. Read YAML file to get a list of users to delete
2. Remove attached policies
3. Remove inline policies
4. Remove all access keys
5. Delete User

TODO:
- [ ] Write output to yaml file
- [ ] Get last associated key access time
- [ ] Add a command line option to specify the number of days since last access key usage
- [ ] Add a command line option to specify the pattern for user names
- [ ] Add a command line option to specify the output file name
- [ ] Add a command line option to specify the input file name

 */

pub struct KeyEntry {
    create_date: DateTime,
    last_access_date: DateTime,
    key_id: String,
}

pub struct UserEntry {
    pub user: User,
    pub last_access_date: DateTime,
    pub keys: Vec<AccessKeyMetadata>,
}

pub fn get_client(config: SdkConfig) -> Client {
    Client::new(&config)
}

pub async fn run() -> Result<Vec<UserEntry>, SdkError<ListUsersError>> {
    let config = aws_config::load_from_env().await;
    run_with_config(config).await
}

pub async fn run_with_config(config: SdkConfig) -> Result<Vec<UserEntry>, SdkError<ListUsersError>> {
    let mut entries = Vec::new();
    let client = aws_sdk_iam::Client::new(&config);

    match list_users(client.clone()).await {
        Ok(users) => {
            info!("{} users found", users.len());
            for user in users.iter().take(5) {
                /*let user_name = match &user.user_name {
                    Some(name) => { name }
                    None => {"noname"}
                };*/
                let user_name = user.clone().user_name.unwrap_or("noname".to_string());
                let access_keys = match list_access_keys(client.clone(), user_name.as_str()).await {
                    Ok(keys) => { keys }
                    Err(e) => {
                        error!("Error listing access keys for user {}: {}", user_name, e);
                        Vec::new()
                    }
                };
                let last_key_entry_used = match determine_last_access_date(client.clone(), access_keys.clone()).await {
                    Ok(entry) => { entry }
                    Err(e) => {
                        error!("Error getting last access date for user {}: {}", user_name, e);
                        KeyEntry { create_date: DateTime::from_secs(0), last_access_date: DateTime::from_secs(0), key_id: "".to_string() }
                    }
                };
                entries.push(UserEntry { user: user.clone(), keys: access_keys, last_access_date: last_key_entry_used.last_access_date } );

            }
            Ok(entries)
        }
        Err(e) => {
            eprintln!("Error: {:?}", e);
            Err(e)
        }
    }
}

pub async fn list_users(client: Client) -> Result<Vec<User>, SdkError<ListUsersError>> {
    let paginator = client
        .list_users()
        .max_items(10)
        .into_paginator()
        .items()
        .send();

    paginator.collect::<Result<Vec<User>, _>>().await
}

pub async fn list_access_keys(client: Client, username: &str) -> Result<Vec<AccessKeyMetadata>, SdkError<ListAccessKeysError>> {
    let paginator = client
        .list_access_keys()
        .user_name(username)
        .into_paginator()
        .items()
        .send();

    paginator.collect::<Result<Vec<AccessKeyMetadata>, _>>().await
}

pub async fn get_access_key_last_used(client: Client, key_id: &str) -> Result<GetAccessKeyLastUsedOutput, SdkError<GetAccessKeyLastUsedError>> {
    client.get_access_key_last_used()
        .access_key_id(key_id)
        .send()
        .await
}

pub async fn determine_last_access_date(client: Client, access_keys: Vec<AccessKeyMetadata>) -> Result<KeyEntry, SdkError<GetAccessKeyLastUsedError>> {
    let mut entry = KeyEntry { create_date: DateTime::from_secs(0), last_access_date: DateTime::from_secs(0), key_id: "".to_string() };
    let mut last_date = DateTime::from_secs(0);
    for key in access_keys {
        let k = get_access_key_last_used(client.clone(), key.clone().access_key_id.unwrap_or_default().as_str()).await;
        entry.key_id = key.clone().access_key_id.unwrap_or_default().to_string();
        entry.last_access_date = k.ok().unwrap().access_key_last_used.unwrap().last_used_date.unwrap();
    }
    Ok(entry)
}
