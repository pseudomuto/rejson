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

#[cfg(unix)]
#[test]
fn written_private_key_is_owner_read_only() -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let temp = assert_fs::TempDir::new()?;

    cargo_bin_cmd!()
        .arg("keygen")
        .arg("--keydir")
        .arg(temp.path())
        .arg("--write")
        .assert()
        .success();

    let key_file = fs::read_dir(temp.path())?.next().unwrap()?;
    assert_eq!(0o400, key_file.metadata()?.permissions().mode() & 0o777);

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
