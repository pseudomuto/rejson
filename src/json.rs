use std::{fmt, path::Path, str::FromStr};

use anyhow::Result;
use serde_json::Value;

use crate::crypto::Key;

const PK_KEY: &str = "_public_key";
const IGNORE_PREFIX: &str = "_";

#[derive(Debug)]
pub struct SecretsFile {
    value: Value,
}

impl SecretsFile {
    /// Creates a new [Parser] by reading the specified file.
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
    pub fn transform<F>(&mut self, transformer: F) -> Result<()>
    where
        F: Fn(String) -> Result<String>,
    {
        self.value
            .as_object_mut()
            .unwrap()
            .iter_mut()
            .try_for_each(|(k, v)| transform(k, v, &transformer))
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

fn transform<F>(key: &str, value: &mut Value, tfn: &F) -> Result<()>
where
    F: Fn(String) -> Result<String>,
{
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

        let mut parser = SecretsFile { value: data };
        assert!(parser.transform(|_| Ok("Encrypted".to_string())).is_ok());
        assert_eq!(exp.to_string(), parser.value.to_string());
    }
}
