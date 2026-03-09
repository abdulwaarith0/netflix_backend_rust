use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use openssl::rand::rand_bytes;
use openssl::symm::{decrypt, encrypt, Cipher};
use std::env;

/// A self-contained crypto error — no actix types here.
/// Callers (handlers) map this to the appropriate HttpResponse.
#[derive(Debug)]
pub enum CryptoError {
    EnvVarMissing,
    InvalidKeyLength,
    Base64DecodeFailed,
    EncryptionFailed(String),
    DecryptionFailed,
    Utf8ConversionFailed,
    IvTooShort,
}

impl std::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CryptoError::EnvVarMissing => write!(f, "SECRET_KEY not set"),
            CryptoError::InvalidKeyLength => write!(f, "Secret key must be exactly 32 bytes after decoding"),
            CryptoError::Base64DecodeFailed => write!(f, "Failed to decode base64 data"),
            CryptoError::EncryptionFailed(e) => write!(f, "Encryption error: {}", e),
            CryptoError::DecryptionFailed => write!(f, "Failed to decrypt password"),
            CryptoError::Utf8ConversionFailed => write!(f, "Decrypted bytes are not valid UTF-8"),
            CryptoError::IvTooShort => write!(f, "Encrypted data is too short to contain an IV"),
        }
    }
}

// ── Key loading ───────────────────────────────────────────────────────────────

/// Reads SECRET_KEY from the environment, base64-decodes it,
/// and validates it is exactly 32 bytes (required for AES-256).
pub fn get_secret_key() -> Result<Vec<u8>, CryptoError> {
    let encoded = env::var("SECRET_KEY").map_err(|_| CryptoError::EnvVarMissing)?;

    let key = BASE64
        .decode(encoded.as_bytes())
        .map_err(|_| CryptoError::Base64DecodeFailed)?;

    if key.len() != 32 {
        return Err(CryptoError::InvalidKeyLength);
    }

    Ok(key)
}

// ── Encryption ────────────────────────────────────────────────────────────────

/// Encrypts `password` with AES-256-CBC using a random IV.
///
/// Output format (base64-encoded): `[ IV (16 bytes) | ciphertext ]`
///
/// The IV is prepended so `decrypt_password` can extract it without
/// needing a separate storage field.
pub fn encrypt_password(password: &str) -> Result<String, CryptoError> {
    let key = get_secret_key()?;
    let cipher = Cipher::aes_256_cbc();

    // Generate a cryptographically random 16-byte IV for every encryption.
    // Reusing or omitting the IV leaks information about identical plaintexts.
    let mut iv = vec![0u8; 16];
    rand_bytes(&mut iv).map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;

    let ciphertext = encrypt(cipher, &key, Some(&iv), password.as_bytes())
        .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;

    // Prepend IV to ciphertext, then base64-encode the whole thing.
    let mut payload = iv;
    payload.extend_from_slice(&ciphertext);

    Ok(BASE64.encode(&payload))
}

// ── Decryption ────────────────────────────────────────────────────────────────

/// Decrypts a password that was encrypted by `encrypt_password`.
///
/// Expects `encrypted_password` to be base64-encoded `[ IV (16 bytes) | ciphertext ]`.
pub fn decrypt_password(encrypted_password: &str, key: &[u8]) -> Result<String, CryptoError> {
    let payload = BASE64
        .decode(encrypted_password.as_bytes())
        .map_err(|_| CryptoError::Base64DecodeFailed)?;

    // The first 16 bytes are the IV stored by encrypt_password.
    if payload.len() < 16 {
        return Err(CryptoError::IvTooShort);
    }

    let (iv, ciphertext) = payload.split_at(16);

    let plaintext = decrypt(Cipher::aes_256_cbc(), key, Some(iv), ciphertext)
        .map_err(|_| CryptoError::DecryptionFailed)?;

    String::from_utf8(plaintext).map_err(|_| CryptoError::Utf8ConversionFailed)
}