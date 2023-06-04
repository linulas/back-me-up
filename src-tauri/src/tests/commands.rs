use crate::commands;
use dotenv::dotenv;

#[actix_rt::test]
async fn test_list_folders() {
    dotenv().ok();
    let result = commands::list_home_folders().await;

    if let Err(e) = &result {
        eprintln!("{e:?}");
    }

    assert!(result.is_ok());
}
