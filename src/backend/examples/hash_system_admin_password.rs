//! Generate an Argon2 PHC string for SYSTEM_ADMIN_PASSWORD_HASH (same defaults as user registration).
//!
//! Usage: cargo run --example hash_system_admin_password -- 'your-secure-password'

use argon2::password_hash::{PasswordHasher, SaltString};
use argon2::Argon2;
use password_hash::rand_core::OsRng;

fn main() {
    let password = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage: cargo run --example hash_system_admin_password -- '<password>'");
        std::process::exit(1);
    });
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .expect("hash failed")
        .to_string();
    println!("{hash}");
}
