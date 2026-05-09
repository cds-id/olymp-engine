use blurp_notification::{EmailService, MailgunProvider, SmtpProvider};
use blurp_notification::models::*;

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
                verification_link: "https://sorastore.com/verify?token=abc123xyz".into(),
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
                magic_link: "https://sorastore.com/auth/callback?token=abc123xyz".into(),
                expires_in_minutes: 15,
            }).await
        }
        "reset" => {
            svc.send_password_reset(to_email, PasswordResetData {
                name: "John Doe".into(),
                reset_link: "https://sorastore.com/reset?token=abc123xyz".into(),
                expires_in_minutes: 30,
            }).await
        }
        "order" => {
            svc.send_order_confirmation(to_email, OrderConfirmationData {
                name: "John Doe".into(),
                order_number: "ORD-20260505-1234".into(),
                total_idr: 1250000,
                items: vec![
                    OrderItemData {
                        product_name: "Premium Wireless Headphones".into(),
                        variant_name: "Black / Large".into(),
                        quantity: 1,
                        price_idr: 899000,
                    },
                    OrderItemData {
                        product_name: "USB-C Charging Cable".into(),
                        variant_name: "2m / White".into(),
                        quantity: 2,
                        price_idr: 175500,
                    },
                ],
                shipping_address: "John Doe\nJl. Sudirman No. 123\nJakarta Selatan 12190\nIndonesia".into(),
                tracking_link: Some("https://sorastore.com/track?token=guest123".into()),
            }).await
        }
        "shipped" => {
            svc.send_order_shipped(to_email, OrderShippedData {
                name: "John Doe".into(),
                order_number: "ORD-20260505-1234".into(),
                courier: "JNE Express".into(),
                tracking_number: "JNE1234567890".into(),
                tracking_link: Some("https://jne.co.id/track/JNE1234567890".into()),
            }).await
        }
        "guest" => {
            svc.send_guest_tracking(to_email, GuestTrackingData {
                name: "John Doe".into(),
                order_number: "ORD-20260505-1234".into(),
                tracking_link: "https://sorastore.com/track?token=guest123abc".into(),
            }).await
        }
        _ => {
            println!("Unknown type: {}. Options: register, welcome, magic, reset, order, shipped, guest", email_type);
            return;
        }
    };
    
    match result {
        Ok(_) => println!("✓ Email sent successfully!"),
        Err(e) => println!("✗ Error: {:?}", e),
    }
}
