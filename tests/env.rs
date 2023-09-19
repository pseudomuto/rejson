use std::fs;

use anyhow::Result;
use assert_cmd::Command;
use assert_fs::prelude::*;

const PUB_KEY: &str = "b595226c62427adbfc4a809cd7577488a6d402b2f930e1d603164ae3191a616e";
const PRIV_KEY: &str = "88649a9e83f8f1984ad35ac8e8e86529aab518572c0341f46d1e0bc97f676f2b";

#[test]
fn env() -> Result<()> {
    let key_file = assert_fs::NamedTempFile::new(PUB_KEY)?;
    fs::write(key_file.path(), PRIV_KEY)?;

    let file = assert_fs::NamedTempFile::new("secrets.ejson")?;
    fs::write(
        file.path(),
        serde_json::json!({
            "_public_key": PUB_KEY,
            "environment": {
                "some":"EJ[1:l6yw664nxaddSXGiWUZfuVeoUSpTFHzqAyCpfF8Awxc=:xOfucLDkACGlPCyJ6QViggEidVswUlsH:B/f3DJMkdZHF+Wu9F6XUFwuTmxyfBA==]"
            }
        })
        .to_string(),
    )?;

    Command::cargo_bin("rejson")?
        .arg("env")
        .arg(file.path())
        .arg("--keydir")
        .arg(key_file.parent().unwrap())
        .assert()
        .success()
        .stdout(predicates::str::contains("export some=secret"));

    Ok(())
}

#[test]
fn env_to_file() -> Result<()> {
    let key_file = assert_fs::NamedTempFile::new(PUB_KEY)?;
    fs::write(key_file.path(), PRIV_KEY)?;

    let file = assert_fs::NamedTempFile::new("secrets.ejson")?;
    fs::write(
        file.path(),
        serde_json::json!({
            "_public_key": PUB_KEY,
            "environment": {
                "some":"EJ[1:l6yw664nxaddSXGiWUZfuVeoUSpTFHzqAyCpfF8Awxc=:xOfucLDkACGlPCyJ6QViggEidVswUlsH:B/f3DJMkdZHF+Wu9F6XUFwuTmxyfBA==]"
            }
        })
        .to_string(),
    )?;

    let out_file = assert_fs::NamedTempFile::new(".envrc")?;

    Command::cargo_bin("rejson")?
        .arg("env")
        .arg(file.path())
        .arg("--keydir")
        .arg(key_file.parent().unwrap())
        .arg("--out")
        .arg(out_file.path())
        .assert()
        .success();

    out_file.assert(predicates::str::contains("export some=secret"));

    Ok(())
}

#[test]
fn env_shell_escape() -> Result<()> {
    let key_file = assert_fs::NamedTempFile::new(PUB_KEY)?;
    fs::write(key_file.path(), PRIV_KEY)?;

    let file = assert_fs::NamedTempFile::new("secrets.ejson")?;
    fs::write(
        file.path(),
        serde_json::json!({
            "_public_key": PUB_KEY,
            "environment": {
                "some": "EJ[1:1an1ebJDsGEnhGd94K9XonLvMokD4HSiKT5xgagdlEw=:KLlxcpkMMUCk4X5aZpNGCG6jUqJoytU2:lAk6EmtaEovXAgw9LuNJYZCYk3DR5ri0KjP3tfNo87U2bguF44qW8hL0BXfuM5olFz0=]"
            }
        })
        .to_string(),
    )?;

    Command::cargo_bin("rejson")?
        .arg("env")
        .arg(file.path())
        .arg("--keydir")
        .arg(key_file.parent().unwrap())
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "export some=\'Some thing with %$# symbols like \\\'",
        ));

    Ok(())
}
