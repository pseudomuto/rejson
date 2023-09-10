use std::{collections::HashMap, fmt, path::Path, str::FromStr};

use anyhow::Result;
use serde_json::Value;

use crate::crypto::Key;

const PK_KEY: &str = "_public_key";
const IGNORE_PREFIX: &str = "_";

#[derive(Debug)]
pub struct SecretsFile {
    pub(crate) value: Value,
}

impl SecretsFile {
    /// Creates a new [SecretsFile] by reading the specified file.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        std::fs::read_to_string(path.as_ref())?.parse()
    }

    /// Extracts the public key from the JSON object.
    pub fn public_key(&self) -> Option<Key> {
        if let Some(s) = self.value[PK_KEY].as_str() {
            return s.parse().ok();
        }

        None
    }

    /// Performs the supplied transformation function on each eligible value in the document.
    /// Eligible in this case refers to string values who's key does not start with an underscore.
    ///
    /// The transformer will be called for each eligible string value and is expected to return a
    /// new value to be used in it's place.
    ///
    /// This function transforms the values in place by mutating the underlying structure.
    pub fn transform<F: Fn(String) -> Result<String>>(&mut self, transformer: F) -> Result<()> {
        self.value
            .as_object_mut()
            .unwrap()
            .iter_mut()
            .try_for_each(|(k, v)| transform(k, v, &transformer))
    }

    /// Returns a map of all direct children of the supplied key with scalar values.
    pub fn children(&self, root_key: &str) -> Option<HashMap<&str, &str>> {
        self.value.get(root_key).map(|value| {
            value
                .as_object()
                .unwrap()
                .iter()
                .fold(HashMap::new(), |mut acc, (key, value)| {
                    if value.is_string() {
                        acc.insert(key.as_str(), value.as_str().unwrap());
                    }

                    acc
                })
        })
    }

    /// Returns a new [SecretsFile] that is a clone of this one without the _public_key field.
    pub fn without_public_key(&self) -> Self {
        let mut value = self.value.clone();
        value.as_object_mut().unwrap().remove("_public_key");
        Self { value }
    }
}

impl fmt::Display for SecretsFile {
    /// Returns the pretty-printed JSON representation.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        serde_json::to_string_pretty(&self.value)
            .map_err(|_| fmt::Error {})
            .map(|data| f.write_str(&data))
            .map(|_| ())
    }
}

impl FromStr for SecretsFile {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            value: serde_json::from_str(s)?,
        })
    }
}

fn transform<F: Fn(String) -> Result<String>>(key: &str, value: &mut Value, tfn: &F) -> Result<()> {
    match value {
        Value::String(v) => {
            // Only interested in string values who's keys do not start with an underscore as
            // outlined in the EJSON spec.
            if !key.starts_with(IGNORE_PREFIX) {
                *value = Value::String(tfn(v.to_string())?);
            }
            Ok(())
        }
        Value::Object(_) => value
            .as_object_mut()
            .unwrap()
            .iter_mut()
            .try_for_each(|(k, v)| transform(k, v, tfn)),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use super::*;

    #[test]
    fn public_key() {
        assert!(
            SecretsFile {
                value: json!({"_public_key":"344b86d41cbb5660d98f59b4a7b35f3128e0d0b9c4b06f05ca7ae28b9c7dd72e"})
            }
            .public_key()
            .is_some(),
            "should be good"
        );

        assert!(
            SecretsFile {
                value: json!({"_public_key": "nope"})
            }
            .public_key()
            .is_none(),
            "bad value"
        );

        assert!(SecretsFile { value: json!({}) }.public_key().is_none(), "not found");
    }

    #[test]
    fn from_str() {
        let data = json!({
          "_public_key": "anything",
          "environment": {
            "test":"value"
          },
          "other": "key"
        });

        assert!(data.to_string().parse::<SecretsFile>().is_ok());
    }

    #[test]
    fn transform() {
        let data = json!({
          "_public_key": "anything",
          "environment": {
            "test":"value",
            "_a": {
              "b": "n",
              "_c": "c"
            }
          },
          "other": "key"
        });

        let exp = json!({
          "_public_key": "anything",
          "environment": {
            "test":"Encrypted",
            "_a": {
              "b": "Encrypted",
              "_c": "c"
            }
          },
          "other": "Encrypted"
        });

        let mut file = SecretsFile { value: data };
        assert!(file.transform(|_| Ok("Encrypted".to_string())).is_ok());
        assert_eq!(exp.to_string(), file.value.to_string());
    }

    #[test]
    fn children() {
        let data = json!({
          "_public_key": "anything",
          "environment": {
            "test":"value",
            "_a": {
              "b": "n",
              "_c": "c"
            },
            "other": "thing"
          },
          "other": "key"
        });

        let file = SecretsFile { value: data };
        let env = file.children("environment").unwrap();
        assert_eq!(HashMap::from([("test", "value"), ("other", "thing"),]), env);

        assert!(file.children("wat").is_none());
    }

    #[test]
    fn without_public_key() {
        let data = json!({
          "_public_key": "anything",
          "environment": {
            "test":"value",
            "_a": {
              "b": "n",
              "_c": "c"
            }
          },
          "other": "key"
        });

        let file = SecretsFile { value: data }.without_public_key();
        assert!(file.value.get("_public_key").is_none());
    }
}
