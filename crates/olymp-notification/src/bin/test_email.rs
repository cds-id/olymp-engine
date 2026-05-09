use olymp_notification::{EmailService, MailgunProvider, SmtpProvider};
use olymp_notification::models::*;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenvy::from_filename(".env.local").ok();

    let args: Vec<String> = std::env::args().collect();
    let provider_type = args.get(1).map(|s| s.as_str()).unwrap_or("mailgun");
    let email_type = args.get(2).map(|s| s.as_str()).unwrap_or("magic");
    let to_email = args.get(3).map(|s| s.as_str()).unwrap_or("test@example.com");

    println!("Provider: {}, Type: {}, To: {}", provider_type, email_type, to_email);

    let svc: EmailService = match provider_type {
        "smtp" => {
            let provider = SmtpProvider::from_env().expect("SMTP config");
            EmailService::new(Box::new(provider))
        }
        _ => {
            let provider = MailgunProvider::from_env().expect("Mailgun config");
            EmailService::new(Box::new(provider))
        }
    };

    let result = match email_type {
        "register" => {
            svc.send_registration(to_email, RegistrationData {
                name: "John Doe".into(),
                verification_link: "https://olymp.id/verify?token=abc123xyz".into(),
            }).await
        }
        "welcome" => {
            svc.send_welcome(to_email, WelcomeData {
                name: "John Doe".into(),
            }).await
        }
        "magic" => {
            svc.send_magic_link(to_email, MagicLinkData {
                name: "John Doe".into(),
                magic_link: "https://olymp.id/auth/callback?token=abc123xyz".into(),
                expires_in_minutes: 15,
            }).await
        }
        "reset" => {
            svc.send_password_reset(to_email, PasswordResetData {
                name: "John Doe".into(),
                reset_link: "https://olymp.id/reset?token=abc123xyz".into(),
                expires_in_minutes: 30,
            }).await
        }
        _ => {
            println!("Unknown type: {}. Options: register, welcome, magic, reset", email_type);
            return;
        }
    };

    match result {
        Ok(_) => println!("✓ Email sent successfully!"),
        Err(e) => println!("✗ Error: {:?}", e),
    }
}
