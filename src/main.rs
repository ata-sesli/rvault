pub mod cli;
pub mod crypto;
pub mod storage;
pub mod error;
pub mod clipboard;
pub mod vault;
pub mod config;
pub mod session;
pub mod keystore;
// pub mod keystore_bin;
use cli::Cli;
use clap::Parser;
use storage::Database;
use cli::Commands;
use storage::Table;

use crate::keystore::keystore_path;

fn main() {
    let args = Cli::parse();
    let mut config = config::Config::new().unwrap();
    let stored_hash = config.master_password_hash.as_ref().unwrap();
    // The 'Setup' command is special and can be run at any time.
    let is_protected_command = match args.command {
        Commands::Setup {} => {
            if config.master_password_hash.is_some() {
                println!("⚠️ RVault has already been set up. To reset, delete your config file.");
                return;
            }
            println!("Setting up RVault for the first time...");
            let master_password = rpassword::prompt_password("Please create a master password: ").unwrap();
            let master_password_confirm = rpassword::prompt_password("Please confirm your master password: ").unwrap();
            // Get the stored hash from the config we loaded at the start
            if master_password != master_password_confirm {
                eprintln!("❌ Passwords do not match. Aborting setup.");
                return;
            }
            let hashed = crypto::hash_data(master_password.as_bytes()).map_err(|e| e.to_string()).unwrap();
            config.master_password_hash = Some(hashed.hash);
            config.save_config().unwrap();

            // create keystore file
            let path = keystore_path().unwrap();
            keystore::create_key_vault(&master_password, &path, &mut config)
                .map_err(|e| eprintln!("❌ Keystore create failed: {e}"))
                .ok();
            return;
        }
        Commands::Generate { length, special_characters } => {
            let final_password = crypto::generate_password(length, special_characters);
            clipboard::copy_text(final_password);
            println!("Generated password has been copied! You can use it now.");
            return;
        }
        Commands::Unlock {} => {
            let master_password = rpassword::prompt_password("Enter Master Password: ").unwrap();
            // Your existing logic for verifying the password and getting the key is correct.
            match vault::Vault::get_encryption_key(&master_password, stored_hash) {
                Ok(encryption_key) => {
                    match session::start_session(&encryption_key) {
                        Ok(token) => {
                            session::write_current(&token).expect("Failed to write current session file");
                            eprintln!("✅ Vault unlocked."); // Use eprintln for user messages
                        }
                        Err(e) => eprintln!("❌ Failed to start session: {}", e),
                    }
                }
                Err(e) => eprintln!("❌ Unlock failed: {}", e),
            }
            return;
        }
        Commands::Lock {} => {
            match session::end_session() {
                Ok(_) => {
                    println!("Vault has been locked.")
                },
                Err(e) => eprintln!("Error: {}", e),
            }
            return;
        }
        _ => true
    };
    // --- THE GUARD ---
    // If the command was not handled above, it's a protected command.
    // We must have a valid session key to continue.
    let ek = if is_protected_command {
        match session::get_key_from_session() {
            Ok(key) => key, // The key is valid, proceed.
            Err(e) => {
                eprintln!("❌ Error: {}", e);
                eprintln!("Please run 'rvault unlock' to start a session.");
                return; // Exit if the vault is locked.
            }
        }
    } else {
        // This case should not be reached, but we handle it safely.
        return;
    };
    match args.command {
        Commands::Create { vault_name } => {
            let db = storage::Database::new().unwrap();
            let _ = Table::new(&db, vault_name).unwrap();
        }
        Commands::Add { vault, platform, id_and_password } => {
            let db = storage::Database::new().unwrap();
            let table = Table::new(&db, vault).unwrap();
            table.add_entry_with_key(&db,&ek ,platform, id_and_password);
        }
        Commands::Remove { vault, platform, id } => {
            let db = storage::Database::new().unwrap();
            let table = Table::new(&db, vault).unwrap();
            table.remove_entry(&db, platform, id);
        }
        Commands::Get { vault, platform, id } => {
            let db = storage::Database::new().unwrap();
            let table = Table::new(&db, vault).unwrap();
            let _ = table.get_password_with_key(&db,&ek, platform, id);
        }
        _ => todo!()
    }
}