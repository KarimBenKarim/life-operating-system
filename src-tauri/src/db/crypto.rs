use crate::db::errors::{DbError, Result};
use argon2::{password_hash::SaltString, Algorithm, Argon2, Params, Version};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Secret key wrapper that zeroizes its content when dropped.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct DerivedKey {
    pub key: [u8; 32],
}

impl DerivedKey {
    /// Formats the 32-byte binary key into SQLCipher PRAGMA key syntax.
    pub fn to_pragma_hex(&self) -> String {
        format!("\"x'{}'\"", hex::encode(self.key))
    }
}

/// Derives a 256-bit (32-byte) key from the user passcode and salt using Argon2id.
/// Argon2id parameters strictly conform to ADR 0022 and security architecture:
/// m=65536 (64MB memory), t=3 (3 iterations), p=4 (4 parallelism threads).
pub fn derive_key_argon2id(passcode: &str, salt: &[u8]) -> Result<DerivedKey> {
    let params = Params::new(65536, 3, 4, Some(32))
        .map_err(|e| DbError::KeyDerivation(format!("Invalid Argon2id parameters: {e}")))?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut output_key = [0u8; 32];
    argon2
        .hash_password_into(passcode.as_bytes(), salt, &mut output_key)
        .map_err(|e| DbError::KeyDerivation(format!("Argon2id hash derivation failed: {e}")))?;

    Ok(DerivedKey { key: output_key })
}

/// Generates a new 16-byte random salt for Argon2id key derivation.
pub fn generate_salt() -> [u8; 16] {
    let salt = SaltString::generate(&mut rand_core::OsRng);
    let mut salt_bytes = [0u8; 16];
    let bytes = salt.as_str().as_bytes();
    let len = bytes.len().min(16);
    salt_bytes[..len].copy_from_slice(&bytes[..len]);
    salt_bytes
}
