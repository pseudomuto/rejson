# (r)EJSON

[![CI](https://github.com/pseudomuto/rejson/actions/workflows/ci.yaml/badge.svg)](https://github.com/pseudomuto/rejson/actions/workflows/ci.yaml)
[![Latest version](https://img.shields.io/crates/v/rejson.svg)](https://crates.io/crates/rejson)
[![Docs](https://img.shields.io/badge/docs-rs-blue)](https://docs.rs/rejson/latest)

 `rejson` is a utility for managing a collection of secrets in source control. The secrets are encrypted using
 [public key], [elliptic curve] cryptography ([NaCl] [Box]: [Curve25519] + [Salsa20] + [Poly1305-AES]). Secrets are
 collected in a JSON file, in which all the string values are encrypted. Public keys are embedded in the file, and
 the decrypter looks up the corresponding private key from its local filesystem.

> This is a rust port of [EJSON] with a few extra bells and whistles. Full credit should go to the team that made EJSON. No
innovation here other than needing Rust bindings and wanting a few extra features I'm not sure belonged upstream.

[public key]: http://en.wikipedia.org/wiki/Public-key_cryptography
[elliptic curve]: http://en.wikipedia.org/wiki/Elliptic_curve_cryptography
[NaCl]: http://nacl.cr.yp.to/
[Box]: http://nacl.cr.yp.to/box.html
[Curve25519]: http://en.wikipedia.org/wiki/Curve25519
[Poly1305-AES]: http://en.wikipedia.org/wiki/Poly1305-AES
[Salsa20]: http://en.wikipedia.org/wiki/Salsa20
[EJSON]: https://github.com/Shopify/ejson

## Usage

### CLI

See `rejson -h` or (`cargo run -- -h`) for usage details.

```ignore
A command line utility for managing secrets

Usage: rejson <COMMAND>

Commands:
  encrypt  Encrypt one or more EJSON files
  decrypt  Decrypt an EJSON file
  keygen   Generate a new EJSON key pair
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### Docker

A docker image is published for each release of rEJSON. Usage is similar to using the binary, only the `/keys` and
`/files` volumes are required for encrypt/decrypt functionality.

```ignore
# generate a new key pair
docker run --rm -it rejson keygen

# encrypt a file to disk
docker run --rm -it \
  -v $(pwd)/keys:/keys \
  -v $(pwd)/secrets:/files \
  rejson encrypt /files/secrets.ejson

# decrypt a file to stdout
docker run --rm -it \
  -v $(pwd)/keys:/keys \
  -v $(pwd)/secrets:/files \
  rejson decrypt /files/secrets.ejson
```

### Code 

```rust
use std::fs;

use rejson::{KeyPair, SecretsFile};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = "build/secrets.ejson";
    let mut secrets_file = SecretsFile::load(file).expect("failed to load file");
    secrets_file.transform(rejson::compact()?)?;
    secrets_file.transform(rejson::encrypt(&secrets_file)?)?;

    let json = secrets_file.to_string();
    let data = json.as_bytes();
    fs::write(file, data)?;

    println!("Wrote {} bytes to {}", data.len(), file);
    Ok(())
}
```

## Development

### Local Setup

* Ensure you're using nightly (currently required only for Rustfmt and the `trait_alias` feature).
* Add pre-commit to avoid committing malformatted code 
  
```ignore
ln -s -f ../../build/pre-commit .git/hooks/pre-commit
```

### Cutting a New release

Run `build/release`. This will:

* Update version in Cargo.toml
* Create a new commit with the message "Release v<version>"
* `git tag -sm "Release v<version" v<version>`
* `git push --tags`

From there, the release pipeline will publish the crate and the corresponding docker image.

