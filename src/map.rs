use std::{collections::HashMap, ops::Deref, path::Path};

use anyhow::Result;
use serde_json::Value;

use crate::{decrypt, Key, SecretsFile};

const SEPARATOR: &str = ".";

/// SecretsMap is a flattened map of secrets, tpically loaded from a EJSON/JSON file. The values
/// are not decrypted here, unless that was done before loading.
///
/// Keys are transformed into dot-notation since this map represents flattened map of secrets. When
/// a key contains '.' characters, it will be surrounded by square brackets.
///
/// ```
/// use rejson::SecretsMap;
/// use serde_json::json;
///
/// # fn main() {
/// let data = json!({
///   "_public_key": "<YOUR_PUBLIC_KEY>",
///   "key": "value",
///   "sub": {
///     "key": "subvalue",
///     "file.ext": "dotvalue"
///   }
/// });
///
/// let secrets: SecretsMap = data.into();
/// assert_eq!("value", secrets.fetch("key"));
/// assert_eq!("subvalue", secrets.fetch("sub.key"));
/// assert_eq!("dotvalue", secrets.fetch("sub.[file.ext]"));
/// # }
/// ```
///
/// For more examples, checkout the _examples_ directory.
pub struct SecretsMap {
    inner: HashMap<String, String>,
}

impl SecretsMap {
    /// Creates a new [SecretsMap] by reading the specified file.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let secrets = SecretsFile::load(path)?;
        Ok(secrets.value.into())
    }

    /// Creates a new [SecretsMap] by reading the supplied file and decrypting it.
    pub fn load_and_decrypt<P: AsRef<Path>>(path: P, private_key: Key) -> Result<Self> {
        let mut secrets = SecretsFile::load(path)?;
        secrets.transform(decrypt(&secrets, private_key)?)?;
        Ok(secrets.value.into())
    }

    /// Fetches the given key from the map, panicking if it isn't found.
    pub fn fetch<K: AsRef<str>>(&self, key: K) -> String {
        self.inner.get(key.as_ref()).unwrap().to_owned()
    }

    /// Fetches the given key from the map, returning the supplied default value if it doesn't
    /// exist.
    pub fn fetch_or<K: AsRef<str>, V: Into<String>>(&self, key: K, default: V) -> String {
        self.inner.get(key.as_ref()).unwrap_or(&default.into()).to_owned()
    }
}

impl Deref for SecretsMap {
    type Target = HashMap<String, String>;

    /// Delegates (non-mut) calls to the inner map.
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<Value> for SecretsMap {
    fn from(value: Value) -> Self {
        let mut map = HashMap::new();

        if let Some(value) = value.as_object() {
            value.iter().for_each(|(k, v)| {
                extract_keys(&mut map, &safe_key(k), v);
            });
        }

        Self { inner: map }
    }
}

fn extract_keys(map: &mut HashMap<String, String>, key: &str, value: &Value) {
    match value {
        Value::Object(obj) => obj.iter().for_each(|(k, v)| {
            extract_keys(map, &format!("{}.{}", key, safe_key(k)), v);
        }),
        Value::String(s) => {
            map.insert(key.into(), s.to_string());
        }
        Value::Number(n) => {
            map.insert(key.into(), n.to_string());
        }
        Value::Bool(b) => {
            map.insert(key.into(), b.to_string());
        }
        _ => {}
    }
}

fn safe_key<K: Into<String>>(k: K) -> String {
    let key = k.into();

    if key.contains(SEPARATOR) {
        format!("[{}]", key)
    } else {
        key
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn load() {
        let secrets_path = Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap())
            .join("examples")
            .join("data")
            .join("secrets.ejson");

        assert!(SecretsMap::load(secrets_path).is_ok());
    }

    #[test]
    fn load_and_decrypt() -> Result<()> {
        let secrets_path = Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap())
            .join("examples")
            .join("data")
            .join("secrets.ejson");

        let key_path = Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap())
            .join("examples")
            .join("data")
            .join("2549b26efec29cf60e473797f5dda5f41d99460cf1c32f34f1c0247d9bd7ff5b");

        let map = SecretsMap::load_and_decrypt(secrets_path, Key::from_file(key_path)?)?;
        assert_eq!("key", map.fetch("some"));

        Ok(())
    }

    #[test]
    fn fetch() {
        let data = json!({
          "_public_key": "anything",
          "some": "key"
        });

        let map: SecretsMap = data.into();
        assert_eq!("key", map.fetch("some"));
        assert_eq!("default", map.fetch_or("wat", "default"));
    }

    #[test]
    fn non_string_scalars() {
        let data = json!({
          "_public_key": "anything",
          "int": 10,
          "float": 10.542,
          "bool": true,
        });

        let map: SecretsMap = data.into();
        assert_eq!("10", map.fetch("int"));
        assert_eq!("10.542", map.fetch("float"));
        assert_eq!("true", map.fetch("bool"));
    }

    #[test]
    fn key_wrapping() {
        let data = json!({
          "_public_key": "anything",
          "environment.test": "top-level value", // ensure it's not overridden
          "environment": {
            "test":"value",
            "_a": {
              "b": "n",
              "_c": "c",
              "key.json": "contents"
            }
          },
          "other": "key"
        });

        let map: SecretsMap = data.into();
        assert_eq!("anything", map.fetch("_public_key"));
        assert_eq!("key", map.fetch("other"));
        assert_eq!("top-level value", map.fetch("[environment.test]"));
        assert_eq!("value", map.fetch("environment.test"));
        assert_eq!("n", map.fetch("environment._a.b"));
        assert_eq!("contents", map.fetch("environment._a.[key.json]"));
    }
}
