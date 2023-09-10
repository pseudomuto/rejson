use std::{env, path::Path};

use rejson::SecretsMap;

extern crate rejson;

/// Read an already decrypted file. For the sake of this demo, it just loads the encrypted file
/// and shows that the values haven't been altered.
fn main() {
    let path = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("examples")
        .join("data")
        .join("secrets.ejson");

    let map = SecretsMap::load(path).expect("failed to load file");

    println!("sub.key: {}", map.fetch("sub.key"));
    println!("sub.[namespaced.key]: {}", map.fetch("sub.[namespaced.key]"));
}
