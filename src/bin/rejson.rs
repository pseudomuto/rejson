use std::{fs, io::Write};

use anyhow::Result;
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

        #[arg(env = "EJSON_KEYDIR", long)]
        keydir: Option<String>,

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
        #[arg(env = "EJSON_KEYDIR", long)]
        keydir: Option<String>,

        /// Write the private key to the key dir.
        #[arg(short, long)]
        write: bool,
    },

    /// Export the all values under the "environment" key.
    Env {
        /// The file to decrypt.
        file: String,

        #[arg(env = "EJSON_KEYDIR", long)]
        keydir: Option<String>,

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

        #[arg(env = "EJSON_KEYDIR", long)]
        keydir: Option<String>,

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

    match cli.command {
        Commands::Encrypt { file } => encrypt(file),
        Commands::Decrypt {
            file,
            keydir,
            key_from_stdin,
            out,
            strip_key,
        } => decrypt(file, keydir, key_from_stdin, out, strip_key),
        Commands::Keygen { keydir, write } => keygen(keydir, write),
        Commands::Env {
            file,
            keydir,
            key_from_stdin,
            out,
        } => export_env(file, keydir, key_from_stdin, out),
        Commands::KubeSecrets {
            file,
            keydir,
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

fn decrypt(
    file: String,
    keydir: Option<String>,
    key_from_stdin: bool,
    out: Option<String>,
    strip_key: bool,
) -> Result<()> {
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

fn keygen(keydir: Option<String>, write: bool) -> Result<()> {
    if write && keydir.is_none() {
        return Err(anyhow::anyhow!(
            "Either EJSON_KEYDIR must be set or --keydir must be supplied"
        ));
    }

    let pair = KeyPair::generate().unwrap();
    println!("Public Key:");
    println!("{}", pair.public_key());

    if !write {
        println!("Private Key:");
        println!("{}", pair.private_key());
        return Ok(());
    }

    let path = std::path::Path::new(&keydir.unwrap()).join(pair.public_key());
    std::fs::File::create(path)?
        .write_all(pair.private_key().as_bytes())
        .map_err(anyhow::Error::msg)
}

fn export_env(file: String, keydir: Option<String>, key_from_stdin: bool, out: Option<String>) -> Result<()> {
    let mut secrets_file = SecretsFile::load(file)?;

    let private_key = load_private_key(&secrets_file, keydir, key_from_stdin)?;
    secrets_file.transform(rejson::decrypt(&secrets_file, private_key)?)?;

    match secrets_file.children(ENV_KEY) {
        Some(map) => {
            let map = &map;

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

fn kube_secrets_manifest(
    file: String,
    keydir: Option<String>,
    key_from_stdin: bool,
    out: Option<String>,
) -> Result<()> {
    let secrets_file = SecretsFile::load(&file)?;
    let private_key = load_private_key(&secrets_file, keydir, key_from_stdin)?;
    let secrets = SecretsMap::load_and_decrypt(&file, private_key)?;

    let manifest = SecretsManifest::new(
        secrets
            .iter()
            .filter(|(k, _)| k.starts_with(KUBE_SECRETS_KEY))
            .map(|(k, v)| (k.strip_prefix(&format!("{}.", KUBE_SECRETS_KEY)).unwrap(), v.as_str()))
            .collect(),
    );

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
fn load_private_key(secrets_file: &SecretsFile, keydir: Option<String>, key_from_stdin: bool) -> Result<Key> {
    if key_from_stdin {
        let mut buffer = String::new();
        std::io::stdin().read_line(&mut buffer)?;
        return buffer.trim().parse();
    }

    let private_key = match keydir {
        // Load the key from the keydir.
        Some(keydir) => rejson::load_private_key(secrets_file, &keydir)?,
        // Read the key default keydir.
        None => rejson::load_private_key(secrets_file, DEFAULT_KEYDIR)?,
    };

    Ok(private_key)
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert()
}
