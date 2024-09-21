use actix_web::HttpResponse;
use base64::decode;
use openssl::symm::{decrypt, Cipher, encrypt};
use std::env;
use base64::encode as base64_encode;


pub fn encrypt_password(password: &str) -> Result<String, String> {
    let cipher = Cipher::aes_256_cbc();
    let secret_key = env::var("SECRET_KEY").map_err(|_| "SECRET_KEY not set".to_string())?;
    let decoded_key = base64::decode(&secret_key).map_err(|_| "Failed to decode SECRET_KEY".to_string())?;

    if decoded_key.len() != 32 {
        return Err("Secret key must be exactly 32 bytes after decoding.".to_string());
    }

    let encrypted_data = encrypt(cipher, &decoded_key, None, password.as_bytes())
        .map_err(|e| format!("Encryption error: {:?}", e))?;

    Ok(base64_encode(encrypted_data))
}

pub fn get_secret_key() -> Result<Vec<u8>, HttpResponse> {
    let encoded_key = env::var("SECRET_KEY").map_err(|_| HttpResponse::InternalServerError().body("SECRET_KEY not set"))?;
    let key = decode(&encoded_key).map_err(|_| HttpResponse::InternalServerError().body("Failed to decode SECRET_KEY"))?;
    if key.len() == 32 {
        Ok(key)
    } else {
        Err(HttpResponse::InternalServerError().body("Invalid key length"))
    }
}

pub fn decrypt_password(encrypted_password: &str, secret_key: &[u8]) -> Result<String, HttpResponse> {
    let encrypted_data = decode(encrypted_password).unwrap_or_default();
    let decrypted_data = decrypt(Cipher::aes_256_cbc(), secret_key, None, &encrypted_data)
        .map_err(|_| HttpResponse::InternalServerError().body("Failed to decrypt password"))?;
    String::from_utf8(decrypted_data).map_err(|_| HttpResponse::InternalServerError().body("Failed to convert password from bytes"))
}
