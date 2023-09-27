use super::Error;
use crate::{set_state_and_test_connection, storage};
use back_me_up::commands::os::get_hostname;
use back_me_up::models::app::{Config, MutexState};
use inquire::validator::Validation;
use inquire::{CustomType, Text};
use std::net::IpAddr;
use std::str::FromStr;

fn setup_config() -> Result<Config, Error> {
    let client_name =
        get_hostname().map_or_else(|_| String::from("unknown_client"), |hostname| hostname); // TODO: append UID to string
    let validator = |input: &str| {
        if input.chars().count() > 50 {
            Ok(Validation::Invalid(
                "You're only allowed 50 characters.".into(),
            ))
        } else {
            Ok(Validation::Valid)
        }
    };

    let username = Text::new("Backup server username:")
        .with_validator(validator)
        .prompt()?;

    let server_addr_validator = |input: &str| match IpAddr::from_str(input) {
        Ok(_) => Ok(Validation::Valid),
        Err(_) => Ok(Validation::Invalid(
            "Please enter a valid IP address.".into(),
        )),
    };

    let server_address = Text::new("Backup server ip-address:")
        .with_validator(server_addr_validator)
        .prompt()?;

    let server_port = CustomType::<u16>::new("Backup server port:")
        .with_error_message("Please type a valid port number")
        .prompt()?;

    Ok(Config {
        username,
        client_name,
        server_address,
        server_port,
        allow_background_backup: false,
    })
}

pub async fn begin(state: &MutexState) -> Result<Config, Error> {
    loop {
        if let Ok(config) = set_state_and_test_connection(state, setup_config()?).await {
            storage::Storage::load()?.write_conig(&config);
            return Ok(config);
        }

        println!("⛔️ Could not connect with the provided credentials, try again and make sure your credentials are correct.\n");
    }
}
