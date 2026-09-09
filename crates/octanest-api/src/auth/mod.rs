//! Auth module: passwords, sessions, local + WorkOS providers.

pub mod external;
pub mod local;
pub mod password;
pub mod pending;
pub mod session;
pub mod workos;

pub use password::{
    hash_password, hash_password_str, verify_password, PasswordError, MIN_PASSWORD_LEN,
};
pub use session::{
    clear_session_cookie, secure_cookies, AuthError, ResolvedSession, SessionService,
    SESSION_COOKIE_NAME, SESSION_IDLE, SESSION_REMEMBER,
};
