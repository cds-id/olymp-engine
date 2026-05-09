mod common;

use olymp_auth::UserService;
use common::{create_test_user, TestDb};

#[tokio::test]
async fn test_user_get_or_create_new() {
    let db = TestDb::new().await;
    let service = UserService::new(db.pool.clone());

    let email = "newuser@example.com";
    let user_id = service
        .get_or_create(email)
        .await
        .expect("Failed to get or create user");

    assert!(!user_id.is_nil());

    let user = service
        .get_by_id(user_id)
        .await
        .expect("Failed to get user")
        .expect("User not found");
    assert_eq!(user.1, email);
}

#[tokio::test]
async fn test_user_get_or_create_existing() {
    let db = TestDb::new().await;
    let service = UserService::new(db.pool.clone());

    let email = "existing@example.com";
    let user_id1 = create_test_user(&db.pool, email).await;

    let user_id2 = service
        .get_or_create(email)
        .await
        .expect("Failed to get or create user");

    assert_eq!(user_id1, user_id2);
}

#[tokio::test]
async fn test_user_get_by_id_not_found() {
    let db = TestDb::new().await;
    let service = UserService::new(db.pool.clone());

    let fake_id = uuid::Uuid::now_v7();
    let user = service
        .get_by_id(fake_id)
        .await
        .expect("Failed to query user");

    assert!(user.is_none());
}
