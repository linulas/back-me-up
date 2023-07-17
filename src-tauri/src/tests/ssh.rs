use std::path::PathBuf;

use crate::models::app::Config;
use crate::ssh::connect;
use dotenv::dotenv;
use log::error;

#[actix_rt::test]
async fn test_connection() {
    dotenv().ok();
    let control_directory =
        std::env::var("SSH_CONTROL_DIRECTORY").expect("SSH_CONTROL_DIRECTORY must be set");
    let config = Config {
        client_name: String::from("Test"),
        username: std::env::var("SSH_USER").expect("SSH_USER must be set"),
        server_address: std::env::var("SSH_HOST").expect("SSH_HOST must be set"),
        server_port: std::env::var("SSH_PORT")
            .expect("SSH_PORT must be set")
            .parse()
            .expect("SSH_PORT must be a number"),
        allow_background_backup: true,
    };
    let connection = connect::to_server(config, PathBuf::from(control_directory)).await;
    if let Err(e) = &connection {
        error!("{e:?}");
    }
    assert!(connection.is_ok());

    connection
        .expect("should have a connection to close")
        .close()
        .await
        .expect("Failed to close");
}

#[actix_rt::test]
async fn test_sftp_client() {
    dotenv().ok();
    let control_directory =
        std::env::var("SSH_CONTROL_DIRECTORY").expect("SSH_CONTROL_DIRECTORY must be set");
    let config = Config {
        client_name: String::from("Test"),
        username: std::env::var("SSH_USER").expect("SSH_USER must be set"),
        server_address: std::env::var("SSH_HOST").expect("SSH_HOST must be set"),
        server_port: std::env::var("SSH_PORT")
            .expect("SSH_PORT must be set")
            .parse()
            .expect("SSH_PORT must be a number"),
        allow_background_backup: true,
    };
    let client = connect::Connection::new(config, PathBuf::from(control_directory))
        .await
        .expect("Failed to connect")
        .sftp_client;
    let path = std::path::Path::new("./test.txt");

    assert!(client.create(path).await.is_ok());
    assert!(client.fs().remove_file(path).await.is_ok());

    client.close().await.expect("Failed to close");
}
