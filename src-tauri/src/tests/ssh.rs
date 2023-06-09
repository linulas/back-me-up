use crate::models::app::Config;
use crate::ssh::connect;
use dotenv::dotenv;

#[actix_rt::test]
async fn test_connection() {
    dotenv().ok();
    let config = Config {
        username: std::env::var("SSH_USER").expect("SSH_USER must be set"),
        server_address: std::env::var("SSH_HOST").expect("SSH_HOST must be set"),
        server_port: std::env::var("SSH_PORT").expect("SSH_PORT must be set").parse().expect("SSH_PORT must be a number"),
    };
    let connection = connect::to_server(config).await;
    if let Err(e) = &connection {
        eprintln!("{e:?}");
    }
    assert!(connection.is_ok());

    connection.expect("should have a connection to close").close().await.expect("Failed to close");
}

#[actix_rt::test]
async fn test_sftp_client() {
    dotenv().ok();
    let config = Config {
        username: std::env::var("SSH_USER").expect("SSH_USER must be set"),
        server_address: std::env::var("SSH_HOST").expect("SSH_HOST must be set"),
        server_port: std::env::var("SSH_PORT").expect("SSH_PORT must be set").parse().expect("SSH_PORT must be a number"),
    };
    let client = connect::Connection::new(config)
        .await
        .expect("Failed to connect")
        .sftp_client;
    let path = std::path::Path::new("./test.txt");

    assert!(client.create(path).await.is_ok());
    assert!(client.fs().remove_file(path).await.is_ok());

    client.close().await.expect("Failed to close");
}
