use crate::models::*;

// ─── Configuration ─────────────────────────────────────────

// Logo URL - update when image hosted
// Currently using CSS text logo in template
const BRAND_COLOR: &str = "#6366f1";  // Indigo
const BRAND_COLOR_DARK: &str = "#4f46e5";
const STORE_NAME: &str = "SoraStore";
const STORE_URL: &str = "https://sorastore.com";
const SUPPORT_EMAIL: &str = "support@sorastore.com";

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

fn format_idr(amount: i64) -> String {
    let s = amount.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.insert(0, '.');
        }
        result.insert(0, c);
    }
    format!("Rp {}", result)
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
            Welcome to SoraStore! Please verify your email address to complete your registration and start shopping.
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
        "Verify your email - SoraStore",
        &content,
        "Please verify your email address to complete registration"
    )
}

pub fn registration_text(data: &RegistrationData) -> String {
    format!(r#"Hi {name},

Welcome to SoraStore! Please verify your email address to complete your registration.

Click here to verify: {link}

If you didn't create this account, please ignore this email.

- The SoraStore Team"#,
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
            <h1 style="margin: 0 0 8px; font-size: 28px; font-weight: 700; color: #111827;">Welcome to SoraStore!</h1>
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
                                <p style="margin: 0; font-size: 14px; color: #6b7280;">Build your wishlist</p>
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
        "Welcome to SoraStore!",
        &content,
        "Your account is ready - start shopping now!"
    )
}

pub fn welcome_text(data: &WelcomeData) -> String {
    format!(r#"Hi {name},

Welcome to SoraStore! Your account is verified and ready.

What you can do:
- Browse our curated product collection
- Save favorites to your wishlist  
- Track orders with real-time updates

Start shopping: {store_url}

- The SoraStore Team"#,
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
            <h1 style="margin: 0 0 8px; font-size: 28px; font-weight: 700; color: #111827;">Sign In to SoraStore</h1>
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
        "Sign in to SoraStore",
        &content,
        &format!("Your login link (expires in {} minutes)", data.expires_in_minutes)
    )
}

pub fn magic_link_text(data: &MagicLinkData) -> String {
    format!(r#"Hi {name},

Click this link to sign in to your SoraStore account. This link expires in {expires} minutes:

{link}

If you didn't request this, please ignore this email.

- The SoraStore Team"#,
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
        "Reset your password - SoraStore",
        &content,
        "Reset your SoraStore password"
    )
}

pub fn password_reset_text(data: &PasswordResetData) -> String {
    format!(r#"Hi {name},

We received a request to reset your password. Click this link to create a new password (expires in {expires} minutes):

{link}

If you didn't request this, please ignore this email.

- The SoraStore Team"#,
        name = data.name,
        expires = data.expires_in_minutes,
        link = data.reset_link
    )
}

// ─── Order Confirmation ────────────────────────────────────

pub fn order_confirmation_html(data: &OrderConfirmationData) -> String {
    let mut items_html = String::new();
    for item in &data.items {
        items_html.push_str(&format!(r##"
            <tr>
                <td style="padding: 12px 0; border-bottom: 1px solid #e5e7eb;">
                    <p style="margin: 0 0 4px; font-weight: 600; color: #111827;">{product}</p>
                    <p style="margin: 0; font-size: 14px; color: #6b7280;">{variant}</p>
                </td>
                <td style="padding: 12px 0; border-bottom: 1px solid #e5e7eb; text-align: center; color: #6b7280;">
                    x{qty}
                </td>
                <td style="padding: 12px 0; border-bottom: 1px solid #e5e7eb; text-align: right; font-weight: 600; color: #111827;">
                    {price}
                </td>
            </tr>
        "##,
            product = item.product_name,
            variant = item.variant_name,
            qty = item.quantity,
            price = format_idr(item.price_idr)
        ));
    }

    let tracking_section = if let Some(ref link) = data.tracking_link {
        format!(r##"
            <div style="background: #f0fdf4; border-radius: 8px; padding: 16px; margin: 24px 0; text-align: center;">
                <p style="margin: 0 0 12px; font-size: 14px; color: #166534;">
                    <strong>📧 Track Your Order</strong>
                </p>
                <a href="{link}" style="color: {brand}; font-size: 14px;">Click here to track your order anytime</a>
            </div>
        "##, link = link, brand = BRAND_COLOR)
    } else {
        String::new()
    };

    let content = format!(r##"
        <div style="text-align: center; margin-bottom: 32px;">
            <div style="width: 64px; height: 64px; background: linear-gradient(135deg, #10b981 0%, #059669 100%); border-radius: 50%; margin: 0 auto 16px; display: flex; align-items: center; justify-content: center;">
                <span style="font-size: 32px;">✅</span>
            </div>
            <h1 style="margin: 0 0 8px; font-size: 28px; font-weight: 700; color: #111827;">Order Confirmed!</h1>
            <p style="margin: 0; font-size: 16px; color: #6b7280;">Order #{order_number}</p>
        </div>
        
        <p style="margin: 0 0 24px; font-size: 16px; line-height: 1.6; color: #374151;">
            Hi <strong>{name}</strong>,
        </p>
        <p style="margin: 0 0 24px; font-size: 16px; line-height: 1.6; color: #374151;">
            Thank you for your order! We've received it and will start processing soon.
        </p>
        
        <!-- Order Items -->
        <div style="background: #f9fafb; border-radius: 12px; padding: 20px; margin: 24px 0;">
            <h3 style="margin: 0 0 16px; font-size: 16px; font-weight: 600; color: #111827;">Order Summary</h3>
            <table role="presentation" cellspacing="0" cellpadding="0" border="0" width="100%">
                <thead>
                    <tr>
                        <th style="padding: 8px 0; text-align: left; font-size: 12px; font-weight: 600; color: #6b7280; text-transform: uppercase; border-bottom: 2px solid #e5e7eb;">Item</th>
                        <th style="padding: 8px 0; text-align: center; font-size: 12px; font-weight: 600; color: #6b7280; text-transform: uppercase; border-bottom: 2px solid #e5e7eb;">Qty</th>
                        <th style="padding: 8px 0; text-align: right; font-size: 12px; font-weight: 600; color: #6b7280; text-transform: uppercase; border-bottom: 2px solid #e5e7eb;">Price</th>
                    </tr>
                </thead>
                <tbody>
                    {items}
                    <tr>
                        <td colspan="2" style="padding: 16px 0 0; text-align: right; font-size: 18px; font-weight: 700; color: #111827;">
                            Total
                        </td>
                        <td style="padding: 16px 0 0; text-align: right; font-size: 18px; font-weight: 700; color: {brand};">
                            {total}
                        </td>
                    </tr>
                </tbody>
            </table>
        </div>
        
        <!-- Shipping Address -->
        <div style="background: #f9fafb; border-radius: 12px; padding: 20px; margin: 24px 0;">
            <h3 style="margin: 0 0 12px; font-size: 16px; font-weight: 600; color: #111827;">📍 Shipping To</h3>
            <p style="margin: 0; font-size: 14px; line-height: 1.6; color: #374151;">
                {address}
            </p>
        </div>
        
        {tracking}
        
        <div style="text-align: center; margin: 32px 0;">
            <a href="{store_url}/orders" class="btn" style="display: inline-block; padding: 14px 32px; background: {brand}; color: #ffffff; text-decoration: none; border-radius: 8px; font-weight: 600; font-size: 16px;">
                View Order Details
            </a>
        </div>
    "##,
        name = data.name,
        order_number = data.order_number,
        items = items_html,
        total = format_idr(data.total_idr),
        address = data.shipping_address.replace("\n", "<br>"),
        tracking = tracking_section,
        store_url = STORE_URL,
        brand = BRAND_COLOR,
    );
    
    base_html(
        &format!("Order Confirmed - {}", data.order_number),
        &content,
        &format!("Your order {} has been confirmed", data.order_number)
    )
}

pub fn order_confirmation_text(data: &OrderConfirmationData) -> String {
    let mut items_text = String::new();
    for item in &data.items {
        items_text.push_str(&format!("• {} x {} ({}): {}\n", 
            item.quantity, item.product_name, item.variant_name, format_idr(item.price_idr)));
    }

    let tracking = if let Some(ref link) = data.tracking_link {
        format!("\nTrack your order: {}\n", link)
    } else {
        String::new()
    };

    format!(r#"Hi {name},

Thank you for your order! Order #{order_number} has been confirmed.

ORDER SUMMARY
{items}
Total: {total}

SHIPPING TO
{address}
{tracking}
- The SoraStore Team"#,
        name = data.name,
        order_number = data.order_number,
        items = items_text,
        total = format_idr(data.total_idr),
        address = data.shipping_address,
        tracking = tracking
    )
}

// ─── Order Shipped ─────────────────────────────────────────

pub fn order_shipped_html(data: &OrderShippedData) -> String {
    let tracking_btn = if let Some(ref link) = data.tracking_link {
        format!(r##"
            <div style="text-align: center; margin: 32px 0;">
                <a href="{link}" class="btn" style="display: inline-block; padding: 14px 32px; background: {brand}; color: #ffffff; text-decoration: none; border-radius: 8px; font-weight: 600; font-size: 16px;">
                    Track Package
                </a>
            </div>
        "##, link = link, brand = BRAND_COLOR)
    } else {
        String::new()
    };

    let content = format!(r##"
        <div style="text-align: center; margin-bottom: 32px;">
            <div style="width: 64px; height: 64px; background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%); border-radius: 50%; margin: 0 auto 16px; display: flex; align-items: center; justify-content: center;">
                <span style="font-size: 32px;">🚚</span>
            </div>
            <h1 style="margin: 0 0 8px; font-size: 28px; font-weight: 700; color: #111827;">Your Order Has Shipped!</h1>
            <p style="margin: 0; font-size: 16px; color: #6b7280;">Order #{order_number}</p>
        </div>
        
        <p style="margin: 0 0 24px; font-size: 16px; line-height: 1.6; color: #374151;">
            Hi <strong>{name}</strong>,
        </p>
        <p style="margin: 0 0 24px; font-size: 16px; line-height: 1.6; color: #374151;">
            Great news! Your order is on its way. Here are the shipping details:
        </p>
        
        <div style="background: #eff6ff; border-radius: 12px; padding: 24px; margin: 24px 0;">
            <table role="presentation" cellspacing="0" cellpadding="0" border="0" width="100%">
                <tr>
                    <td width="50%">
                        <p style="margin: 0 0 4px; font-size: 12px; font-weight: 600; color: #6b7280; text-transform: uppercase;">Courier</p>
                        <p style="margin: 0; font-size: 18px; font-weight: 600; color: #1e40af;">{courier}</p>
                    </td>
                    <td width="50%">
                        <p style="margin: 0 0 4px; font-size: 12px; font-weight: 600; color: #6b7280; text-transform: uppercase;">Tracking Number</p>
                        <p style="margin: 0; font-size: 18px; font-weight: 600; color: #1e40af; font-family: monospace;">{tracking}</p>
                    </td>
                </tr>
            </table>
        </div>
        
        {tracking_btn}
    "##,
        name = data.name,
        order_number = data.order_number,
        courier = data.courier,
        tracking = data.tracking_number,
        tracking_btn = tracking_btn,
    );
    
    base_html(
        &format!("Order Shipped - {}", data.order_number),
        &content,
        &format!("Your order {} is on its way!", data.order_number)
    )
}

pub fn order_shipped_text(data: &OrderShippedData) -> String {
    let tracking = if let Some(ref link) = data.tracking_link {
        format!("\nTrack your package: {}\n", link)
    } else {
        String::new()
    };

    format!(r#"Hi {name},

Great news! Your order #{order_number} is on its way.

Courier: {courier}
Tracking Number: {tracking_number}
{tracking}
- The SoraStore Team"#,
        name = data.name,
        order_number = data.order_number,
        courier = data.courier,
        tracking_number = data.tracking_number,
        tracking = tracking
    )
}

// ─── Guest Order Tracking ──────────────────────────────────

pub fn guest_tracking_html(data: &GuestTrackingData) -> String {
    let content = format!(r##"
        <div style="text-align: center; margin-bottom: 32px;">
            <div style="width: 64px; height: 64px; background: linear-gradient(135deg, {brand} 0%, {brand_dark} 100%); border-radius: 50%; margin: 0 auto 16px; display: flex; align-items: center; justify-content: center;">
                <span style="font-size: 32px;">📦</span>
            </div>
            <h1 style="margin: 0 0 8px; font-size: 28px; font-weight: 700; color: #111827;">Track Your Order</h1>
            <p style="margin: 0; font-size: 16px; color: #6b7280;">Order #{order_number}</p>
        </div>
        
        <p style="margin: 0 0 24px; font-size: 16px; line-height: 1.6; color: #374151;">
            Hi <strong>{name}</strong>,
        </p>
        <p style="margin: 0 0 24px; font-size: 16px; line-height: 1.6; color: #374151;">
            Use the link below to track your order status anytime. This link is valid for 30 days.
        </p>
        
        <div style="text-align: center; margin: 32px 0;">
            <a href="{link}" class="btn" style="display: inline-block; padding: 14px 32px; background: {brand}; color: #ffffff; text-decoration: none; border-radius: 8px; font-weight: 600; font-size: 16px;">
                Track Order
            </a>
        </div>
        
        <div style="background: #f0f9ff; border-radius: 8px; padding: 16px; margin: 24px 0;">
            <p style="margin: 0; font-size: 14px; color: #0369a1;">
                <strong>💡 Tip:</strong> Bookmark this link for easy access to your order status!
            </p>
        </div>
        
        <p style="margin: 24px 0 0; font-size: 14px; color: #6b7280; text-align: center;">
            Or copy this link:
        </p>
        <p style="margin: 8px 0 0; font-size: 13px; color: {brand}; word-break: break-all; text-align: center;">
            {link}
        </p>
    "##,
        name = data.name,
        order_number = data.order_number,
        link = data.tracking_link,
        brand = BRAND_COLOR,
        brand_dark = BRAND_COLOR_DARK,
    );
    
    base_html(
        &format!("Track Order - {}", data.order_number),
        &content,
        &format!("Track your order {} anytime", data.order_number)
    )
}

pub fn guest_tracking_text(data: &GuestTrackingData) -> String {
    format!(r#"Hi {name},

Here's your tracking link for order #{order_number}:

{link}

This link is valid for 30 days. Bookmark it for easy access!

- The SoraStore Team"#,
        name = data.name,
        order_number = data.order_number,
        link = data.tracking_link
    )
}
