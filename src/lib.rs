mod crypto;
mod json;

use anyhow::Result;
pub use crypto::KeyPair;
pub use json::SecretsFile;

/// Returns a transform function for use with [Parser::transform] that will encrypt all eligible
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
