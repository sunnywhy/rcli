use std::io::Read;
use std::path::Path;

use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use base64::Engine;
use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};

use crate::cli::TextSignFormat;
use crate::utils::get_reader;

trait TextSign {
    // &[u8] impl Read, easy to test
    /// Sign the data from the reader and return the signature
    fn sign(&self, reader: &mut dyn Read) -> anyhow::Result<Vec<u8>>;
}

trait TextVerify {
    /// Verify the signature with the data from the reader and return true if the signature is valid
    fn verify(&self, reader: impl Read, signature: &[u8]) -> anyhow::Result<bool>;
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

    pub fn load(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let key = std::fs::read(path)?;
        Self::try_new(&key)
    }
}

pub fn process_text_sign(input: &str, key: &str, format: TextSignFormat) -> anyhow::Result<()> {
    let mut reader = get_reader(input)?;
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;
    let signed = match format {
        TextSignFormat::Blake3 => {
            let signer = Blake3::load(key)?;
            signer.sign(&mut reader)?
        }
        TextSignFormat::Ed25519 => todo!(),
    };
    let signed = BASE64_URL_SAFE_NO_PAD.encode(signed);
    println!("{}", signed);
    Ok(())
}

pub fn process_text_verify(
    input: &str,
    key: &str,
    format: TextSignFormat,
    sig: &str,
) -> anyhow::Result<()> {
    let mut reader = get_reader(input)?;
    let key = std::fs::read(key)?;
    let sig = BASE64_URL_SAFE_NO_PAD.decode(sig)?;
    let verified = match format {
        TextSignFormat::Blake3 => {
            let key: [u8; 32] = key.try_into().expect("Invalid key length");
            let verifier = Blake3 { key };
            verifier.verify(&mut reader, &sig)?
        }
        TextSignFormat::Ed25519 => todo!(),
    };
    println!("{}", verified);
    Ok(())
}
