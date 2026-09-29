use core::fmt;
use std::collections::{BTreeMap, HashMap};

use anyhow::{Result, anyhow};
use base64::Engine;

const KEY_DELIMITER: &str = ".";
const NAMESPACE_KEY: &str = "_namespace";
const TYPE_KEY: &str = "_type";

/// A wrapper around a map of dot-delimited keys and values that can be converted into a Kubernetes
/// manifest of secrets.
pub struct SecretsManifest<'a> {
    inner: BTreeMap<&'a str, BTreeMap<&'a str, &'a str>>,
}

impl<'a> SecretsManifest<'a> {
    /// Creates a new [SecretsManifest] from keys in the form `secret_name.key`. Returns an error if
    /// any key isn't nested under a secret name.
    pub fn new(from: HashMap<&'a str, &'a str>) -> Result<Self> {
        // Convert the delimited keys and values into a HashMap of secret name => <Key, Value>.
        //
        // NB: Converts to BTreeMap after filtering so values are sorted by key.
        let resources = from
            .into_iter()
            .collect::<BTreeMap<&str, &str>>()
            .into_iter()
            .try_fold(BTreeMap::new(), |mut map, (k, v)| {
                // secret.key => name = secret, key = key
                // secret.[file.ext] => name = secret, key = file.ext
                let (name, key) = k.split_once(KEY_DELIMITER).ok_or_else(|| {
                    anyhow!(
                        "Expected {:?} to be an object of secret values (e.g. {{\"{}\": {{\"KEY\": \"value\"}}}})",
                        k,
                        k
                    )
                })?;

                // Update values (creating if necessary).
                let values: &mut BTreeMap<&str, &str> = map.entry(name).or_default();
                values.insert(key.trim_matches(&['[', ']'] as &[_]), v);

                Ok::<_, anyhow::Error>(map)
            })?;

        Ok(Self { inner: resources })
    }
}

impl fmt::Display for SecretsManifest<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let b64 = base64::engine::general_purpose::STANDARD;

        self.inner.iter().try_for_each(|(k, data)| {
            writeln!(f, "---")?;
            writeln!(f, "apiVersion: v1")?;
            writeln!(f, "kind: Secret")?;
            writeln!(f, "metadata:")?;
            writeln!(f, "  name: {}", k)?;

            if let Some(ns) = data.get(NAMESPACE_KEY) {
                writeln!(f, "  namespace: {}", ns)?;
            }

            if let Some(kind) = data.get(TYPE_KEY) {
                writeln!(f, "type: {}", kind)?;
            }

            writeln!(f, "data:")?;

            data.iter()
                .filter(|&(k, _)| k != &NAMESPACE_KEY && k != &TYPE_KEY)
                .try_for_each(|(k, v)| writeln!(f, "  {}: {}", k, b64.encode(v)))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_secrets() {
        let secrets = HashMap::from([
            ("database.READ_ONLY_DATABASE_URL", "pgsql://ro_db_url"),
            ("credentials.path", "/some/path/file.ext"),
            ("credentials._namespace", "testing"),
            ("database.DATABASE_URL", "pgsql://db_url"),
        ]);

        let exp = BTreeMap::from([
            (
                "credentials",
                BTreeMap::from([("_namespace", "testing"), ("path", "/some/path/file.ext")]),
            ),
            (
                "database",
                BTreeMap::from([
                    ("DATABASE_URL", "pgsql://db_url"),
                    ("READ_ONLY_DATABASE_URL", "pgsql://ro_db_url"),
                ]),
            ),
        ]);

        let manifest = SecretsManifest::new(secrets).unwrap();
        assert_eq!(exp, manifest.inner);
    }

    #[test]
    fn key_without_secret_name() {
        let secrets = HashMap::from([("database.DATABASE_URL", "pgsql://db_url"), ("orphan", "value")]);

        let err = SecretsManifest::new(secrets).err().expect("should fail");
        assert!(err.to_string().contains("orphan"), "{}", err);
    }

    #[test]
    fn secret_with_type() {
        let secrets = HashMap::from([
            ("database.READ_ONLY_DATABASE_URL", "pgsql://ro_db_url"),
            ("credentials.path", "/some/path/file.ext"),
            ("credentials._namespace", "testing"),
            ("credentials._type", "kubernetes.io/tls"),
            ("database.DATABASE_URL", "pgsql://db_url"),
        ]);

        let exp = BTreeMap::from([
            (
                "credentials",
                BTreeMap::from([
                    ("_namespace", "testing"),
                    ("_type", "kubernetes.io/tls"),
                    ("path", "/some/path/file.ext"),
                ]),
            ),
            (
                "database",
                BTreeMap::from([
                    ("DATABASE_URL", "pgsql://db_url"),
                    ("READ_ONLY_DATABASE_URL", "pgsql://ro_db_url"),
                ]),
            ),
        ]);

        let manifest = SecretsManifest::new(secrets).unwrap();
        assert_eq!(exp, manifest.inner);
    }
}
