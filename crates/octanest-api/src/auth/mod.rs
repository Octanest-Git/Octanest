//! Auth domain: password hashing, sessions, and (later) providers / RPC.

pub mod password;

pub use password::{hash_password, hash_password_str, verify_password, PasswordError, MIN_PASSWORD_LEN};
