use clap::{Parser, Subcommand};
use whisper::secure::{self, load_key_from_env};

#[derive(Parser)]
#[command(name = "encryptor")]
#[command(author = "Abel")]
#[command(version = "1.01")]
#[command(about = "Encrypt and decrypt text using AES-256-GCM", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Encrypt the given plaintext string")]
    Encrypt {
        /// The text to encrypt
        #[arg(short, long, help = "Plaintext string to be encrypted")]
        text: String,
    },
    #[command(about = "Decrypt the given ciphertext string")]
    Decrypt {
        /// The text to decrypt
        #[arg(short, long, help = "Ciphertext string to be decrypted")]
        text: String,
    },
}

fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let cli = Cli::parse();
    // Read key_bytes from envronment
    let key_bytes = match load_key_from_env("ENCRYPT_KEY") {
        Ok(key) => key,
        Err(e) => {
            eprintln!("Error - {e}");
            std::process::exit(1);
        }
    };

    match cli.command {
        // Read key
        Commands::Encrypt { text } => {
            let crypt_text = secure::aes::encrypt(&text, &key_bytes)?;
            println!("Encrypted text: {crypt_text}");
        }
        Commands::Decrypt { text } => {
            let decrypt_text = secure::aes::decrypt(&text, &key_bytes)?;
            println!("Decrypted text: {decrypt_text}");
        }
    }

    Ok(())
}
