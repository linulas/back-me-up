use crate::ssh::connect;
use dotenv::dotenv;

#[actix_rt::test]
async fn test_connection() {
    dotenv().ok();
    let connection = connect::to_home_server().await;
    if let Err(e) = &connection {
        eprintln!("{e:?}");
    }
    assert!(connection.is_ok());

    connection.unwrap().close().await.unwrap();
}

#[actix_rt::test]
async fn test_sftp_client() {
    dotenv().ok();
    let session = connect::to_home_server().await.expect("Failed to connect");
    let client = connect::Connection::new(session).await.client;
    let path = std::path::Path::new("./test.txt");

    assert!(client.create(path).await.is_ok());
    assert!(client.fs().remove_file(path).await.is_ok());

    client.close().await.unwrap();
}
