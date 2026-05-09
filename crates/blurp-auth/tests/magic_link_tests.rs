mod common;

use blurp_auth::MagicLinkService;
use common::TestDb;

#[tokio::test]
async fn test_magic_link_create_and_verify() {
    let db = TestDb::new().await;
    let service = MagicLinkService::new(db.pool.clone(), 900);

    let email = "user@example.com";
    let token = service.create(email).await.expect("Failed to create token");

    assert!(!token.is_empty());

    let valid = service
        .verify(email, &token)
        .await
        .expect("Failed to verify token");
    assert!(valid);
    
    db.cleanup().await;
}

#[tokio::test]
async fn test_magic_link_expired() {
    let db = TestDb::new().await;
    let service = MagicLinkService::new(db.pool.clone(), 0);

    let email = "user@example.com";
    let token = service.create(email).await.expect("Failed to create token");

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let valid = service
        .verify(email, &token)
        .await
        .expect("Failed to verify token");
    assert!(!valid);
    
    db.cleanup().await;
}

#[tokio::test]
async fn test_magic_link_one_time_use() {
    let db = TestDb::new().await;
    let service = MagicLinkService::new(db.pool.clone(), 900);

    let email = "user@example.com";
    let token = service.create(email).await.expect("Failed to create token");

    let valid1 = service
        .verify(email, &token)
        .await
        .expect("Failed to verify token");
    assert!(valid1, "First use should succeed");

    let valid2 = service
        .verify(email, &token)
        .await
        .expect("Failed to verify token");
    assert!(!valid2, "Second use should fail");
    
    db.cleanup().await;
}

#[tokio::test]
async fn test_magic_link_wrong_email() {
    let db = TestDb::new().await;
    let service = MagicLinkService::new(db.pool.clone(), 900);

    let email1 = "user1@example.com";
    let email2 = "user2@example.com";
    let token = service
        .create(email1)
        .await
        .expect("Failed to create token");

    let valid = service
        .verify(email2, &token)
        .await
        .expect("Failed to verify token");
    assert!(!valid);
    
    db.cleanup().await;
}

#[tokio::test]
async fn test_magic_link_invalid_token() {
    let db = TestDb::new().await;
    let service = MagicLinkService::new(db.pool.clone(), 900);

    let email = "user@example.com";
    service.create(email).await.expect("Failed to create token");

    let valid = service
        .verify(email, "invalid_token_xyz")
        .await
        .expect("Failed to verify token");
    assert!(!valid);
    
    db.cleanup().await;
}
