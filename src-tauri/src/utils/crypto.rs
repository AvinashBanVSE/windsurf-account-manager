use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use anyhow::Result;
use base64::{engine::general_purpose::STANDARD, Engine};
use keyring::Entry;
use rand::{RngCore, thread_rng};

const APP_NAME: &str = "WindsurfAccountManager";
const KEY_NAME: &str = "MasterKey";

pub struct CryptoService {
    cipher: Aes256Gcm,
}

impl CryptoService {
    pub fn new() -> Result<Self> {
        let key = Self::get_or_create_key()?;
        let key_bytes = STANDARD.decode(&key)?;
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);
        
        Ok(Self { cipher })
    }

    fn get_or_create_key() -> Result<String> {
        let entry = Entry::new(APP_NAME, KEY_NAME)?;
        
        match entry.get_password() {
            Ok(key) => Ok(key),
            Err(_) => {
                // Generate new key
                let mut key = vec![0u8; 32];
                thread_rng().fill_bytes(&mut key);
                let key_base64 = STANDARD.encode(&key);
                
                // Save to system keychain
                entry.set_password(&key_base64)?;
                
                Ok(key_base64)
            }
        }
    }

    pub fn encrypt(&self, plaintext: &str) -> Result<String> {
        // Generate random nonce (96-bit)
        let mut nonce_bytes = [0u8; 12];
        thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        // Encrypt
        let ciphertext = self.cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;
        
        // Combine nonce and ciphertext
        let mut combined = Vec::new();
        combined.extend_from_slice(&nonce_bytes);
        combined.extend_from_slice(&ciphertext);
        
        // Return base64 encoding
        Ok(STANDARD.encode(&combined))
    }

    pub fn decrypt(&self, ciphertext: &str) -> Result<String> {
        // Decode base64
        let combined = STANDARD.decode(ciphertext)?;
        
        // Check minimum length
        if combined.len() < 12 {
            return Err(anyhow::anyhow!("Invalid ciphertext"));
        }
        
        // Separate nonce and ciphertext
        let (nonce_bytes, ciphertext) = combined.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        // Decrypt
        let plaintext = self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;
        
        // Convert to string
        String::from_utf8(plaintext)
            .map_err(|e| anyhow::anyhow!("Invalid UTF-8: {}", e))
    }

    pub fn set_master_password(&self, _password: &str) -> Result<()> {
        // Master password feature can be implemented here, derive key from password
        // Not implemented for now, using system keychain is sufficient
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let service = CryptoService::new().unwrap();
        let plaintext = "Hello, World!";
        
        let encrypted = service.encrypt(plaintext).unwrap();
        let decrypted = service.decrypt(&encrypted).unwrap();
        
        assert_eq!(plaintext, decrypted);
    }
}
