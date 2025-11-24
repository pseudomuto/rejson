use std::{fmt, fs, path::Path, str::FromStr};

use anyhow::{Result, anyhow};
use nacl::public_box;
use rand::RngCore;

use super::{decryptor::Decryptor, encryptor::Encryptor};

// These correspond to the NACL Box constants defined here:
// https://docs.rs/nacl/latest/nacl/public_box/index.html
const KEY_SIZE: usize = 32;
const NONCE_SIZE: usize = 24;

/// A newtype representing an encryption key (32-byte array)
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Key(pub(crate) [u8; KEY_SIZE]);

impl Key {
    /// Reads in a [Key] from the supplied path.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        fs::read_to_string(path)?.trim().parse()
    }

    /// Return a [Key] consisting of all _v_ bytes.
    #[cfg(test)]
    pub fn all(v: u8) -> Self {
        Self([v; KEY_SIZE])
    }

    /// Generate a random [Key].
    pub fn random() -> Self {
        let mut bytes = Self::default().0;
        rand::rng().fill_bytes(&mut bytes);
        Self(bytes)
    }
}

impl fmt::Display for Key {
    /// Writes the hex-encoded representation of this key.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0.map(|b| format!("{:02x}", b)).join(""))
    }
}

impl FromStr for Key {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        if s.len() != 2 * KEY_SIZE {
            return Err(anyhow!("InvalidKey (bad length)"));
        }

        Ok(Self(bytes_from_hex(s).as_slice().try_into()?))
    }
}

fn bytes_from_hex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| {
            s.get(i..i + 2)
                .and_then(|sub| u8::from_str_radix(sub, 16).ok())
                .unwrap()
        })
        .collect()
}

/// A newtype representing a nonce (24-byte array)
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Nonce(pub(crate) [u8; NONCE_SIZE]);

impl Nonce {
    /// Return a [Nonce] consisting of all _v_ bytes.
    #[cfg(test)]
    pub fn all(v: u8) -> Self {
        Self([v; NONCE_SIZE])
    }

    /// Generate a random [Nonce].
    pub fn random() -> Self {
        let mut bytes = Self::default().0;
        rand::rng().fill_bytes(&mut bytes);
        Self(bytes)
    }
}

impl fmt::Display for Nonce {
    /// Writes the hex-encoded representation of this Nonce.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0.map(|b| format!("{:02x}", b)).join(""))
    }
}

/// A struct representing a Curve25519 key pair (public and private).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyPair {
    pub(crate) public: Key,
    pub(crate) private: Key,
}

impl KeyPair {
    /// Creates a new [KeyPair] with the supplied public and private keys.
    pub fn new(public: Key, private: Key) -> Self {
        Self { public, private }
    }

    /// Generates a new [KeyPair] using a randomly generated private key.
    pub fn generate() -> Result<Self> {
        // Generate a random private key. Then using the private key, generate a corresponding public key.
        let priv_key = Key::random();
        let pub_key = public_box::generate_pubkey(&priv_key.0).map_err(|e| anyhow!(e.message))?;
        Ok(Self::new(Key(pub_key.as_slice().try_into()?), priv_key))
    }

    /// Returns the hex encoded public key.
    pub fn public_key(&self) -> String {
        self.public.to_string()
    }

    /// Returns the hex encoded private key.
    pub fn private_key(&self) -> String {
        self.private.to_string()
    }

    /// Creates a new [Encryptor] using the supplied emphermal key and this [KeyPair].
    pub fn encryptor(&self, peer_public: Key) -> Result<Encryptor> {
        Encryptor::create(self.clone(), peer_public)
    }

    /// Createa s new [Decryptor] using this key pair.
    pub fn decryptor(&self) -> Decryptor {
        Decryptor::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_from_file() {
        let key_path = Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap())
            .join("examples")
            .join("data")
            .join("2549b26efec29cf60e473797f5dda5f41d99460cf1c32f34f1c0247d9bd7ff5b");

        assert!(Key::from_file(key_path).is_ok());
    }

    #[test]
    fn key_serde() {
        let key = Key::random();
        let key_str = key.to_string();
        assert_eq!(2 * KEY_SIZE, key_str.len());

        let parsed = key_str.parse().unwrap();
        assert_eq!(key, parsed);
    }

    #[test]
    fn new() {
        let pub_key = Key::all(1);
        let priv_key = Key::all(2);

        let pair = KeyPair::new(pub_key.clone(), priv_key.clone());
        assert_eq!(pub_key, pair.public);
        assert_eq!(priv_key, pair.private);
    }

    #[test]
    fn generate() {
        let pair = KeyPair::generate().unwrap();
        assert_ne!(pair.public_key(), pair.private_key());
        assert!(!pair.public_key().contains("00000"));
        assert!(!pair.private_key().contains("00000"));
        assert_eq!(2 * KEY_SIZE, pair.public_key().len());
        assert_eq!(2 * KEY_SIZE, pair.private_key().len());
    }
}
