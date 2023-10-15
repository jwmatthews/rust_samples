#![warn(clippy::pedantic)]

use aws_sdk_iam::{self, Client};
use aws_sdk_iam::types::{AccessKeyLastUsed, AccessKeyMetadata, User};
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


pub struct UserEntry {
    pub user: User,
    pub last_access_date: Option<DateTime>,
    pub keys: Vec<AccessKeyMetadata>,
}

#[must_use]
pub fn get_client(config: &SdkConfig) -> Client {
    Client::new(config)
}

/// Will create a default `SdkConfig` from the environment and then call `run_with_config`
/// # Errors
///
/// Will return `Err` if a problem is encountered talking to AWS APIs
pub async fn run() -> Result<Vec<UserEntry>, SdkError<ListUsersError>> {
    let config = aws_config::load_from_env().await;
    run_with_config(config).await
}

/// Uses a specific `SdkConfig` to create a client
/// # Errors
///
/// Will return `Err` if a problem is encountered talking to AWS APIs
pub async fn run_with_config(config: SdkConfig) -> Result<Vec<UserEntry>, SdkError<ListUsersError>> {
    let mut entries = Vec::new();
    let client = aws_sdk_iam::Client::new(&config);
    let users = list_users(client.clone()).await?;
    info!("{} users found", users.len());
    for user in users {
        let user_name = user.user_name.clone().unwrap_or("noname".to_string());
        let access_keys = match list_access_keys(client.clone(), user_name.as_str()).await {
            Ok(keys) => { keys }
            Err(e) => {
                error!("Error listing access keys for user {}: {}", user_name, e);
                Vec::new()
            }
        };
        if let Some(lku) = determine_last_access_date(client.clone(), access_keys.clone()).await {
            entries.push(UserEntry { user: user.clone(), keys: access_keys, last_access_date: lku.last_used_date });
        } else {
            error!("We didn't find a last access date for user {}", user_name);
            entries.push(UserEntry { user: user.clone(), keys: access_keys, last_access_date: None });
        };

    }
    Ok(entries)
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

pub async fn determine_last_access_date(client: Client, access_keys: Vec<AccessKeyMetadata>) -> Option<AccessKeyLastUsed> {

    let mut saved_last_used : Option<AccessKeyLastUsed> = None;

    for key in &access_keys {
        let resp = get_access_key_last_used(client.clone(), key.clone().access_key_id.unwrap_or_default().as_str()).await.ok()?;
        info!("Response: {:?}", resp);
        let tmp_last_used = match resp.access_key_last_used {
            Some(aklu) => {
                info!("\tLast used: {:?}", aklu);
                match aklu.last_used_date {
                    Some(val_last_used_date) => {
                        info!("\tLast used date: {:?}", val_last_used_date);
                        Some(aklu.clone())
                    }
                    None => {
                        error!("\tUnable to find last used date for {:?}", resp.user_name);
                        None
                    }
                }
            }
            None => {
                error!("\tUnable to find last used date");
                None
            }
        };

        // Check the actual last used date value to ensure it is set, we often see keys that have not been used so they have no last used date
        // If it is set, then find the most recent key which has been used and save it so we can return it later
        if tmp_last_used.is_some() && tmp_last_used.as_ref().unwrap().last_used_date.is_some() {
            // We ensured that this key has a last_used_date, so we can safely unwrap the value
            if saved_last_used.is_none() {
                // First time through with a valid date, so set the value
                saved_last_used = tmp_last_used;
            } else {
                // Check if the current key has a more recent last_used_date than what we saved
                if tmp_last_used.as_ref().unwrap().last_used_date.unwrap() > saved_last_used.as_ref().unwrap().last_used_date.unwrap() {
                    saved_last_used = tmp_last_used;
                }
            }
        }
    };
    saved_last_used
}
