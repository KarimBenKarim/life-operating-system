use argon2::{Algorithm, Argon2, Params, Version};
use rand::RngCore;
use std::fmt;
use zeroize::{Zeroize, Zeroizing};

use crate::db::errors::DatabaseError;

/// Salt length in bytes (128-bit)
pub const SALT_BYTES: usize = 16;
/// Derived key length in bytes (256-bit)
pub const KEY_BYTES: usize = 32;

/// Argon2id parameters per ADR 0022:
/// - Memory: 65,536 KiB (64 MiB)
/// - Time / Iterations: 3
/// - Parallelism: 4
pub const ARGON2_M_COST: u32 = 65536;
pub const ARGON2_T_COST: u32 = 3;
pub const ARGON2_P_COST: u32 = 4;

/// Container for a derived 256-bit database encryption key that automatically
/// zeroizes its memory on drop and prevents secret leakage in formatting.
pub struct DerivedKey {
    bytes: Zeroizing<[u8; KEY_BYTES]>,
}

impl DerivedKey {
    pub fn new(bytes: [u8; KEY_BYTES]) -> Self {
        Self {
            bytes: Zeroizing::new(bytes),
        }
    }

    /// Access raw key bytes.
    pub fn as_bytes(&self) -> &[u8; KEY_BYTES] {
        &self.bytes
    }

    /// Convert derived key bytes into a zeroizing hex string formatted for SQLCipher PRAGMA key.
    pub fn to_pragma_key_hex(&self) -> Zeroizing<String> {
        let hex_str = hex::encode(*self.bytes);
        Zeroizing::new(hex_str)
    }
}

/// Format derived key into a zeroizing PRAGMA key statement for SQLCipher:
/// `PRAGMA key = "x'<64_hex_digits>'";`.
pub fn format_pragma_key(key: &DerivedKey) -> Zeroizing<String> {
    Zeroizing::new(format!(
        "PRAGMA key = \"x'{}'\";",
        key.to_pragma_key_hex().as_str()
    ))
}

impl fmt::Debug for DerivedKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DerivedKey([REDACTED_KEY])")
    }
}

impl fmt::Display for DerivedKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED_KEY]")
    }
}

/// Generate a cryptographically secure 16-byte random salt.
pub fn generate_salt() -> [u8; SALT_BYTES] {
    let mut salt = [0u8; SALT_BYTES];
    rand::thread_rng().fill_bytes(&mut salt);
    salt
}

/// Derive a 256-bit database key from a master passcode and 16-byte salt using Argon2id.
pub fn derive_key(passcode: &str, salt: &[u8; SALT_BYTES]) -> Result<DerivedKey, DatabaseError> {
    if passcode.is_empty() {
        return Err(DatabaseError::InvalidPasscode);
    }

    let params = Params::new(ARGON2_M_COST, ARGON2_T_COST, ARGON2_P_COST, Some(KEY_BYTES))
        .map_err(|e| DatabaseError::CryptoError(format!("Invalid Argon2 parameters: {e}")))?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut key_buf = [0u8; KEY_BYTES];
    argon2
        .hash_password_into(passcode.as_bytes(), salt, &mut key_buf)
        .map_err(|e| DatabaseError::CryptoError(format!("Argon2id key derivation failed: {e}")))?;

    let derived_key = DerivedKey::new(key_buf);
    key_buf.zeroize();

    Ok(derived_key)
}
