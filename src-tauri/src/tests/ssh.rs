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
