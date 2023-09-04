use std::{fs, io::Write};

use anyhow::Result;
use clap::{Parser, Subcommand};
use rejson::{self, KeyPair, SecretsFile};

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
        output: Option<String>,
    },

    /// Generate a new EJSON key pair.
    #[command(name = "keygen", alias = "g")]
    Generate {
        #[arg(env = "EJSON_KEYDIR", long)]
        keydir: Option<String>,

        /// Write the private key to the key dir.
        #[arg(short, long)]
        write: bool,
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
            output,
        } => decrypt(file, keydir, key_from_stdin, output),
        Commands::Generate { keydir, write } => generate(keydir, write),
    }
}

fn encrypt(files: Vec<String>) -> Result<()> {
    files.iter().try_for_each(|file_path| {
        let mut secrets_file = SecretsFile::load(file_path)?;
        secrets_file.transform(rejson::encrypt(&secrets_file)?)?;

        let json = secrets_file.to_string();
        let data = json.as_bytes();

        fs::write(file_path, data)?;
        println!("Wrote {} bytes to {}", data.len(), file_path);
        Ok(())
    })
}

fn decrypt(file: String, keydir: Option<String>, key_from_stdin: bool, _out: Option<String>) -> Result<()> {
    let mut secrets_file = SecretsFile::load(file)?;

    let private_key = match keydir {
        // Load the key from the keydir.
        Some(keydir) => rejson::load_private_key(&secrets_file, &keydir)?,
        // Read the key from stdin.
        None => {
            if key_from_stdin {
                let mut buffer = String::new();
                std::io::stdin().read_line(&mut buffer)?;
                buffer.trim().parse()?
            } else {
                return Err(anyhow::anyhow!("Either --keydir or --key-from-std must be supplied"));
            }
        }
    };

    secrets_file.transform(rejson::decrypt(&secrets_file, private_key)?)?;
    println!("{}", secrets_file);
    Ok(())
}

fn generate(keydir: Option<String>, write: bool) -> Result<()> {
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

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert()
}
