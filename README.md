# (r)EJSON

[![CI](https://github.com/pseudomuto/rejson/actions/workflows/ci.yaml/badge.svg)](https://github.com/pseudomuto/rejson/actions/workflows/ci.yaml)
[![Latest version](https://img.shields.io/crates/v/rejson.svg)](https://crates.io/crates/rejson)
[![Docs](https://img.shields.io/badge/docs-rs-blue)](https://docs.rs/rejson/latest)
[![codecov](https://codecov.io/gh/pseudomuto/rejson/graph/badge.svg?token=pEmI3xM9Ae)](https://codecov.io/gh/pseudomuto/rejson)

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

## Install

**From Releases**

```ignore
curl -fsSL https://github.com/pseudomuto/rejson/releases/download/v0.2.0/rejson_0.2.0_$(uname -s)_$(uname -m).tar.gz | tar xzf -
```

**With Cargo**

`cargo install rejson`

Since this is a drop-in replacement for `ejson` you can add `alias ejson="rejson"` if you like. The expectation is that
this is 100% compatible with `ejson` and it only additive. If that's not the case, it's a bug, and I'd appreciate you 
filing an issue.

### Additions to EJSON

* A `--strip-key` flag on `decrypt` which will remove `_public_key` from the result.
* `env` command which will export all keys under the top-level `environment` key.
* `kube-secrets` command which will output K8s secret manifests for values under the `kubernetes` key.

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
  env      Export the all scalar values under the "environment" key
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

To export all environment values in the environment key, run `eval $(rejson env secrets.ejson)`.

```ignore
{
  "_public_key": "...",
  "environment": {
    "SOME_KEY": "SOME_VALUE"
  }
}
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
    let file = "examples/data/secrets.ejson";
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
See the [_examples_](examples/) directory for more.

## Development

### Local Setup

* Make sure you have the nightly toolchain (used for rustfmt only)
* Add pre-commit to avoid committing malformatted code 
  
```ignore
ln -sf ../../build/pre-commit .git/hooks/pre-commit
```

### Cutting a New release

Run `build/release <new_version>`. This will:

* Update version in Cargo.toml
* Create a new commit with the message "Release v<version>"
* `git tag -sm "Release v<version" v<version>`
* `git push --tags`

Once the release pipeline has finished and published the crate, run the following to create the GitHub release with
attached binaries, etc.

```ignore
goreleaser release --clean
```

> Yes, I've hacked goreleaser into thinking this is a go project so I can leverage it for running cross, publishing docker
images, and setting up GitHub releases.

