mod common;

use blurp_auth::JwtService;
use uuid::Uuid;

#[test]
fn test_jwt_issue_and_verify() {
    let service = JwtService::new("test-secret-key-32-chars-long!".to_string(), 900);
    let user_id = Uuid::now_v7();
    let email = "user@example.com".to_string();

    let token = service
        .issue(user_id, email.clone())
        .expect("Failed to issue JWT");

    assert!(!token.is_empty());

    let claims = service.verify(&token).expect("Failed to verify JWT");
    assert_eq!(claims.sub, user_id.to_string());
    assert_eq!(claims.email, email);
}

#[test]
fn test_jwt_invalid_token() {
    let service = JwtService::new("test-secret-key-32-chars-long!".to_string(), 900);

    let result = service.verify("invalid.token.here");
    assert!(result.is_err());
}

#[test]
fn test_jwt_wrong_secret() {
    let service1 = JwtService::new("secret-key-1-32-chars-long!!!".to_string(), 900);
    let service2 = JwtService::new("secret-key-2-32-chars-long!!!".to_string(), 900);

    let user_id = Uuid::now_v7();
    let email = "user@example.com".to_string();

    let token = service1
        .issue(user_id, email)
        .expect("Failed to issue JWT");

    let result = service2.verify(&token);
    assert!(result.is_err());
}
