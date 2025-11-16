use serde::Serialize;

/// Email template data for magic link
#[derive(Serialize)]
pub struct MagicLinkTemplateData {
    pub email: String,
    pub magic_link: String,
    pub expiry_minutes: i64,
}

/// Email template data for TOTP enrollment
#[derive(Serialize)]
pub struct TotpEnrollTemplateData {
    pub email: String,
    pub secret: String,
    pub qr_code_url: String,
}

/// Email template renderer
pub struct EmailTemplates;

impl EmailTemplates {
    /// Render magic link email
    pub fn magic_link(email: &str, token: &str, base_url: &str, expiry_seconds: i64) -> (String, String) {
        let magic_link = format!("{}?token={}", base_url, token);
        let expiry_minutes = expiry_seconds / 60;

        let subject = "Your Login Link".to_string();

        let text_body = format!(
            r#"Hi,

Click the link below to sign in to your account:

{}

This link will expire in {} minutes.

If you didn't request this link, you can safely ignore this email.

Thanks,
The Passwordless Auth Team"#,
            magic_link, expiry_minutes
        );

        let html_body = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Your Login Link</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
            line-height: 1.6;
            color: #333;
            max-width: 600px;
            margin: 0 auto;
            padding: 20px;
        }}
        .container {{
            background-color: #f9f9f9;
            border-radius: 8px;
            padding: 30px;
            border: 1px solid #e0e0e0;
        }}
        .button {{
            display: inline-block;
            padding: 12px 24px;
            background-color: #007bff;
            color: white;
            text-decoration: none;
            border-radius: 4px;
            margin: 20px 0;
        }}
        .footer {{
            margin-top: 30px;
            padding-top: 20px;
            border-top: 1px solid #e0e0e0;
            font-size: 12px;
            color: #666;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h2>Sign in to your account</h2>
        <p>Hi {},</p>
        <p>Click the button below to sign in to your account:</p>
        <a href="{}" class="button">Sign In</a>
        <p>Or copy and paste this link into your browser:</p>
        <p style="word-break: break-all; font-size: 12px; color: #666;">{}</p>
        <p><strong>This link will expire in {} minutes.</strong></p>
        <p>If you didn't request this link, you can safely ignore this email.</p>
        <div class="footer">
            <p>Thanks,<br>The Passwordless Auth Team</p>
        </div>
    </div>
</body>
</html>"#,
            email, magic_link, magic_link, expiry_minutes
        );

        (subject, format!("{}\n\n---HTML---\n\n{}", text_body, html_body))
    }

    /// Render TOTP enrollment email
    pub fn totp_enrollment(email: &str, secret: &str, otpauth_url: &str) -> (String, String) {
        let subject = "Two-Factor Authentication Enabled".to_string();

        let text_body = format!(
            r#"Hi,

Two-factor authentication (TOTP) has been enabled for your account.

Your secret key is: {}

You can also scan this URL in your authenticator app:
{}

Please store this secret key securely. You'll need it to sign in to your account.

Thanks,
The Passwordless Auth Team"#,
            secret, otpauth_url
        );

        let html_body = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Two-Factor Authentication Enabled</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
            line-height: 1.6;
            color: #333;
            max-width: 600px;
            margin: 0 auto;
            padding: 20px;
        }}
        .container {{
            background-color: #f9f9f9;
            border-radius: 8px;
            padding: 30px;
            border: 1px solid #e0e0e0;
        }}
        .secret {{
            background-color: #f0f0f0;
            padding: 15px;
            border-radius: 4px;
            font-family: monospace;
            word-break: break-all;
            margin: 20px 0;
        }}
        .footer {{
            margin-top: 30px;
            padding-top: 20px;
            border-top: 1px solid #e0e0e0;
            font-size: 12px;
            color: #666;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h2>Two-Factor Authentication Enabled</h2>
        <p>Hi {},</p>
        <p>Two-factor authentication (TOTP) has been enabled for your account.</p>
        <p><strong>Your secret key:</strong></p>
        <div class="secret">{}</div>
        <p>Scan this in your authenticator app:</p>
        <div class="secret" style="font-size: 10px;">{}</div>
        <p><strong>Important:</strong> Please store this secret key securely. You'll need it to sign in to your account.</p>
        <div class="footer">
            <p>Thanks,<br>The Passwordless Auth Team</p>
        </div>
    </div>
</body>
</html>"#,
            email, secret, otpauth_url
        );

        (subject, format!("{}\n\n---HTML---\n\n{}", text_body, html_body))
    }

    /// Render session revocation notification
    pub fn session_revoked(email: &str) -> (String, String) {
        let subject = "Your session has been revoked".to_string();

        let text_body = format!(
            r#"Hi {},

A session for your account has been revoked. If this wasn't you, please contact support immediately.

Thanks,
The Passwordless Auth Team"#,
            email
        );

        let html_body = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Session Revoked</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
            line-height: 1.6;
            color: #333;
            max-width: 600px;
            margin: 0 auto;
            padding: 20px;
        }}
        .container {{
            background-color: #fff3cd;
            border-radius: 8px;
            padding: 30px;
            border: 1px solid #ffc107;
        }}
        .footer {{
            margin-top: 30px;
            padding-top: 20px;
            border-top: 1px solid #ffc107;
            font-size: 12px;
            color: #666;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h2>⚠️ Session Revoked</h2>
        <p>Hi {},</p>
        <p>A session for your account has been revoked.</p>
        <p><strong>If this wasn't you, please contact support immediately.</strong></p>
        <div class="footer">
            <p>Thanks,<br>The Passwordless Auth Team</p>
        </div>
    </div>
</body>
</html>"#,
            email
        );

        (subject, format!("{}\n\n---HTML---\n\n{}", text_body, html_body))
    }
}
