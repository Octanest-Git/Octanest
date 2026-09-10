//! Auth module: passwords, sessions, local + WorkOS + OIDC providers.

pub mod admin;
pub mod external;
pub mod gate;
pub mod local;
pub mod oidc;
pub mod password;
pub mod pending;
pub mod profile;
pub mod seed;
pub mod session;
pub mod verify_reset;
pub mod workos;

pub use password::{
    hash_password, hash_password_str, verify_password, PasswordError, MIN_PASSWORD_LEN,
};
pub use session::{
    clear_session_cookie, secure_cookies, AuthError, ResolvedSession, SessionService,
    SESSION_COOKIE_NAME, SESSION_IDLE, SESSION_REMEMBER,
};
