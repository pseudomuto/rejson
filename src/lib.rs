#![doc = include_str!("README.md")]

mod crypto;
mod json;
mod kube;
mod map;

use std::path::Path;

use anyhow::Result;
pub use crypto::{Key, KeyPair};
pub use json::SecretsFile;
pub use kube::SecretsManifest;
pub use map::SecretsMap;

const NEW_LINE: &str = "\n";
const CARRIAGE_RETURN: &str = "\r";

/// Returns a transform function that compacts multiline strings into single lines with line
/// break characters. This is useful when adding something like a service account in the EJSON file
/// and having the encrypt function compact it before encryption.
pub fn compact() -> Result<impl Fn(String) -> Result<String>> {
    Ok(|s: String| {
        if s.contains(NEW_LINE) || s.contains(CARRIAGE_RETURN) {
            return Ok(s
                .trim()
                .replace(NEW_LINE, r"\n")
                .replace(CARRIAGE_RETURN, r"\r")
                .to_string());
        }

        Ok(s)
    })
}

/// Returns a transform function for use with [SecretsFile::transform] that will encrypt all eligible
/// values (that aren't already encrypted).
pub fn encrypt(secrets_file: &SecretsFile) -> Result<impl Fn(String) -> Result<String>> {
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

/// Returns a transform that will decrypt incoming values from the supplied secrets file. This is
/// done by creating a [KeyPair] consisting of the public key from the file and the supplied
/// private key.
pub fn decrypt(secrets_file: &SecretsFile, private_key: Key) -> Result<impl Fn(String) -> Result<String>> {
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
    Key::from_file(Path::new(keydir).join(public_key.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_transform() -> Result<()> {
        let cases = [
            ("\n\n\n\r\r\n", ""),
            ("some\nstring", r"some\nstring"),
            ("some\r\nstring", r"some\r\nstring"),
        ];

        let tf = compact()?;

        cases.into_iter().try_for_each(|(given, want)| -> Result<()> {
            assert_eq!(want, tf(given.to_string())?);
            Ok(())
        })
    }
}
