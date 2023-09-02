use anyhow::Result;
use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::{fs, process::Command};

#[test]
fn generate_without_writing() -> Result<()> {
    Command::cargo_bin("rejson")?
        .arg("keygen")
        .assert()
        .success()
        .stdout(predicates::str::contains("Private Key:\n"));

    Ok(())
}

#[test]
fn generate_and_write_file_to_keydir() -> Result<()> {
    let temp = assert_fs::TempDir::new()?;

    Command::cargo_bin("rejson")?
        .arg("--keydir")
        .arg(temp.path())
        .arg("keygen")
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

    Command::cargo_bin("rejson")?
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
