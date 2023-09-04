#![feature(trait_alias)]

mod crypto;
mod json;

use std::{fs, path::Path};

use anyhow::Result;
pub use crypto::{Key, KeyPair};
pub use json::{SecretsFile, Transform};

/// Returns a transform function for use with [Parser::transform] that will encrypt all eligible
/// values (that aren't already encrypted).
pub fn encrypt(secrets_file: &SecretsFile) -> Result<impl Transform> {
    let public_key = secrets_file.public_key().unwrap();
    let ephemeral_key = KeyPair::generate()?;
    let encryptor = ephemeral_key.encryptor(public_key)?;

    Ok(move |s: String| {
        // Skip encryption if this value is already an EJSON message.
        if crypto::Message::is_valid(&s) {
            return Ok(s);
        }

        encryptor.encrypt(s)
    })
}

/// Returns a [Transform] that will decrypt incoming values from the supplied secrets file. This is
/// done by creating a [KeyPair] consisting of the public key from the file and the supplied
/// private key.
pub fn decrypt(secrets_file: &SecretsFile, private_key: Key) -> Result<impl Transform> {
    let public_key = secrets_file.public_key().unwrap();
    let decryptor = KeyPair::new(public_key, private_key).decryptor();

    Ok(move |s: String| {
        if !crypto::Message::is_valid(&s) {
            // Skip decryption for values that aren't encrypted.
            return Ok(s);
        }

        decryptor.decrypt(s)
    })
}

/// Loads the private key from disk, searching for a file named as the public key defined in the
/// secrets file.
pub fn load_private_key(secrets_file: &SecretsFile, keydir: &str) -> Result<Key> {
    let public_key = secrets_file.public_key().unwrap();
    let path = Path::new(keydir).join(public_key.to_string());
    fs::read_to_string(path)?.trim().parse()
}
