use aes_gcm::{AeadInPlace, Aes256Gcm, KeyInit};
use argon2::{Argon2, Params, Version};
use sha2::{Digest, Sha256};
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

const MIN_SALT_LEN: usize = 16;

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum CryptoError {
    #[error("aes decryption failed")]
    AesDecrypt,

    #[error("aes encryption failed")]
    AesEncrypt,

    #[error("invalid aes key length")]
    AesKeyLength,

    #[error("invalid argon2 params config")]
    Argon2Params,

    #[error("argon2 hash failed")]
    Argon2Hash,

    #[error("salt length too short")]
    SaltLength,
}

#[derive(Debug, PartialEq, Eq, Zeroize, ZeroizeOnDrop)]
pub struct DerivedKeys {
    pub enc_key: [u8; 32],
    pub auth_key: [u8; 32],
}

impl DerivedKeys {
    pub fn derive(password: &str, salt: &[u8]) -> Result<Self, CryptoError> {
        if salt.len() < MIN_SALT_LEN {
            return Err(CryptoError::SaltLength);
        }

        let params = Params::new(32 * 1024, 3, 1, None).map_err(|_| CryptoError::Argon2Params)?;
        let mut derived = Zeroizing::new([0u8; 64]);
        Argon2::new(argon2::Algorithm::Argon2id, Version::V0x13, params)
            .hash_password_into(password.as_bytes(), salt, derived.as_mut_slice())
            .map_err(|_| CryptoError::Argon2Hash)?;

        let mut enc_key = [0u8; 32];
        let mut auth_key = [0u8; 32];
        enc_key.copy_from_slice(&derived[0..32]);
        auth_key.copy_from_slice(&derived[32..64]);
        Ok(Self { enc_key, auth_key })
    }

    #[inline]
    #[must_use]
    pub fn auth_hash(&self) -> [u8; 32] {
        hash_data(&self.auth_key)
    }
}

pub fn aes_encrypt(plaintext: &[u8], nonce: &[u8], enc_key: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let aes = Aes256Gcm::new_from_slice(enc_key).map_err(|_| CryptoError::AesKeyLength)?;
    let mut ciphertext = plaintext.to_vec();
    let tag = aes
        .encrypt_in_place_detached(nonce.into(), &[], &mut ciphertext)
        .map_err(|_| CryptoError::AesEncrypt)?;

    ciphertext.extend_from_slice(&tag);
    Ok(ciphertext)
}

pub fn aes_decrypt(ciphertext: &[u8], nonce: &[u8], enc_key: &[u8]) -> Result<Vec<u8>, CryptoError> {
    if ciphertext.len() < 16 {
        return Err(CryptoError::AesDecrypt);
    }

    let aes = Aes256Gcm::new_from_slice(enc_key).map_err(|_| CryptoError::AesKeyLength)?;
    let (ciphertext, tag) = ciphertext.split_at(ciphertext.len() - 16);
    let mut buffer = ciphertext.to_vec();
    aes.decrypt_in_place_detached(nonce.into(), &[], &mut buffer, tag.into())
        .map_err(|_| CryptoError::AesDecrypt)?;
    Ok(buffer)
}

#[must_use]
pub fn hash_data(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_keys() {
        let password = "password";
        let salt = [0u8; 16];
        let keys = DerivedKeys::derive(password, &salt).unwrap();
        assert_ne!(keys.enc_key, [0u8; 32]);
        assert_ne!(keys.auth_key, [0u8; 32]);
    }

    #[test]
    fn test_derive_small_salt() {
        let password = "password";
        let salt = [0u8; 15];
        assert!(DerivedKeys::derive(password, &salt).is_err());
    }

    #[test]
    fn test_hash_auth_key() {
        let key = [0u8; 32];
        let auth_key = hash_data(&key);
        assert_ne!(auth_key, key);
    }

    #[test]
    fn test_aes() {
        let plaintext = "plaintext";
        let nonce = [0u8; 12];
        let enc_key = [0u8; 32];
        let enc = aes_encrypt(plaintext.as_bytes(), &nonce, &enc_key).unwrap();
        assert_ne!(enc, plaintext.as_bytes());

        let dec = aes_decrypt(&enc, &nonce, &enc_key).unwrap();
        assert_eq!(dec, plaintext.as_bytes());

        let dec = aes_decrypt(&enc, &nonce, &[1u8; 32]).unwrap_err();
        assert_eq!(dec, CryptoError::AesDecrypt);
    }
}
