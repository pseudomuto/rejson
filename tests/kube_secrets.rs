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
                },
                "tls-cert": {
                    "_namespace": "cert-manager",
                    "_type": "kubernetes.io/tls",
                    "ca.crt": "EJ[1:YeaslA2I2b2MaIJgnDSKCZyE5JoZfbU60jdXYVefzgo=:lvyuyUzBIJUcZdxvqP5cz3oOX+UE2KHr:a1HJv8fciDo2uI3UeUdfaaZhhjB5KbCtBuf3knGTMG9a2a4=]",
                    "tls.crt": "EJ[1:YeaslA2I2b2MaIJgnDSKCZyE5JoZfbU60jdXYVefzgo=:bkg3iUfkYJG5Br7quMujuA+J2cUMOZmM:nHd0SRV1eeL0rxC0nzuoVi4jTvOBSAMdskZMz1+a8w8=]",
                    "tls.key": "EJ[1:YeaslA2I2b2MaIJgnDSKCZyE5JoZfbU60jdXYVefzgo=:Eexh3Hl446GCy5Kloz55YVfr7WXOWg2g:27YfcdLAVmud4c8C4MFpHjvo9WE3geVt]"
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
  DATABASE_URL: cGdzcWw6Ly9zb21lLWRi
---
apiVersion: v1
kind: Secret
metadata:
  name: tls-cert
  namespace: cert-manager
type: kubernetes.io/tls
data:
  ca.crt: U29tZSBDQSBDZXJ0aWZpY2F0ZQ==
  tls.crt: U29tZSBDZXJ0aWZpY2F0ZQ==
  tls.key: U29tZSBLZXk="#;

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
