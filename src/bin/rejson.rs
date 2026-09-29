use std::{fs, io::Write};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use rejson::{self, Key, KeyPair, SecretsFile, SecretsManifest, SecretsMap};

/// The default place to find private keys.
const DEFAULT_KEYDIR: &str = "/opt/ejson/keys";

/// Key for env command.
const ENV_KEY: &str = "environment";

/// Key for kube-secrets command.
const KUBE_SECRETS_KEY: &str = "kubernetes";

#[derive(Parser)]
#[command(author, about, version)]
struct Cli {
    // Global (like upstream EJSON), so it can be given before or after the subcommand.
    /// The directory containing EJSON private keys.
    #[arg(short, long, env = "EJSON_KEYDIR", global = true, default_value = DEFAULT_KEYDIR)]
    keydir: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Encrypt one or more EJSON files.
    #[command(alias = "e")]
    Encrypt {
        /// The file(s) to encrypt.
        #[arg(num_args = 1.., value_parser)]
        file: Vec<String>,
    },

    /// Decrypt an EJSON file.
    ///
    /// Decrypt the given file; that is, decrypt all the encrypted keys within it, printing the full decrypted file.
    /// The key mentioned in the ejson file must be present in the keydir.
    #[command(alias = "d")]
    Decrypt {
        /// The file to decrypt.
        file: String,

        /// Read the private key from stdin.
        #[arg(long)]
        key_from_stdin: bool,

        /// If given, write the decrypted file to FILE rather than stdout.
        #[arg(short, long)]
        out: Option<String>,

        /// Omit the _public_key from the result.
        #[arg(short, long)]
        strip_key: bool,
    },

    /// Generate a new EJSON key pair.
    #[command(alias = "g")]
    Keygen {
        /// Write the private key to the key dir.
        #[arg(short, long)]
        write: bool,
    },

    /// Export the all values under the "environment" key.
    Env {
        /// The file to decrypt.
        file: String,

        /// Read the private key from stdin.
        #[arg(long)]
        key_from_stdin: bool,

        /// The path to write the export statements to.
        #[arg(short, long)]
        out: Option<String>,
    },

    /// Generate a K8s manifest for secrets defined under the "kubernetes" key.
    ///
    /// The expected format for this key is as follows:
    ///
    /// ```json
    /// {
    ///   "_public_key": "...",
    ///   ...
    ///   "kubernetes": {
    ///     "secret_name": {
    ///       "KEY": "EJ[1:...]",
    ///       "OTHER_KEY": "EJ[1:...]"
    ///     },
    ///     ...
    ///     ...
    ///   }
    /// }
    /// ```
    ///
    /// The output would be a manifest will something like the following (one for each secret
    /// separated by `---`)
    ///
    /// ```yaml
    /// api: v1
    /// kind: Secret
    /// metadata:
    ///   name: secret_name
    /// data:
    ///   KEY: <base64 decrypted value>
    ///   OTHER_KEY: <base64 decrypted value>
    /// ```
    ///
    /// You can optionally add `"_namespace": "my-ns"` to any secret to have it be defined in the
    /// my-ns namespace.
    KubeSecrets {
        /// The file to decrypt.
        file: String,

        /// Read the private key from stdin.
        #[arg(long)]
        key_from_stdin: bool,

        /// The path to write the manifest to.
        #[arg(short, long)]
        out: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let keydir = cli.keydir;

    match cli.command {
        Commands::Encrypt { file } => encrypt(file),
        Commands::Decrypt {
            file,
            key_from_stdin,
            out,
            strip_key,
        } => decrypt(file, keydir, key_from_stdin, out, strip_key),
        Commands::Keygen { write } => keygen(keydir, write),
        Commands::Env {
            file,
            key_from_stdin,
            out,
        } => export_env(file, keydir, key_from_stdin, out),
        Commands::KubeSecrets {
            file,
            key_from_stdin,
            out,
        } => kube_secrets_manifest(file, keydir, key_from_stdin, out),
    }
}

fn encrypt(files: Vec<String>) -> Result<()> {
    files.iter().try_for_each(|file_path| {
        let mut secrets_file = SecretsFile::load(file_path)?;
        secrets_file.transform(rejson::compact()?)?;
        secrets_file.transform(rejson::encrypt(&secrets_file)?)?;

        let json = secrets_file.to_string();
        let data = json.as_bytes();

        fs::write(file_path, data)?;
        println!("Wrote {} bytes to {}", data.len(), file_path);
        Ok(())
    })
}

fn decrypt(file: String, keydir: String, key_from_stdin: bool, out: Option<String>, strip_key: bool) -> Result<()> {
    let mut secrets_file = SecretsFile::load(file)?;

    let private_key = load_private_key(&secrets_file, keydir, key_from_stdin)?;
    secrets_file.transform(rejson::decrypt(&secrets_file, private_key)?)?;

    if strip_key {
        // Useful for things like exporting tfvars without wanting to see the warning
        // about an unknown variable. Clearly you could do this on the CLI, but we've
        // got a tool, so you know...make it do what you want.
        secrets_file = secrets_file.without_public_key();
    }

    if let Some(path) = out {
        fs::write(path, secrets_file.to_string())?;
    } else {
        println!("{}", secrets_file);
    }

    Ok(())
}

fn keygen(keydir: String, write: bool) -> Result<()> {
    let pair = KeyPair::generate().unwrap();
    println!("Public Key:");
    println!("{}", pair.public_key());

    if !write {
        println!("Private Key:");
        println!("{}", pair.private_key());
        return Ok(());
    }

    let path = std::path::Path::new(&keydir).join(pair.public_key());
    let mut options = fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);

    // Private keys should only be readable by the owner (matches upstream EJSON).
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o400);

    options
        .open(&path)
        .and_then(|mut file| file.write_all(pair.private_key().as_bytes()))
        .with_context(|| {
            format!(
                "Failed to write private key to {} (does the keydir exist and is it writable?)",
                path.display()
            )
        })
}

fn export_env(file: String, keydir: String, key_from_stdin: bool, out: Option<String>) -> Result<()> {
    let mut secrets_file = SecretsFile::load(file)?;

    let private_key = load_private_key(&secrets_file, keydir, key_from_stdin)?;
    secrets_file.transform(rejson::decrypt(&secrets_file, private_key)?)?;

    match secrets_file.children(ENV_KEY)? {
        Some(map) => {
            let map = &map;

            // Keys are written unescaped, so anything other than a plain identifier could inject
            // shell code when the output is eval'd. Check them all before writing anything.
            if let Some(key) = map.keys().find(|k| !is_env_var_name(k)) {
                anyhow::bail!("{:?} is not a valid environment variable name", key);
            }

            out.map_or_else(
                || {
                    map.iter()
                        .for_each(|(k, v)| println!("export {}={}", k, shell_escape::escape(v.to_string().into())));
                    Ok(())
                },
                |out| {
                    let mut file = fs::File::create(out)?;
                    map.iter()
                        .try_for_each(|(k, v)| {
                            writeln!(file, "export {}={}", k, shell_escape::escape(v.to_string().into()))
                        })
                        .map_err(|e| anyhow::anyhow!(e.to_string()))
                },
            )
        }
        None => {
            eprintln!("No {} key found. Nothing to export.", ENV_KEY);
            Ok(())
        }
    }
}

/// Returns whether the supplied key is a valid POSIX shell variable name (`[A-Za-z_][A-Za-z0-9_]*`).
fn is_env_var_name(key: &str) -> bool {
    let mut chars = key.chars();

    chars.next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn kube_secrets_manifest(file: String, keydir: String, key_from_stdin: bool, out: Option<String>) -> Result<()> {
    let secrets_file = SecretsFile::load(&file)?;

    // Fail on a non-object "kubernetes" value rather than silently producing an empty manifest (a
    // scalar would flatten to a key without the "kubernetes." prefix and be filtered out below).
    secrets_file.children(KUBE_SECRETS_KEY)?;

    let private_key = load_private_key(&secrets_file, keydir, key_from_stdin)?;
    let secrets = SecretsMap::load_and_decrypt(&file, private_key)?;

    let prefix = format!("{}.", KUBE_SECRETS_KEY);
    let manifest = SecretsManifest::new(
        secrets
            .iter()
            .filter_map(|(k, v)| k.strip_prefix(&prefix).map(|k| (k, v.as_str())))
            .collect(),
    )?;

    out.map_or_else(
        || {
            println!("{}", manifest);
            Ok(())
        },
        |out| {
            let mut file = fs::File::create(out)?;
            writeln!(file, "{}", manifest).map_err(anyhow::Error::msg)
        },
    )
}

/// Load the private key from the keydir or stdin.
fn load_private_key(secrets_file: &SecretsFile, keydir: String, key_from_stdin: bool) -> Result<Key> {
    if key_from_stdin {
        let mut buffer = String::new();
        std::io::stdin().read_line(&mut buffer)?;
        return buffer.trim().parse();
    }

    rejson::load_private_key(secrets_file, &keydir)
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert()
}

#[test]
fn keydir_defaults_to_upstream_location() {
    use clap::CommandFactory;

    let cmd = Cli::command();
    let keydir = cmd.get_arguments().find(|a| a.get_id() == "keydir").unwrap();
    assert_eq!([std::ffi::OsStr::new(DEFAULT_KEYDIR)], keydir.get_default_values());
}
