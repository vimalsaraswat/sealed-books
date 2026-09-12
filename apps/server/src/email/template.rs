//! Clean, responsive email templates for Sealed Books transactional notifications.

/// Generates an enterprise-branded HTML email body for a 6-digit OTP verification code.
pub fn otp_email_html(code: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Sealed Books Verification Code</title>
</head>
<body style="margin: 0; padding: 0; background-color: #f8fafc; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;">
  <table width="100%" border="0" cellpadding="0" cellspacing="0" style="background-color: #f8fafc; padding: 40px 20px;">
    <tr>
      <td align="center">
        <table width="100%" border="0" cellpadding="0" cellspacing="0" style="max-width: 480px; background-color: #ffffff; border: 1px solid #e2e8f0; border-radius: 12px; overflow: hidden; box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.05);">
          <!-- Header -->
          <tr>
            <td align="center" style="padding: 32px 32px 16px 32px; background-color: #0f172a;">
              <table border="0" cellpadding="0" cellspacing="0">
                <tr>
                  <td align="center" style="padding-bottom: 8px;">
                    <div style="display: inline-block; width: 44px; height: 44px; line-height: 44px; border-radius: 10px; background-color: #4f46e5; color: #ffffff; font-size: 20px; font-weight: bold; text-align: center;">
                      &#x1F512;
                    </div>
                  </td>
                </tr>
                <tr>
                  <td align="center">
                    <h1 style="margin: 0; font-size: 20px; font-weight: 700; color: #ffffff; letter-spacing: -0.025em;">
                      Sealed Books
                    </h1>
                    <p style="margin: 4px 0 0 0; font-size: 11px; color: #94a3b8; font-family: monospace;">
                      Cryptographic Audit Ledger & Consensus Vault
                    </p>
                  </td>
                </tr>
              </table>
            </td>
          </tr>

          <!-- Main Content -->
          <tr>
            <td style="padding: 32px;">
              <h2 style="margin: 0 0 12px 0; font-size: 16px; font-weight: 600; color: #0f172a;">
                Your Verification Security Code
              </h2>
              <p style="margin: 0 0 24px 0; font-size: 13px; line-height: 1.5; color: #475569;">
                Use the one-time code below to authenticate into your secure ledger vault. This code is valid for <strong>15 minutes</strong>.
              </p>

              <!-- Code Box -->
              <div style="background-color: #f1f5f9; border: 1px solid #cbd5e1; border-radius: 8px; padding: 18px; text-align: center; margin-bottom: 24px;">
                <span style="font-family: 'Courier New', Courier, monospace; font-size: 32px; font-weight: 800; letter-spacing: 0.25em; color: #1e1b4b; display: inline-block;">
                  {}
                </span>
              </div>

              <p style="margin: 0; font-size: 12px; line-height: 1.5; color: #64748b;">
                If you did not request this verification code, you can safely ignore this email. No access will be granted without this code.
              </p>
            </td>
          </tr>

          <!-- Footer -->
          <tr>
            <td style="padding: 20px 32px; background-color: #f8fafc; border-top: 1px solid #e2e8f0; text-align: center;">
              <p style="margin: 0; font-size: 11px; color: #94a3b8; font-family: monospace;">
                Secured by Hedera Consensus Service &bull; Topic 0.0.10462941
              </p>
            </td>
          </tr>
        </table>
      </td>
    </tr>
  </table>
</body>
</html>"#,
        code
    )
}

/// Generates a plaintext fallback body for email clients that do not render HTML.
pub fn otp_email_text(code: &str) -> String {
    format!(
        "Your Sealed Books verification security code is: {}\n\nThis code will expire in 15 minutes.\n\nIf you did not request this code, you can safely ignore this message.",
        code
    )
}
