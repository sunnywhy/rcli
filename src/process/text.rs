use std::io::Read;
use std::path::Path;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Key, Nonce,
};
use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;

use crate::cli::TextSignFormat;
use crate::process_genpass;
use crate::utils::get_reader;

pub trait TextSign {
    // &[u8] impl Read, easy to test
    /// Sign the data from the reader and return the signature
    fn sign(&self, reader: &mut dyn Read) -> anyhow::Result<Vec<u8>>;
}

pub trait TextVerify {
    /// Verify the signature with the data from the reader and return true if the signature is valid
    fn verify(&self, reader: impl Read, signature: &[u8]) -> anyhow::Result<bool>;
}

pub trait KeyLoader {
    fn load(path: impl AsRef<Path>) -> anyhow::Result<Self>
    where
        Self: Sized;
}

pub trait KeyGenerator {
    fn generate() -> anyhow::Result<Vec<Vec<u8>>>;
}

struct Blake3 {
    key: [u8; 32],
}

struct Ed25519Signer {
    key: SigningKey,
}

struct Ed25519Verifier {
    key: VerifyingKey,
}

struct ChaCha20Poly1305Engine {
    key: Key,
    nonce: Nonce,
}

impl TextSign for Blake3 {
    fn sign(&self, reader: &mut dyn Read) -> anyhow::Result<Vec<u8>> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        Ok(blake3::keyed_hash(&self.key, &buf).as_bytes().to_vec())
    }
}

impl TextSign for Ed25519Signer {
    fn sign(&self, reader: &mut dyn Read) -> anyhow::Result<Vec<u8>> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let signature = self.key.sign(&buf);
        Ok(signature.to_bytes().to_vec())
    }
}

impl TextVerify for Blake3 {
    fn verify(&self, mut reader: impl Read, signature: &[u8]) -> anyhow::Result<bool> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let hash = blake3::keyed_hash(&self.key, &buf);
        let hash = hash.as_bytes();
        Ok(hash == signature)
    }
}

impl TextVerify for Ed25519Verifier {
    fn verify(&self, mut reader: impl Read, signature: &[u8]) -> anyhow::Result<bool> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let signature = ed25519_dalek::Signature::from_slice(signature)?;
        Ok(self.key.verify(&buf, &signature).is_ok())
    }
}

impl KeyLoader for Blake3 {
    fn load(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let key = std::fs::read(path)?;
        Self::try_new(&key)
    }
}

impl KeyLoader for Ed25519Signer {
    fn load(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let key = std::fs::read(path)?;
        Self::try_new(&key)
    }
}

impl KeyLoader for ChaCha20Poly1305Engine {
    fn load(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let key = std::fs::read(path)?;
        Self::try_new(&key)
    }
}

impl KeyGenerator for Blake3 {
    fn generate() -> anyhow::Result<Vec<Vec<u8>>> {
        let key = process_genpass(32, true, true, true, true)?;
        let key = key.as_bytes().to_vec();
        Ok(vec![key])
    }
}

impl KeyGenerator for Ed25519Signer {
    fn generate() -> anyhow::Result<Vec<Vec<u8>>> {
        let mut csprng = OsRng;
        let sk = SigningKey::generate(&mut csprng);
        let pk = sk.verifying_key();
        let sk = sk.as_bytes().to_vec();
        let pk = pk.to_bytes().to_vec();
        Ok(vec![sk, pk])
    }
}

impl KeyLoader for Ed25519Verifier {
    fn load(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let key = std::fs::read(path)?;
        Self::try_new(&key)
    }
}

impl Blake3 {
    pub fn new(key: [u8; 32]) -> Self {
        Self { key }
    }
    pub fn try_new(key: &[u8]) -> anyhow::Result<Self> {
        let key = &key[..32];
        let key: [u8; 32] = key.try_into()?;
        let signer = Blake3::new(key);
        Ok(signer)
    }
}

impl Ed25519Signer {
    pub fn new(key: SigningKey) -> Self {
        Self { key }
    }

    pub fn try_new(key: &[u8]) -> anyhow::Result<Self> {
        let key = SigningKey::from_bytes(key.try_into()?);
        Ok(Self::new(key))
    }
}

impl Ed25519Verifier {
    pub fn new(key: VerifyingKey) -> Self {
        Self { key }
    }

    pub fn try_new(key: &[u8]) -> anyhow::Result<Self> {
        let key = VerifyingKey::from_bytes(key.try_into()?)?;
        Ok(Self::new(key))
    }
}

impl ChaCha20Poly1305Engine {
    pub fn new(key: Key, nonce: Nonce) -> Self {
        Self { key, nonce }
    }

    pub fn try_new(key: &[u8]) -> anyhow::Result<Self> {
        // use the first 12 bytes as nonce
        let nonce = Nonce::from_slice(&key[..12]);
        // use the rest as key
        let key = Key::from_slice(&key[12..44]);

        Ok(Self::new(*key, *nonce))
    }

    pub fn encrypt(&self, mut reader: impl Read) -> anyhow::Result<String> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let cipher = ChaCha20Poly1305::new(&self.key);
        let ciphertext = cipher.encrypt(&self.nonce, buf.as_ref())?;
        let encoded = URL_SAFE_NO_PAD.encode(ciphertext);
        Ok(encoded)
    }

    pub fn decrypt(&self, mut reader: impl Read) -> anyhow::Result<String> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let ciphertext = URL_SAFE_NO_PAD.decode(buf)?;
        let cipher = ChaCha20Poly1305::new(&self.key);
        let plaintext = cipher.decrypt(&self.nonce, ciphertext.as_ref())?;
        let plaintext = String::from_utf8(plaintext)?;
        Ok(plaintext)
    }
}

pub fn process_text_sign(input: &str, key: &str, format: TextSignFormat) -> anyhow::Result<String> {
    let mut reader = get_reader(input)?;
    let signed = match format {
        TextSignFormat::Blake3 => {
            let signer = Blake3::load(key)?;
            signer.sign(&mut reader)?
        }
        TextSignFormat::Ed25519 => {
            let signer = Ed25519Signer::load(key)?;
            signer.sign(&mut reader)?
        }
    };
    let signed = URL_SAFE_NO_PAD.encode(signed);
    Ok(signed)
}

pub fn process_text_verify(
    input: &str,
    key: &str,
    format: TextSignFormat,
    sig: &str,
) -> anyhow::Result<bool> {
    let mut reader = get_reader(input)?;
    let sig = URL_SAFE_NO_PAD.decode(sig)?;
    let verified = match format {
        TextSignFormat::Blake3 => {
            let verifier = Blake3::load(key)?;
            verifier.verify(&mut reader, &sig)?
        }
        TextSignFormat::Ed25519 => {
            let verifier = Ed25519Verifier::load(key)?;
            verifier.verify(&mut reader, &sig)?
        }
    };
    Ok(verified)
}

pub fn process_generate_key(format: TextSignFormat) -> anyhow::Result<Vec<Vec<u8>>> {
    match format {
        TextSignFormat::Blake3 => Blake3::generate(),
        TextSignFormat::Ed25519 => Ed25519Signer::generate(),
    }
}

pub fn process_text_encrypt(input: &str, key: &str) -> anyhow::Result<String> {
    let engine = ChaCha20Poly1305Engine::load(key)?;
    let mut reader = get_reader(input)?;
    let encrypted = engine.encrypt(&mut reader)?;
    Ok(encrypted)
}

pub fn process_text_decrypt(input: &str, key: &str) -> anyhow::Result<String> {
    let engine = ChaCha20Poly1305Engine::load(key)?;
    let mut reader = get_reader(input)?;
    let decrypted = engine.decrypt(&mut reader)?;
    Ok(decrypted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blake3_sign_verify() -> anyhow::Result<()> {
        let verifier = Blake3::load("fixtures/blake3.txt")?;
        let data = b"hello, world!";
        let signature = verifier.sign(&mut &data[..])?;
        assert!(verifier.verify(&data[..], &signature)?);
        Ok(())
    }

    #[test]
    fn test_ed25519_sign_verify() -> anyhow::Result<()> {
        let signer = Ed25519Signer::load("fixtures/ed25519.sk")?;
        let verifier = Ed25519Verifier::load("fixtures/ed25519.pk")?;
        let data = b"hello, world!";
        let signature = signer.sign(&mut &data[..])?;
        assert!(verifier.verify(&data[..], &signature)?);
        Ok(())
    }

    #[test]
    fn test_chacha20poly1305_encrypt_decrypt() -> anyhow::Result<()> {
        let engine = ChaCha20Poly1305Engine::load("fixtures/chacha20poly1305.txt")?;
        let data = b"hello, world!";
        let encrypted = engine.encrypt(&mut &data[..])?;
        let decrypted = engine.decrypt(&mut encrypted.as_bytes())?;
        assert_eq!(data, decrypted.as_bytes());
        Ok(())
    }
}
