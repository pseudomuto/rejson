# (r)EJSON

[![CI](https://github.com/pseudomuto/rejson/actions/workflows/ci.yaml/badge.svg)](https://github.com/pseudomuto/rejson/actions/workflows/ci.yaml)
[![Latest version](https://img.shields.io/crates/v/rejson.svg)](https://crates.io/crates/rejson)
[![Docs](https://img.shields.io/badge/docs-rs-blue)](https://docs.rs/rejson/latest)

A rust port of EJSON with a few extra bells and whistles. Full credit should go to the team that made EJSON. No
innovation here other than needing Rust bindings and wanting a few extra features I'm not sure belonged upstream.

## Local Setup

* Ensure you're using nightly (currently required only for Rustfmt).
* Add pre-commit to avoid committing malformated code 
  
```
ln -s -f ../../build/pre-commit .git/hooks/pre-commit
```

## Usage

See `rejson -h` or (`cargo run -- -h`) for usage details.

```
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

```
# generate a new key pair
docker run --rm -it rejson:0.1.0 keygen

# encrypt a file to disk
docker run --rm -it \
  -v $(pwd)/keys:/keys \
  -v $(pwd)/secrets:/files \
  rejson:0.1.0 encrypt /files/secrets.ejson

# decrypt a file to stdout
docker run --rm -it \
  -v $(pwd)/keys:/keys \
  -v $(pwd)/secrets:/files \
  rejson:0.1.0 decrypt /files/secrets.ejson
```

### Cutting a New release

* Update version in Cargo.toml
* Create a new commit with the message "Release v<version>"
* `git tag -sm "Release v<version" v<version>`
* `git push --tags`

This will publish the crate and the corresponding docker image.
