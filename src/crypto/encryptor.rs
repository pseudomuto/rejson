use anyhow::Result;
use nacl::{public_box, secret_box};

use super::{
    keys::{Key, KeyPair, Nonce},
    message::Message,
};

/// A struct for managing the encryption of strings into serialized messages for storing in EJSON
/// files.
pub struct Encryptor {
    /// The keypair used for encryption/decryption.
    keys: KeyPair,
    /// The (typically DH) shared/stream key.
    shared_key: Key,
}

impl Encryptor {
    /// Creates a new [Encryptor] using the supplied key paid, peer public key, and shared key.
    pub fn new(keys: KeyPair, shared_key: Key) -> Self {
        Self { keys, shared_key }
    }

    /// Creates a new [Encryptor] from the given [KeyPair] and peer public key. A shared key is
    /// calculated from these values and used to construct the [Encryptor].
    pub fn create(keys: KeyPair, peer_public: Key) -> Result<Self> {
        let shared_key = Key(
            public_box::calc_dhshared_key(&peer_public.0, &keys.private.0)
                .map_err(|e| anyhow::anyhow!(e.message))?
                .as_slice()
                .try_into()?,
        );

        Ok(Self { keys, shared_key })
    }

    /// Encrypts the given string returning the value to be stored in the EJSON file.
    pub fn encrypt<S: Into<String>>(&self, plaintext: S) -> Result<String> {
        let nonce = Nonce::random();
        let value = secret_box::pack(plaintext.into().as_bytes(), &nonce.0, &self.shared_key.0)
            .map_err(|e| anyhow::anyhow!(e.message))?;

        // Box the message and return in EJSON format
        Ok(Message {
            version: 1,
            key: self.keys.public.clone(),
            nonce,
            value,
        }
        .to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let keys = KeyPair::generate().unwrap();
        let peer_key = Key::random();
        let encryptor = Encryptor::create(keys.clone(), peer_key.clone()).unwrap();

        assert_eq!(keys, encryptor.keys);
        assert_ne!(Key::default(), encryptor.shared_key);
    }

    #[test]
    fn encrypt() {
        let encryptor = Encryptor::create(KeyPair::generate().unwrap(), Key::random()).unwrap();
        assert!(Message::is_valid(&encryptor.encrypt("ssshhhhh").unwrap()));
    }
}
