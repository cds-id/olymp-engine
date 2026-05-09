mod common;

use olymp_auth::RegistrationService;
use common::TestDb;

#[tokio::test]
async fn test_register_with_password() {
    let db = TestDb::new().await;
    let service = RegistrationService::new(db.pool.clone());

    let user_id = service
        .register_with_password("user@example.com", "testuser", "SecurePass123", Some("Test User"))
        .await
        .expect("Failed to register");

    assert!(!user_id.is_nil());
    db.cleanup().await;
}

#[tokio::test]
async fn test_register_duplicate_email() {
    let db = TestDb::new().await;
    let service = RegistrationService::new(db.pool.clone());

    service
        .register_with_password("user@example.com", "user1", "SecurePass123", None)
        .await
        .expect("Failed to register first user");

    let result = service
        .register_with_password("user@example.com", "user2", "SecurePass123", None)
        .await;

    assert!(result.is_err());
    db.cleanup().await;
}

#[tokio::test]
async fn test_register_duplicate_username() {
    let db = TestDb::new().await;
    let service = RegistrationService::new(db.pool.clone());

    service
        .register_with_password("user1@example.com", "testuser", "SecurePass123", None)
        .await
        .expect("Failed to register first user");

    let result = service
        .register_with_password("user2@example.com", "testuser", "SecurePass123", None)
        .await;

    assert!(result.is_err());
    db.cleanup().await;
}

#[test]
fn test_validate_username_too_short() {
    let result = RegistrationService::validate_username("ab");
    assert!(result.is_err());
}

#[test]
fn test_validate_username_too_long() {
    let result = RegistrationService::validate_username(&"a".repeat(51));
    assert!(result.is_err());
}

#[test]
fn test_validate_username_invalid_chars() {
    let result = RegistrationService::validate_username("user@name");
    assert!(result.is_err());
}

#[test]
fn test_validate_username_valid() {
    let result = RegistrationService::validate_username("valid_user-123");
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_get_by_username() {
    let db = TestDb::new().await;
    let service = RegistrationService::new(db.pool.clone());

    let user_id = service
        .register_with_password("user@example.com", "testuser", "SecurePass123", None)
        .await
        .expect("Failed to register");

    let result = service
        .get_by_username("testuser")
        .await
        .expect("Failed to query");

    assert!(result.is_some());
    let (id, _) = result.unwrap();
    assert_eq!(id, user_id);
    db.cleanup().await;
}
