mod cli;

use clap::Parser;
use crate::cli::{Cli, Commands};

// Import everything needed from the new library
use rvault_core::{
    clipboard, config, crypto, keystore, session, storage, vault,
    storage::{Database, Table},
};
use rvault_core::keystore::keystore_path; // Special case import for path

fn main() {
    let args = Cli::parse();
    // If no command is provided, launch TUI
    if args.command.is_none() {
        if let Err(e) = rvault_tui::run() {
            eprintln!("Application error: {}", e);
        }
        return;
    }
    let command = args.command.unwrap();

    let mut config = config::Config::new().unwrap();
    let stored_hash = config.master_password_hash.as_ref().unwrap();
    // The 'Setup' command is special and can be run at any time.
    let is_protected_command = match &command {
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
            keystore::create_key_vault(&master_password, &path)
                .map_err(|e| eprintln!("❌ Keystore create failed: {e}"))
                .ok();
            return;
        }
        Commands::Update {} => {
            println!("Checking for updates...");
            let status = self_update::backends::github::Update::configure()
                .repo_owner("ata-sesli")
                .repo_name("rvault")
                .bin_name("rvault")
                .show_download_progress(true)
                .show_output(true)
                .current_version(env!("CARGO_PKG_VERSION"))
                .build();
            
            match status {
                 Ok(update_builder) => {
                      match update_builder.update() {
                          Ok(status) => {
                               if status.updated() {
                                   println!("Update successful! Version {} is now installed.", status.version());
                               } else {
                                   println!("Already up to date.");
                               }
                          }
                          Err(e) => {
                               eprintln!("Update failed: {}", e);
                               eprintln!("Please ensure you have internet connection and the repository has releases.");
                          }
                      }
                 }
                 Err(e) => {
                      eprintln!("Configuration failed: {}", e);
                 }
            }
            return;
        }
        Commands::Generate { length, special_characters } => {
            let final_password = crypto::generate_password(*length, *special_characters);
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
    match command {
        Commands::Create { vault_name } => {
            let db = storage::Database::new().unwrap();
            let _ = Table::new(&db, vault_name).unwrap();
            println!("Storage created successfully!");
        }
        Commands::Add { vault, platform, id_and_password } => {
            let db = storage::Database::new().unwrap();
            if let Ok(table) = Table::new(&db, vault) {
                let (user_id, _) = id_and_password.split_once(':').unwrap();
                let user_id_owned = user_id.to_string();
                table.add_entry_with_key(&db,&ek ,platform.clone(), id_and_password);
                println!("Account {} in {} has been added successfully!", user_id_owned, platform);
            }
        }
        Commands::Remove { vault, platform, id } => {
            let db = storage::Database::new().unwrap();
            if let Ok(table) = Table::new(&db, vault) {
                 table.remove_entry(&db, platform.clone(), id.clone());
                 println!("Account {} in {} has been removed successfully!", id, platform);
            }
        }
        Commands::Get { vault, platform, id } => {
            let db = storage::Database::new().unwrap();
            if let Ok(table) = Table::new(&db, vault) {
                match table.get_password_with_key(&db,&ek, platform, id) {
                    Ok(_) => println!("Password has been copied! You can use it now."),
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
        }
        _ => todo!()
    }
}