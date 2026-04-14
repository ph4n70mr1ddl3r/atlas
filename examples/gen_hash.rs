use argon2::password_hash::{SaltString, PasswordHasher};
use argon2::Argon2;
use rand::RngCore;

fn main() {
    let mut salt_bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut salt_bytes);
    let salt = SaltString::encode_b64(&salt_bytes).unwrap();
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(b"admin123", &salt).unwrap();
    println!("{}", hash);
}
