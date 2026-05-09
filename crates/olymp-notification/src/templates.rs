use crate::models::*;

// ─── Configuration ─────────────────────────────────────────

// Logo URL - update when image hosted
// Currently using CSS text logo in template
const BRAND_COLOR: &str = "#6366f1";  // Indigo
const BRAND_COLOR_DARK: &str = "#4f46e5";
const STORE_NAME: &str = "Olymp LMS";
const STORE_URL: &str = "https://olymp.id";
const SUPPORT_EMAIL: &str = "support@olymp.id";

// ─── Base Template ─────────────────────────────────────────

fn base_html(title: &str, content: &str, preheader: &str) -> String {
    format!(r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta http-equiv="X-UA-Compatible" content="IE=edge">
    <title>{title}</title>
    <!--[if mso]>
    <noscript>
        <xml>
            <o:OfficeDocumentSettings>
                <o:PixelsPerInch>96</o:PixelsPerInch>
            </o:OfficeDocumentSettings>
        </xml>
    </noscript>
    <![endif]-->
    <style>
        /* Reset */
        body, table, td, p, a, li {{ -webkit-text-size-adjust: 100%; -ms-text-size-adjust: 100%; }}
        table, td {{ mso-table-lspace: 0pt; mso-table-rspace: 0pt; }}
        img {{ -ms-interpolation-mode: bicubic; border: 0; height: auto; line-height: 100%; outline: none; text-decoration: none; }}
        body {{ margin: 0 !important; padding: 0 !important; width: 100% !important; }}
        
        /* Typography */
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif; }}
        
        /* Button */
        .btn {{
            display: inline-block;
            padding: 14px 32px;
            background: {brand};
            color: #ffffff !important;
            text-decoration: none;
            border-radius: 8px;
            font-weight: 600;
            font-size: 16px;
            text-align: center;
            transition: background 0.2s;
        }}
        .btn:hover {{ background: {brand_dark}; }}
        
        /* Dark mode */
        @media (prefers-color-scheme: dark) {{
            .dark-bg {{ background-color: #1f2937 !important; }}
            .dark-text {{ color: #f3f4f6 !important; }}
            .dark-muted {{ color: #9ca3af !important; }}
        }}
        
        /* Mobile */
        @media only screen and (max-width: 600px) {{
            .container {{ width: 100% !important; padding: 16px !important; }}
            .card {{ padding: 24px !important; }}
            .btn {{ padding: 12px 24px !important; }}
        }}
    </style>
</head>
<body style="margin: 0; padding: 0; background-color: #f3f4f6;">
    <!-- Preheader -->
    <div style="display: none; max-height: 0; overflow: hidden; mso-hide: all;">
        {preheader}
        &nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;
    </div>
    
    <!-- Email Body -->
    <table role="presentation" cellspacing="0" cellpadding="0" border="0" width="100%" style="background-color: #f3f4f6;">
        <tr>
            <td align="center" style="padding: 40px 16px;">
                <table role="presentation" cellspacing="0" cellpadding="0" border="0" width="100%" style="max-width: 600px;">
                    
                    <!-- Header with Logo -->
                    <tr>
                        <td align="center" style="padding-bottom: 32px;">
                            <a href="{store_url}" target="_blank" style="text-decoration: none;">
                                <table role="presentation" cellspacing="0" cellpadding="0" border="0">
                                    <tr>
                                        <td style="background: linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%); width: 44px; height: 44px; border-radius: 10px; text-align: center; vertical-align: middle;">
                                            <span style="color: #ffffff; font-size: 22px; font-weight: bold;">S</span>
                                        </td>
                                        <td style="padding-left: 12px;">
                                            <span style="font-size: 28px; font-weight: 700; color: #111827;">Sora</span><span style="font-size: 28px; font-weight: 700; color: #6366f1;">Store</span>
                                        </td>
                                    </tr>
                                </table>
                            </a>
                        </td>
                    </tr>
                    
                    <!-- Main Card -->
                    <tr>
                        <td>
                            <table role="presentation" cellspacing="0" cellpadding="0" border="0" width="100%" style="background: #ffffff; border-radius: 16px; box-shadow: 0 4px 6px -1px rgba(0,0,0,0.1), 0 2px 4px -1px rgba(0,0,0,0.06);">
                                <tr>
                                    <td class="card" style="padding: 40px;">
                                        {content}
                                    </td>
                                </tr>
                            </table>
                        </td>
                    </tr>
                    
                    <!-- Footer -->
                    <tr>
                        <td style="padding: 32px 16px; text-align: center;">
                            <p style="margin: 0 0 8px; font-size: 14px; color: #6b7280;">
                                &copy; 2026 {store}. All rights reserved.
                            </p>
                            <p style="margin: 0 0 16px; font-size: 13px; color: #9ca3af;">
                                Questions? Contact us at <a href="mailto:{support}" style="color: {brand};">{support}</a>
                            </p>
                            <p style="margin: 0; font-size: 12px; color: #9ca3af;">
                                You received this email because you signed up for {store}.<br>
                                If you didn't request this, you can safely ignore it.
                            </p>
                        </td>
                    </tr>
                    
                </table>
            </td>
        </tr>
    </table>
</body>
</html>"##,
        title = title,
        preheader = preheader,
        content = content,
        store = STORE_NAME,
        store_url = STORE_URL,
        support = SUPPORT_EMAIL,
        brand = BRAND_COLOR,
        brand_dark = BRAND_COLOR_DARK
    )
}

// ─── Registration ──────────────────────────────────────────

pub fn registration_html(data: &RegistrationData) -> String {
    let content = format!(r##"
        <div style="text-align: center; margin-bottom: 32px;">
            <div style="width: 64px; height: 64px; background: linear-gradient(135deg, {brand} 0%, {brand_dark} 100%); border-radius: 50%; margin: 0 auto 16px; display: flex; align-items: center; justify-content: center;">
                <span style="font-size: 32px;">✉️</span>
            </div>
            <h1 style="margin: 0 0 8px; font-size: 28px; font-weight: 700; color: #111827;">Verify Your Email</h1>
            <p style="margin: 0; font-size: 16px; color: #6b7280;">One quick step to get started</p>
        </div>
        
        <p style="margin: 0 0 24px; font-size: 16px; line-height: 1.6; color: #374151;">
            Hi <strong>{name}</strong>,
        </p>
        <p style="margin: 0 0 24px; font-size: 16px; line-height: 1.6; color: #374151;">
            Welcome to Olymp LMS! Please verify your email address to complete your registration and start shopping.
        </p>
        
        <div style="text-align: center; margin: 32px 0;">
            <a href="{link}" class="btn" style="display: inline-block; padding: 14px 32px; background: {brand}; color: #ffffff; text-decoration: none; border-radius: 8px; font-weight: 600; font-size: 16px;">
                Verify Email Address
            </a>
        </div>
        
        <p style="margin: 24px 0 0; font-size: 14px; color: #6b7280; text-align: center;">
            Button not working? Copy this link:
        </p>
        <p style="margin: 8px 0 0; font-size: 13px; color: {brand}; word-break: break-all; text-align: center;">
            {link}
        </p>
    "##,
        name = data.name,
        link = data.verification_link,
        brand = BRAND_COLOR,
        brand_dark = BRAND_COLOR_DARK,
    );
    
    base_html(
        "Verify your email - Olymp LMS",
        &content,
        "Please verify your email address to complete registration"
    )
}

pub fn registration_text(data: &RegistrationData) -> String {
    format!(r#"Hi {name},

Welcome to Olymp LMS! Please verify your email address to complete your registration.

Click here to verify: {link}

If you didn't create this account, please ignore this email.

- The Olymp LMS Team"#,
        name = data.name,
        link = data.verification_link
    )
}

// ─── Welcome ───────────────────────────────────────────────

pub fn welcome_html(data: &WelcomeData) -> String {
    let content = format!(r##"
        <div style="text-align: center; margin-bottom: 32px;">
            <div style="width: 64px; height: 64px; background: linear-gradient(135deg, #10b981 0%, #059669 100%); border-radius: 50%; margin: 0 auto 16px; display: flex; align-items: center; justify-content: center;">
                <span style="font-size: 32px;">🎉</span>
            </div>
            <h1 style="margin: 0 0 8px; font-size: 28px; font-weight: 700; color: #111827;">Welcome to Olymp LMS!</h1>
            <p style="margin: 0; font-size: 16px; color: #6b7280;">Your account is ready</p>
        </div>
        
        <p style="margin: 0 0 24px; font-size: 16px; line-height: 1.6; color: #374151;">
            Hi <strong>{name}</strong>,
        </p>
        <p style="margin: 0 0 24px; font-size: 16px; line-height: 1.6; color: #374151;">
            Your email has been verified and your account is all set up! Here's what you can do:
        </p>
        
        <table role="presentation" cellspacing="0" cellpadding="0" border="0" width="100%" style="margin: 24px 0;">
            <tr>
                <td style="padding: 16px; background: #f9fafb; border-radius: 12px;">
                    <table role="presentation" cellspacing="0" cellpadding="0" border="0" width="100%">
                        <tr>
                            <td width="40" style="vertical-align: top; padding-right: 12px;">
                                <span style="font-size: 24px;">🛍️</span>
                            </td>
                            <td>
                                <p style="margin: 0 0 4px; font-weight: 600; color: #111827;">Browse Products</p>
                                <p style="margin: 0; font-size: 14px; color: #6b7280;">Discover our curated collection</p>
                            </td>
                        </tr>
                    </table>
                </td>
            </tr>
            <tr><td style="height: 12px;"></td></tr>
            <tr>
                <td style="padding: 16px; background: #f9fafb; border-radius: 12px;">
                    <table role="presentation" cellspacing="0" cellpadding="0" border="0" width="100%">
                        <tr>
                            <td width="40" style="vertical-align: top; padding-right: 12px;">
                                <span style="font-size: 24px;">❤️</span>
                            </td>
                            <td>
                                <p style="margin: 0 0 4px; font-weight: 600; color: #111827;">Save Favorites</p>
                                <p style="margin: 0; font-size: 14px; color: #6b7280;">Save your favorite events</p>
                            </td>
                        </tr>
                    </table>
                </td>
            </tr>
            <tr><td style="height: 12px;"></td></tr>
            <tr>
                <td style="padding: 16px; background: #f9fafb; border-radius: 12px;">
                    <table role="presentation" cellspacing="0" cellpadding="0" border="0" width="100%">
                        <tr>
                            <td width="40" style="vertical-align: top; padding-right: 12px;">
                                <span style="font-size: 24px;">📦</span>
                            </td>
                            <td>
                                <p style="margin: 0 0 4px; font-weight: 600; color: #111827;">Track Orders</p>
                                <p style="margin: 0; font-size: 14px; color: #6b7280;">Real-time delivery updates</p>
                            </td>
                        </tr>
                    </table>
                </td>
            </tr>
        </table>
        
        <div style="text-align: center; margin: 32px 0;">
            <a href="{store_url}" class="btn" style="display: inline-block; padding: 14px 32px; background: {brand}; color: #ffffff; text-decoration: none; border-radius: 8px; font-weight: 600; font-size: 16px;">
                Start Shopping
            </a>
        </div>
    "##,
        name = data.name,
        store_url = STORE_URL,
        brand = BRAND_COLOR,
    );
    
    base_html(
        "Welcome to Olymp LMS!",
        &content,
        "Your account is ready - start shopping now!"
    )
}

pub fn welcome_text(data: &WelcomeData) -> String {
    format!(r#"Hi {name},

Welcome to Olymp LMS! Your account is verified and ready.

What you can do:
- Browse our curated product collection
- Save favorite events  
- Track orders with real-time updates

Start shopping: {store_url}

- The Olymp LMS Team"#,
        name = data.name,
        store_url = STORE_URL
    )
}

// ─── Magic Link ────────────────────────────────────────────

pub fn magic_link_html(data: &MagicLinkData) -> String {
    let content = format!(r##"
        <div style="text-align: center; margin-bottom: 32px;">
            <div style="width: 64px; height: 64px; background: linear-gradient(135deg, {brand} 0%, {brand_dark} 100%); border-radius: 50%; margin: 0 auto 16px; display: flex; align-items: center; justify-content: center;">
                <span style="font-size: 32px;">🔐</span>
            </div>
            <h1 style="margin: 0 0 8px; font-size: 28px; font-weight: 700; color: #111827;">Sign In to Olymp LMS</h1>
            <p style="margin: 0; font-size: 16px; color: #6b7280;">Your magic link is ready</p>
        </div>
        
        <p style="margin: 0 0 24px; font-size: 16px; line-height: 1.6; color: #374151;">
            Hi <strong>{name}</strong>,
        </p>
        <p style="margin: 0 0 24px; font-size: 16px; line-height: 1.6; color: #374151;">
            Click the button below to sign in to your account. This link will expire in <strong>{expires} minutes</strong>.
        </p>
        
        <div style="text-align: center; margin: 32px 0;">
            <a href="{link}" class="btn" style="display: inline-block; padding: 14px 32px; background: {brand}; color: #ffffff; text-decoration: none; border-radius: 8px; font-weight: 600; font-size: 16px;">
                Sign In Now
            </a>
        </div>
        
        <div style="background: #fef3c7; border-radius: 8px; padding: 16px; margin: 24px 0;">
            <p style="margin: 0; font-size: 14px; color: #92400e;">
                <strong>⚠️ Security Notice:</strong> If you didn't request this login link, please ignore this email. Your account is safe.
            </p>
        </div>
        
        <p style="margin: 24px 0 0; font-size: 14px; color: #6b7280; text-align: center;">
            Button not working? Copy this link:
        </p>
        <p style="margin: 8px 0 0; font-size: 13px; color: {brand}; word-break: break-all; text-align: center;">
            {link}
        </p>
    "##,
        name = data.name,
        link = data.magic_link,
        expires = data.expires_in_minutes,
        brand = BRAND_COLOR,
        brand_dark = BRAND_COLOR_DARK,
    );
    
    base_html(
        "Sign in to Olymp LMS",
        &content,
        &format!("Your login link (expires in {} minutes)", data.expires_in_minutes)
    )
}

pub fn magic_link_text(data: &MagicLinkData) -> String {
    format!(r#"Hi {name},

Click this link to sign in to your Olymp LMS account. This link expires in {expires} minutes:

{link}

If you didn't request this, please ignore this email.

- The Olymp LMS Team"#,
        name = data.name,
        expires = data.expires_in_minutes,
        link = data.magic_link
    )
}

// ─── Password Reset ────────────────────────────────────────

pub fn password_reset_html(data: &PasswordResetData) -> String {
    let content = format!(r##"
        <div style="text-align: center; margin-bottom: 32px;">
            <div style="width: 64px; height: 64px; background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%); border-radius: 50%; margin: 0 auto 16px; display: flex; align-items: center; justify-content: center;">
                <span style="font-size: 32px;">🔑</span>
            </div>
            <h1 style="margin: 0 0 8px; font-size: 28px; font-weight: 700; color: #111827;">Reset Your Password</h1>
            <p style="margin: 0; font-size: 16px; color: #6b7280;">Let's get you back in</p>
        </div>
        
        <p style="margin: 0 0 24px; font-size: 16px; line-height: 1.6; color: #374151;">
            Hi <strong>{name}</strong>,
        </p>
        <p style="margin: 0 0 24px; font-size: 16px; line-height: 1.6; color: #374151;">
            We received a request to reset your password. Click the button below to create a new password. This link expires in <strong>{expires} minutes</strong>.
        </p>
        
        <div style="text-align: center; margin: 32px 0;">
            <a href="{link}" class="btn" style="display: inline-block; padding: 14px 32px; background: #f59e0b; color: #ffffff; text-decoration: none; border-radius: 8px; font-weight: 600; font-size: 16px;">
                Reset Password
            </a>
        </div>
        
        <div style="background: #fef3c7; border-radius: 8px; padding: 16px; margin: 24px 0;">
            <p style="margin: 0; font-size: 14px; color: #92400e;">
                <strong>⚠️ Didn't request this?</strong> If you didn't ask to reset your password, you can safely ignore this email. Your password won't change.
            </p>
        </div>
    "##,
        name = data.name,
        link = data.reset_link,
        expires = data.expires_in_minutes,
    );
    
    base_html(
        "Reset your password - Olymp LMS",
        &content,
        "Reset your Olymp LMS password"
    )
}

pub fn password_reset_text(data: &PasswordResetData) -> String {
    format!(r#"Hi {name},

We received a request to reset your password. Click this link to create a new password (expires in {expires} minutes):

{link}

If you didn't request this, please ignore this email.

- The Olymp LMS Team"#,
        name = data.name,
        expires = data.expires_in_minutes,
        link = data.reset_link
    )
}

// ─── Order Confirmation ────────────────────────────────────

