use std::fs;

use anyhow::Result;
use assert_cmd::Command;

const PUB_KEY: &str = "b595226c62427adbfc4a809cd7577488a6d402b2f930e1d603164ae3191a616e";
const PRIV_KEY: &str = "88649a9e83f8f1984ad35ac8e8e86529aab518572c0341f46d1e0bc97f676f2b";

#[test]
fn decrypt() -> Result<()> {
    let key_file = assert_fs::NamedTempFile::new(PUB_KEY)?;
    fs::write(key_file.path(), PRIV_KEY)?;

    let file = assert_fs::NamedTempFile::new("secrets.ejson")?;
    fs::write(
        file.path(),
        serde_json::json!({
            "_public_key": PUB_KEY,
            "some":"EJ[1:l6yw664nxaddSXGiWUZfuVeoUSpTFHzqAyCpfF8Awxc=:xOfucLDkACGlPCyJ6QViggEidVswUlsH:B/f3DJMkdZHF+Wu9F6XUFwuTmxyfBA==]"
        })
        .to_string(),
    )?;

    Command::cargo_bin("rejson")?
        .arg("decrypt")
        .arg(file.path())
        .arg("--keydir")
        .arg(key_file.parent().unwrap())
        .assert()
        .success()
        .stdout(predicates::str::contains(r#""some": "secret""#));

    Ok(())
}

#[test]
fn decrypt_ejson_keydir() -> Result<()> {
    let key_file = assert_fs::NamedTempFile::new(PUB_KEY)?;
    fs::write(key_file.path(), PRIV_KEY)?;

    let file = assert_fs::NamedTempFile::new("secrets.ejson")?;
    fs::write(
        file.path(),
        serde_json::json!({
            "_public_key": PUB_KEY,
            "some":"EJ[1:l6yw664nxaddSXGiWUZfuVeoUSpTFHzqAyCpfF8Awxc=:xOfucLDkACGlPCyJ6QViggEidVswUlsH:B/f3DJMkdZHF+Wu9F6XUFwuTmxyfBA==]"
        })
        .to_string(),
    )?;

    Command::cargo_bin("rejson")?
        .env("EJSON_KEYDIR", key_file.parent().unwrap())
        .arg("decrypt")
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicates::str::contains(r#""some": "secret""#));

    Ok(())
}

#[test]
fn decrypt_key_from_stdin() -> Result<()> {
    let file = assert_fs::NamedTempFile::new("secrets.ejson")?;
    fs::write(
        file.path(),
        serde_json::json!({
            "_public_key": PUB_KEY,
            "some":"EJ[1:l6yw664nxaddSXGiWUZfuVeoUSpTFHzqAyCpfF8Awxc=:xOfucLDkACGlPCyJ6QViggEidVswUlsH:B/f3DJMkdZHF+Wu9F6XUFwuTmxyfBA==]"
        })
        .to_string(),
    )?;

    Command::cargo_bin("rejson")?
        .arg("decrypt")
        .arg(file.path())
        .arg("--key-from-stdin")
        .write_stdin(PRIV_KEY)
        .assert()
        .success()
        .stdout(predicates::str::contains(r#""some": "secret""#));

    Ok(())
}
