mod common;

use olymp_auth::PasswordService;

#[test]
fn test_password_hash_and_verify() {
    let password = "SecurePass123";
    let hash = PasswordService::hash_password(password).expect("Failed to hash");
    
    let valid = PasswordService::verify_password(password, &hash)
        .expect("Failed to verify");
    assert!(valid);
}

#[test]
fn test_password_verify_wrong() {
    let password = "SecurePass123";
    let hash = PasswordService::hash_password(password).expect("Failed to hash");
    
    let valid = PasswordService::verify_password("WrongPass123", &hash)
        .expect("Failed to verify");
    assert!(!valid);
}

#[test]
fn test_password_validate_too_short() {
    let result = PasswordService::validate_password("Short1");
    assert!(result.is_err());
}

#[test]
fn test_password_validate_no_uppercase() {
    let result = PasswordService::validate_password("lowercase123");
    assert!(result.is_err());
}

#[test]
fn test_password_validate_no_number() {
    let result = PasswordService::validate_password("NoNumbers");
    assert!(result.is_err());
}

#[test]
fn test_password_validate_strong() {
    let result = PasswordService::validate_password("StrongPass123");
    assert!(result.is_ok());
}
