use core::fmt;
use std::collections::{BTreeMap, HashMap};

use base64::Engine;

const KEY_DELIMITER: &str = ".";

/// A wrapper around a map of dot-delimited keys and values that can be converted into a Kubernetes
/// manifest of secrets.
pub struct SecretsManifest<'a> {
    inner: BTreeMap<&'a str, BTreeMap<&'a str, &'a str>>,
}

impl<'a> SecretsManifest<'a> {
    pub fn new(from: HashMap<&'a str, &'a str>) -> Self {
        // Convert the delimited keys and values into a HashMap of secret name => <Key, Value>.
        //
        // NB: Converts to BTreeMap after filtering so values are sorted by key.
        let resources = from
            .into_iter()
            .map(|(k, v)| (k, v))
            .collect::<BTreeMap<&str, &str>>()
            .iter()
            .fold(BTreeMap::new(), |mut map, (k, v)| {
                // secret.key => name = secret, key = key
                // secret.[file.ext] => name = secret, key = file.ext
                let name = &k[..k.find(KEY_DELIMITER).unwrap()];
                let key = &k[k.find(KEY_DELIMITER).unwrap() + 1..];

                // Update values (creating if necessary).
                let values: &mut BTreeMap<&str, &str> = map.entry(name).or_default();
                values.insert(key.trim_matches(&['[', ']'] as &[_]), v);

                map
            });

        Self { inner: resources }
    }
}

impl fmt::Display for SecretsManifest<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let b64 = base64::engine::general_purpose::STANDARD;

        self.inner.iter().try_for_each(|(k, data)| {
            writeln!(f, "---")?;
            writeln!(f, "api: v1")?;
            writeln!(f, "kind: Secret")?;
            writeln!(f, "metadata:")?;
            writeln!(f, "  name: {}", k)?;
            writeln!(f, "data:")?;

            data.iter()
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
            ("database.DATABASE_URL", "pgsql://db_url"),
        ]);

        let exp = BTreeMap::from([
            ("credentials", BTreeMap::from([("path", "/some/path/file.ext")])),
            (
                "database",
                BTreeMap::from([
                    ("DATABASE_URL", "pgsql://db_url"),
                    ("READ_ONLY_DATABASE_URL", "pgsql://ro_db_url"),
                ]),
            ),
        ]);

        let manifest = SecretsManifest::new(secrets);
        assert_eq!(exp, manifest.inner);
    }
}
