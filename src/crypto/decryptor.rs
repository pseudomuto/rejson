use anyhow::Result;
use nacl::public_box;

use super::{keys::KeyPair, message::Message};

/// A struct for managing the decryption of serialized messages into their original plain text
/// strings.
pub struct Decryptor {
    keys: KeyPair,
}

impl Decryptor {
    /// Creates a new [Decryptor] for the given [KeyPair].
    pub fn new(keys: KeyPair) -> Self {
        Self { keys }
    }

    /// Decrypts the given ciphertext into the original plaintext. The ciphertext is expected to be
    /// a value previously encrypted by EJSON (serialized boxed message).
    ///
    /// NB: Unlike encryption, decryption does not required a shared key.
    pub fn decrypt<S: AsRef<str>>(&self, ciphertext: S) -> Result<String> {
        let message: Message = ciphertext.as_ref().parse()?;
        let plaintext = public_box::open(
            message.value.as_slice(),
            &message.nonce.0,
            &message.key.0,
            &self.keys.private.0,
        )
        .map_err(|e| anyhow::anyhow!(e.message))?;

        Ok(String::from_utf8(plaintext)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decrypt() {
        let durable_key = KeyPair::generate().unwrap();
        let emphemeral_key = KeyPair::generate().unwrap();

        let encryptor = emphemeral_key.encryptor(durable_key.public.clone()).unwrap();
        let decryptor = durable_key.decryptor();

        let plaintext = "My super secret value";
        let ciphertext = encryptor.encrypt(plaintext).unwrap();
        assert_eq!(plaintext, decryptor.decrypt(ciphertext).unwrap());
    }
}
