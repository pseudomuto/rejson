use std::fs;

use anyhow::Result;
use assert_cmd::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn generate_without_writing() -> Result<()> {
    cargo_bin_cmd!()
        .arg("keygen")
        .assert()
        .success()
        .stdout(predicates::str::contains("Private Key:\n"));

    Ok(())
}

#[test]
fn generate_and_write_file_to_keydir() -> Result<()> {
    let temp = assert_fs::TempDir::new()?;

    cargo_bin_cmd!()
        .arg("keygen")
        .arg("--keydir")
        .arg(temp.path())
        .arg("--write")
        .assert()
        .success()
        .stdout(predicates::str::contains("Private Key:").not());

    let paths = fs::read_dir(temp.path())?;
    assert_eq!(1, paths.count());

    Ok(())
}

#[test]
fn generate_and_write_file_to_ejson_keydir() -> Result<()> {
    let temp = assert_fs::TempDir::new()?;

    cargo_bin_cmd!()
        .env("EJSON_KEYDIR", temp.path())
        .arg("keygen")
        .arg("--write")
        .assert()
        .success()
        .stdout(predicates::str::contains("Private Key:").not());

    let paths = fs::read_dir(temp.path())?;
    assert_eq!(1, paths.count());

    Ok(())
}
