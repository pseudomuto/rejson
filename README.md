# (r)EJSON

[![CI](https://github.com/pseudomuto/rejson/actions/workflows/ci.yaml/badge.svg)](https://github.com/pseudomuto/rejson/actions/workflows/ci.yaml)

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

### Docker

A docker image is published for each release of rEJSON. Usage is similar to using the binary, only the `/keys` and
`/files` volumes are required for encrypt/decrypt functionality.

```
# generate a new key pair
docker run --rm -it rejson:0.1.0 keygen

# decrypt a file to stdout
docker run --rm -it \
  -v ~/.config/ejson:/keys \
  -v $(pwd)/tmp:/files \
  rejson:0.1.0 decrypt /files/secrets.ejson
```
