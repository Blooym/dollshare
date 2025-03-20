use anyhow::{Result, bail};
use chacha20poly1305::{
    AeadCore, KeyInit,
    aead::{Aead, OsRng, generic_array::typenum::Unsigned},
};
use hex::ToHex;

type CryptoImpl = chacha20poly1305::ChaCha20Poly1305;
type CryptoNonce = chacha20poly1305::Nonce;
type CryptoNonceSize = <CryptoImpl as AeadCore>::NonceSize;

pub struct Cryptography;

impl Cryptography {
    pub fn encrypt(bytes: &[u8]) -> Result<(String, Vec<u8>)> {
        let key = CryptoImpl::generate_key(&mut OsRng);
        let nonce = CryptoImpl::generate_nonce(&mut OsRng);
        let cipher = CryptoImpl::new_from_slice(&key)?;
        let mut ciphered_bytes = match cipher.encrypt(&nonce, bytes) {
            Ok(b) => b,
            Err(err) => {
                bail!("{err:?}");
            }
        };
        ciphered_bytes.splice(..0, nonce.iter().copied());
        Ok((key.encode_hex_upper(), ciphered_bytes))
    }

    pub fn decrypt(bytes: &[u8], key: &str) -> Result<Vec<u8>> {
        let (nonce, encrypted_bytes) = bytes.split_at(CryptoNonceSize::to_usize());
        let key = hex::decode(key)?;
        let cipher = CryptoImpl::new_from_slice(&key)?;
        match cipher.decrypt(CryptoNonce::from_slice(nonce), encrypted_bytes) {
            Ok(data) => Ok(data),
            Err(err) => bail!(err),
        }
    }
}
