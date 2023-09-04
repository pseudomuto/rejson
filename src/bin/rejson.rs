use std::io::Write;

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
            key_from_stdin,
            output,
        } => decrypt(file, key_from_stdin, output),
        Commands::Generate { keydir, write } => generate(keydir, write),
    }
}

fn encrypt(files: Vec<String>) -> Result<()> {
    files.iter().try_for_each(|file_path| {
        let mut secrets_file = SecretsFile::load(file_path)?;
        secrets_file.transform(rejson::encrypt(&secrets_file)?)?;

        let json = secrets_file.to_string();
        let data = json.as_bytes();

        std::fs::write(file_path, data)?;
        println!("Wrote {} bytes to {}", data.len(), file_path);
        Ok(())
    })
}

fn decrypt(_file: String, _pk_stdin: bool, _out: Option<String>) -> Result<()> {
    unimplemented!("Not implemented yet")
}

fn generate(keydir: Option<String>, write: bool) -> Result<()> {
    let pair = KeyPair::generate().unwrap();
    println!("Public Key:");
    println!("{}", pair.public_key());

    if !write {
        println!("Private Key:");
        println!("{}", pair.private_key());
        return Ok(());
    }

    // TODO: Don't panic on missing keydir
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
