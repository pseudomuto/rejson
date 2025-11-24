use std::fs;

use anyhow::Result;
use assert_cmd::cargo_bin_cmd;
use assert_fs::prelude::*;

const PUB_KEY: &str = "344b86d41cbb5660d98f59b4a7b35f3128e0d0b9c4b06f05ca7ae28b9c7dd72e";

#[test]
fn encrypt_file() -> Result<()> {
    let file = assert_fs::NamedTempFile::new("secrets.ejson")?;
    fs::write(
        file.path(),
        serde_json::json!({
            "_public_key": PUB_KEY,
            "secret": "ssshh"
        })
        .to_string(),
    )?;

    cargo_bin_cmd!()
        .arg("encrypt")
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicates::str::contains(format!(
            "Wrote 215 bytes to {}",
            file.path().display(),
        )));

    Ok(())
}

#[test]
fn encrypt_file_existing_secrets() -> Result<()> {
    let file = assert_fs::NamedTempFile::new("secrets.ejson")?;
    fs::write(
        file.path(),
        serde_json::json!({
            "_public_key": PUB_KEY,
            "some": "EJ[1:wZfxU7QxHVDOpk8rHbS5gcBifYezXYqWIsqbilW4C1I=:PWub0c7QfhmmyHebe9CbVorEgxYgnLgI:c1rAkxbPdSHuYnI/qDNE1LglJQ==]",
            "secret": "ssshh"
        })
        .to_string(),
    )?;

    cargo_bin_cmd!()
        .arg("encrypt")
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicates::str::contains(format!(
            "Wrote 341 bytes to {}",
            file.path().display(),
        )));

    // original secret should be unchanged
    file.assert(predicates::str::contains("EJ[1:wZfxU7QxHVDOpk8rHbS5gcBifYezXYqWIsqbilW4C1I=:PWub0c7QfhmmyHebe9CbVorEgxYgnLgI:c1rAkxbPdSHuYnI/qDNE1LglJQ==]"));

    Ok(())
}

#[test]
fn encrypt_multiple_files() -> Result<()> {
    let file1 = assert_fs::NamedTempFile::new("secrets1.ejson")?;
    let file2 = assert_fs::NamedTempFile::new("secrets2.ejson")?;
    let file3 = assert_fs::NamedTempFile::new("secrets3.ejson")?;

    let mut expected_output = Vec::new();
    [&file1, &file2, &file3].iter().try_for_each(|file| {
        expected_output.push(format!("Wrote 215 bytes to {}", file.path().display()));

        fs::write(
            file.path(),
            serde_json::json!({
                "_public_key": PUB_KEY,
                "secret": "ssshh"
            })
            .to_string(),
        )
    })?;

    cargo_bin_cmd!()
        .arg("encrypt")
        .arg(file1.path())
        .arg(file2.path())
        .arg(file3.path())
        .assert()
        .success()
        .stdout(predicates::str::contains(expected_output.join("\n")));

    Ok(())
}
