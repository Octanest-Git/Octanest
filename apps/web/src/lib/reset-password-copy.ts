/** Locked copy for `/reset-password` (05-UI-SPEC AUTH-12). */

export const REQUEST_SUPPORT =
  "Enter your account email. If it matches a local-password account, we’ll send a link and an 8-digit code.";

export const REDEEM_SUPPORT_CODE =
  "Enter the 8-digit code from your email, then set a new password.";

export const REDEEM_SUPPORT_TOKEN = "Choose a new password to finish resetting your account.";

export const SSO_MODE_BODY = "Password reset is managed by your identity provider.";

export const SUCCESS_HEADING = "Check your email";

export const SUCCESS_BODY =
  "If an account exists for that email, we sent password reset instructions. Check your inbox and spam folder.";

export const NETWORK_ERROR = "Can't reach Octanest. Check your connection and try again.";

export const MISMATCH = "Passwords don’t match. Fix the highlighted fields and try again.";

export const TOO_SHORT = "Password must be at least 8 characters.";

export const INVALID_EXPIRED_CODE =
  "That code is invalid or expired. Request a new reset email and try again.";

export const INVALID_EXPIRED_TOKEN =
  "That link is invalid or expired. Request a new reset email and try again.";

export const SSO_ONLY =
  "This account signs in with SSO. Reset your password with your identity provider.";

export const PASSWORD_HELPER = "At least 8 characters.";
