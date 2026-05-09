mod common;

use olymp_auth::OAuthService;
use common::{create_test_user, TestDb};
use uuid::Uuid;

#[tokio::test]
async fn test_link_oauth_provider() {
    let db = TestDb::new().await;
    let service = OAuthService::new(db.pool.clone());
    let user_id = create_test_user(&db.pool, "user@example.com").await;

    service
        .link_provider(user_id, "google", "google_123", Some("user@gmail.com"), Some("access_token"), None)
        .await
        .expect("Failed to link provider");

    let result = service
        .get_user_by_provider("google", "google_123")
        .await
        .expect("Failed to query");

    assert_eq!(result, Some(user_id));
    db.cleanup().await;
}

#[tokio::test]
async fn test_get_user_by_provider_not_found() {
    let db = TestDb::new().await;
    let service = OAuthService::new(db.pool.clone());

    let result = service
        .get_user_by_provider("google", "nonexistent")
        .await
        .expect("Failed to query");

    assert!(result.is_none());
    db.cleanup().await;
}

#[tokio::test]
async fn test_unlink_provider() {
    let db = TestDb::new().await;
    let service = OAuthService::new(db.pool.clone());
    let user_id = create_test_user(&db.pool, "user@example.com").await;

    service
        .link_provider(user_id, "google", "google_123", None, None, None)
        .await
        .expect("Failed to link");

    service
        .unlink_provider(user_id, "google")
        .await
        .expect("Failed to unlink");

    let result = service
        .get_user_by_provider("google", "google_123")
        .await
        .expect("Failed to query");

    assert!(result.is_none());
    db.cleanup().await;
}
