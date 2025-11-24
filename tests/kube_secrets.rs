use std::fs;

use anyhow::Result;
use assert_cmd::cargo_bin_cmd;

const PUB_KEY: &str = "b595226c62427adbfc4a809cd7577488a6d402b2f930e1d603164ae3191a616e";
const PRIV_KEY: &str = "88649a9e83f8f1984ad35ac8e8e86529aab518572c0341f46d1e0bc97f676f2b";

#[test]
fn kube_secrets() -> Result<()> {
    let key_file = assert_fs::NamedTempFile::new(PUB_KEY)?;
    fs::write(key_file.path(), PRIV_KEY)?;

    let file = assert_fs::NamedTempFile::new("secrets.ejson")?;
    fs::write(
        file.path(),
        serde_json::json!({
            "_public_key": "b595226c62427adbfc4a809cd7577488a6d402b2f930e1d603164ae3191a616e",
            "kubernetes": {
                "basic-auth": {
                    "username": "EJ[1:t33Bwgtq7Zghz1P0D+8ZMiSypiQye4q9DWLuxaOrLEU=:XJHmDeBhyT9aLjbuzuHyhQc4kCHki9A9:JNCzvxQwWDmOmtE0AQO/y2RSV2Y=]",
                    "password": "EJ[1:t33Bwgtq7Zghz1P0D+8ZMiSypiQye4q9DWLuxaOrLEU=:MrpO2Q3ByLTTZCdDXhNwowZvRuVg7c63:lEwnFAwPtrNXb/IKwqXej9V8MjumSSP5Rg==]"
                },
                "database": {
                    "_namespace": "testing",
                    "DATABASE_URL": "EJ[1:t33Bwgtq7Zghz1P0D+8ZMiSypiQye4q9DWLuxaOrLEU=:0w+6gl3gXIOQohjqZnmih8ZLWPVffurJ:7U6ZEcttjrkS5sA73T/y/hESIaoxJUA320XqBoFWvw==]"
                }
            }
        })
        .to_string(),
    )?;

    let exp = r#"---
apiVersion: v1
kind: Secret
metadata:
  name: basic-auth
data:
  password: cEE1NXdvcmQx
  username: dGVzdA==
---
apiVersion: v1
kind: Secret
metadata:
  name: database
  namespace: testing
data:
  DATABASE_URL: cGdzcWw6Ly9zb21lLWRi"#;

    cargo_bin_cmd!()
        .arg("kube-secrets")
        .arg(file.path())
        .arg("--keydir")
        .arg(key_file.parent().unwrap())
        .assert()
        .success()
        .stdout(predicates::str::contains(exp));

    Ok(())
}

#[test]
fn kube_secrets_key_from_stdin() -> Result<()> {
    let key_file = assert_fs::NamedTempFile::new(PUB_KEY)?;
    fs::write(key_file.path(), PRIV_KEY)?;

    let file = assert_fs::NamedTempFile::new("secrets.ejson")?;
    fs::write(
        file.path(),
        serde_json::json!({
            "_public_key": "b595226c62427adbfc4a809cd7577488a6d402b2f930e1d603164ae3191a616e",
            "kubernetes": {
                "database": {
                    "DATABASE_URL": "EJ[1:t33Bwgtq7Zghz1P0D+8ZMiSypiQye4q9DWLuxaOrLEU=:0w+6gl3gXIOQohjqZnmih8ZLWPVffurJ:7U6ZEcttjrkS5sA73T/y/hESIaoxJUA320XqBoFWvw==]"
                }
            }
        })
        .to_string(),
    )?;

    let exp = r#"---
apiVersion: v1
kind: Secret
metadata:
  name: database
data:
  DATABASE_URL: cGdzcWw6Ly9zb21lLWRi"#;

    cargo_bin_cmd!()
        .arg("kube-secrets")
        .arg(file.path())
        .arg("--key-from-stdin")
        .write_stdin(PRIV_KEY)
        .assert()
        .success()
        .stdout(predicates::str::contains(exp));

    Ok(())
}
