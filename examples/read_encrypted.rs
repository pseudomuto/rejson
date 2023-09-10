use std::{env, path::Path};

use rejson::{Key, SecretsMap};

extern crate rejson;

/// Read an encrypted file and decrypt it before returning the [SecretsMap].
fn main() {
    let path = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("examples")
        .join("data")
        .join("secrets.ejson");

    let key_path = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("examples")
        .join("data")
        .join("2549b26efec29cf60e473797f5dda5f41d99460cf1c32f34f1c0247d9bd7ff5b");

    let private_key = Key::from_file(key_path).expect("failed parse private key");
    let map = SecretsMap::load_and_decrypt(path, private_key).expect("failed to load file");

    println!("sub.key: {}", map.fetch("sub.key"));
    println!("sub.[namespaced.key]: {}", map.fetch("sub.[namespaced.key]"));
}
