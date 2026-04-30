use aes_gcm::{
    aead::{Aead, KeyInit, rand_core::RngCore},
    Aes256Gcm, Nonce,
};
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose};

pub struct Crypto {
    cipher: Aes256Gcm,
}

impl Crypto {
    /// Create a new Crypto instance from a passphrase
    /// The passphrase is hashed with SHA-256 to create a 256-bit key
    pub fn new(passphrase: &str) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(passphrase.as_bytes());
        let key_bytes = hasher.finalize();
        
        let cipher = Aes256Gcm::new_from_slice(&key_bytes)
            .expect("Failed to create cipher");
        
        Self { cipher }
    }
    
    /// Encrypt a message
    /// Returns base64-encoded: nonce(12 bytes) + ciphertext
    pub fn encrypt(&self, plaintext: &str) -> Result<String, String> {
        // Generate a random 96-bit nonce
        let mut nonce_bytes = [0u8; 12];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        // Encrypt the plaintext
        let ciphertext = self.cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        // Combine nonce + ciphertext
        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&ciphertext);
        
        // Encode as base64
        Ok(general_purpose::STANDARD.encode(&result))
    }
    
    /// Decrypt a message
    /// Expects base64-encoded: nonce(12 bytes) + ciphertext
    pub fn decrypt(&self, encrypted: &str) -> Result<String, String> {
        // Decode from base64
        let data = general_purpose::STANDARD.decode(encrypted)
            .map_err(|e| format!("Base64 decode failed: {}", e))?;
        
        if data.len() < 12 {
            return Err("Invalid encrypted data: too short".to_string());
        }
        
        // Split nonce and ciphertext
        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        // Decrypt
        let plaintext = self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        // Convert to string
        String::from_utf8(plaintext)
            .map_err(|e| format!("UTF-8 decode failed: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_encrypt_decrypt() {
        let crypto = Crypto::new("test_passphrase");
        let message = "Hello, World!";
        
        let encrypted = crypto.encrypt(message).unwrap();
        let decrypted = crypto.decrypt(&encrypted).unwrap();
        
        assert_eq!(message, decrypted);
    }
    
    #[test]
    fn test_different_keys() {
        let crypto1 = Crypto::new("passphrase1");
        let crypto2 = Crypto::new("passphrase2");
        
        let message = "Secret message";
        let encrypted = crypto1.encrypt(message).unwrap();
        
        // Should fail with different key
        assert!(crypto2.decrypt(&encrypted).is_err());
    }
}
