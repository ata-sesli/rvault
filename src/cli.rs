use clap::{Parser,Subcommand};
use crate::crypto::Encryption;
/// RVault: A modern, secure password manager using encrypted local vaults.
#[derive(Debug,Parser)]
#[command(version,about = "Welcome to RVault!",author = "Ata Sesli")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}
#[derive(Debug,Subcommand)]
pub enum Commands {
    /// Gets the password in the specified platform via id in the given vault and copies to the clipboard.
    /// If no vault is given, the pair will be added to the CURRENT_VAULT.
    /// Example Usage: rvault get instagram johndoe
    Get {
        #[arg(short,long)]
        vault: Option<String>,
        platform: String,
        id: String,
    },
    /// Adds id:password pair to the given vault for the given platform
    /// If no vault is given, the pair will be added to the CURRENT_VAULT.
    /// Example Usage: rvault add instagram johndoe:jd1234
    Add {
        #[arg(short,long)]
        vault: Option<String>,
        platform: String,
        id_and_password: String,
    },
    /// Updates the password in the specified platform via id in the given vault
    /// If no vault is given, the pair will be added to the CURRENT_VAULT.
    /// Example Usage: rvault update instagram johndoe:4321jd
    /// 
    /// Removes the id:password pair in the given vault for the given platform via id
    /// If no vault is given, the pair will be removed from the CURRENT_VAULT.
    /// Example Usage: rvault remove instagram johndoe
    Remove {
        #[arg(short,long)]
        vault: Option<String>,
        platform: String,
        id: String,
    },
    /// Creates a new vault with the given name.
    /// Example Usage: rvault create my_secret_vault
    Create {
        vault_name: Option<String>
    },
    /// Generates a random, unique password under the given constraints
    Generate {
        #[arg(short,long,default_value_t=12)]
        length: u8,
        #[arg(short,long,default_value_t=false)]
        special_characters: bool,
    },
    /// Starts watching the clipboard and saves everything to the default 'clipboard' vault
    /// Example Usage: rvault watch
    Watch {},
    /// Stops watching the clipboard.
    /// Example Usage: ravult unwatch
    Unwatch {},
    /// Exports the specified vault to an encrypted file in the given path.
    /// Example Usage: rvault export my_secret_vault ./
    Export {
        vault_name: String,
        path: String,
        #[arg(short,long)]
        encryption: Option<Encryption>
    }
}