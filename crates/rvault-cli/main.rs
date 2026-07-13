mod cli;
mod extension_api;
mod host;
mod native;

use crate::cli::{BackupCommands, Cli, Commands};
use clap::Parser;
use std::{
    fs,
    io::{self, Write},
    path::Path,
};

// Import everything needed from the new library
use rvault_core::keystore::keystore_path;
use rvault_core::{
    SecretKey, SessionKey, backup, clipboard, config, crypto, identity, keystore, portable_export,
    session, storage,
    storage::{EntryRepository, EntrySelector, NewEntry, Table},
    vault,
}; // Special case import for path

fn main() {
    let first_arg = std::env::args().nth(1);
    if host::is_native_messaging_launch(first_arg.as_deref()) {
        if let Err(e) = native::serve_stdio() {
            eprintln!("RVault native host error: {e}");
        }
        return;
    }

    let args = Cli::parse();
    // If no command is provided, launch TUI
    if args.command.is_none() {
        if let Err(e) = rvault_tui::run() {
            eprintln!("Application error: {}", e);
        }
        return;
    }
    let command = args.command.unwrap();

    if let Commands::Browser { command } = &command {
        if let Err(e) = host::handle_browser_command(command) {
            eprintln!("Error: {e}");
        }
        return;
    }

    if let Commands::Host { command } = &command {
        if let Err(e) = host::handle_host_command(command) {
            eprintln!("Error: {e}");
        }
        return;
    }

    let mut config = config::Config::new().unwrap();
    // The 'Setup' command is special and can be run at any time.
    let is_protected_command = match &command {
        Commands::Setup {} => {
            if config.master_password_hash.is_some() {
                println!("⚠️ RVault has already been set up. To reset, delete your config file.");
                return;
            }
            println!("Setting up RVault for the first time...");
            let master_password =
                rpassword::prompt_password("Please create a master password: ").unwrap();
            let master_password_confirm =
                rpassword::prompt_password("Please confirm your master password: ").unwrap();
            // Get the stored hash from the config we loaded at the start
            if master_password != master_password_confirm {
                eprintln!("❌ Passwords do not match. Aborting setup.");
                return;
            }
            let hashed = crypto::hash_data(master_password.as_bytes())
                .map_err(|e| e.to_string())
                .unwrap();
            config.master_password_hash = Some(hashed.hash);
            config.save_config().unwrap();

            // create keystore file
            let path = keystore_path().unwrap();
            keystore::create_key_vault(&master_password, &path)
                .map_err(|e| eprintln!("❌ Keystore create failed: {e}"))
                .ok();
            return;
        }

        Commands::Generate {
            length,
            special_characters,
        } => {
            match crypto::try_generate_password(*length, *special_characters) {
                Ok(final_password) => {
                    clipboard::copy_text(final_password);
                    println!("Generated password has been copied! You can use it now.");
                }
                Err(error) => eprintln!("Error: {error}"),
            }
            return;
        }
        Commands::Unlock {} => {
            let master_password = rpassword::prompt_password("Enter Master Password: ").unwrap();
            let Some(stored_hash) = config.master_password_hash.as_ref() else {
                eprintln!("❌ RVault has not been set up. Please run 'rvault setup' first.");
                return;
            };
            // Your existing logic for verifying the password and getting the key is correct.
            match vault::Vault::get_encryption_key(&master_password, stored_hash) {
                Ok(encryption_key) => {
                    match session::start_session(&encryption_key) {
                        Ok(token) => {
                            session::write_current(&token)
                                .expect("Failed to write current session file");
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
                }
                Err(e) => eprintln!("Error: {}", e),
            }
            return;
        }
        Commands::Backup { command } => {
            handle_backup_command(command, &config);
            return;
        }
        Commands::Browser { .. } | Commands::Host { .. } => false,
        _ => true,
    };
    // --- THE GUARD ---
    // If the command was not handled above, it's a protected command.
    // We must have a valid session key to continue.
    let ek = if is_protected_command {
        match SessionKey::load() {
            Ok(key) => key, // The key is valid, proceed.
            Err(e) => {
                eprintln!("❌ Error: {}", e);
                if config.master_password_hash.is_none() {
                    eprintln!("Please run 'rvault setup' first.");
                } else {
                    eprintln!("Please run 'rvault unlock' to start a session.");
                }
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
            let _ = EntryRepository::new(&db, vault_name).unwrap();
            println!("Storage created successfully!");
        }
        Commands::Add {
            vault,
            platform,
            id_and_password,
        } => {
            let db = storage::Database::new().unwrap();
            if let Ok(repository) = EntryRepository::new(&db, vault) {
                let Some((user_id, password)) = id_and_password.split_once(':') else {
                    eprintln!("Error: entry must use USER_ID:PASSWORD format");
                    return;
                };
                let user_id_owned = user_id.to_string();
                match repository.add(&ek, NewEntry::new(&platform, user_id, password.as_bytes())) {
                    Ok(()) => println!(
                        "Account {} in {} has been added successfully!",
                        user_id_owned, platform
                    ),
                    Err(error) => eprintln!("Error: {error}"),
                }
            }
        }
        Commands::Remove {
            vault,
            platform,
            id,
        } => {
            let db = storage::Database::new().unwrap();
            if let Ok(repository) = EntryRepository::new(&db, vault) {
                match repository.remove(EntrySelector::new(&platform, &id)) {
                    Ok(()) => println!(
                        "Account {} in {} has been removed successfully!",
                        id, platform
                    ),
                    Err(error) => eprintln!("Error: {error}"),
                }
            }
        }
        Commands::Get {
            vault,
            platform,
            id,
        } => {
            let db = storage::Database::new().unwrap();
            if let Ok(repository) = EntryRepository::new(&db, vault) {
                match repository.get(&ek, EntrySelector::new(&platform, &id)) {
                    Ok(entry) => match std::str::from_utf8(entry.secret.expose()) {
                        Ok(password) => {
                            clipboard::copy_text(password.to_string());
                            println!("Password has been copied! You can use it now.");
                        }
                        Err(error) => eprintln!("Error: {error}"),
                    },
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
        }
        Commands::Identity {} => match identity::load_or_create_identity(ek.as_bytes()) {
            Ok(identity) => println!("{}", identity::public_code_from_key(&identity.public_key)),
            Err(e) => eprintln!("Error: {e}"),
        },
        Commands::Export {
            to,
            entry,
            selected,
            out,
            vault,
        } => match collect_export_selectors(entry, selected) {
            Ok(selectors) => {
                let db = storage::Database::new().unwrap();
                match EntryRepository::new(&db, vault) {
                    Ok(repository) => match build_export_entries(&repository, &ek, &selectors) {
                        Ok(entries) => match portable_export::create_export_bytes(&to, &entries) {
                            Ok(bytes) => match fs::write(&out, bytes) {
                                Ok(_) => println!("Encrypted export written to {out}"),
                                Err(e) => eprintln!("Error writing export: {e}"),
                            },
                            Err(e) => eprintln!("Error creating export: {e}"),
                        },
                        Err(e) => eprintln!("Error reading entries: {e}"),
                    },
                    Err(e) => eprintln!("Error opening vault: {e}"),
                }
            }
            Err(e) => eprintln!("Error: {e}"),
        },
        Commands::Import {
            path,
            vault,
            overwrite_all,
            skip_all,
        } => {
            if overwrite_all && skip_all {
                eprintln!("Error: --overwrite-all and --skip-all cannot be used together.");
                return;
            }
            let db = storage::Database::new().unwrap();
            match Table::new(&db, vault) {
                Ok(table) => {
                    match import_entries_from_file(&db, &table, &ek, &path, overwrite_all, skip_all)
                    {
                        Ok((imported, skipped)) => {
                            println!("Imported {imported} entries. Skipped {skipped} entries.")
                        }
                        Err(e) => eprintln!("Error importing export: {e}"),
                    }
                }
                Err(e) => eprintln!("Error opening vault: {e}"),
            }
        }
        _ => todo!(),
    }
}

fn handle_backup_command(command: &BackupCommands, config: &config::Config) {
    match command {
        BackupCommands::Create { out } => {
            let Some(stored_hash) = config.master_password_hash.as_deref() else {
                eprintln!("❌ RVault has not been set up. Please run 'rvault setup' first.");
                return;
            };
            let master_password =
                rpassword::prompt_password("Enter Master Password for backup: ").unwrap();
            if let Err(e) = vault::Vault::get_encryption_key(&master_password, stored_hash) {
                eprintln!("❌ Backup failed: {e}");
                return;
            }
            match backup::create_backup_file(&master_password, Path::new(out)) {
                Ok(_) => println!("Encrypted backup written to {out}"),
                Err(e) => eprintln!("❌ Backup failed: {e}"),
            }
        }
        BackupCommands::Restore { path, yes } => {
            if !yes && !confirm_restore() {
                println!("Restore cancelled.");
                return;
            }
            let master_password = rpassword::prompt_password("Enter backup password: ").unwrap();
            match backup::restore_backup_file(&master_password, Path::new(path)) {
                Ok(_) => println!("Backup restored. RVault local data was replaced."),
                Err(e) => eprintln!("❌ Restore failed: {e}"),
            }
        }
    }
}

fn confirm_restore() -> bool {
    print!("This will replace local RVault data. Type RESTORE to continue: ");
    let _ = io::stdout().flush();
    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return false;
    }
    input.trim() == "RESTORE"
}

fn collect_export_selectors(
    entry: Option<Vec<String>>,
    selected: Vec<String>,
) -> Result<Vec<(String, String)>, String> {
    let mut selectors = Vec::new();
    if let Some(entry) = entry {
        if entry.len() != 2 {
            return Err("--entry requires PLATFORM and USER_ID".to_string());
        }
        selectors.push((entry[0].clone(), entry[1].clone()));
    }

    for selector in selected {
        let (platform, user_id) = selector
            .split_once(':')
            .ok_or_else(|| format!("invalid selector '{selector}', expected PLATFORM:USER_ID"))?;
        selectors.push((platform.to_string(), user_id.to_string()));
    }

    if selectors.is_empty() {
        return Err(
            "nothing selected; pass --entry PLATFORM USER_ID or --selected PLATFORM:USER_ID"
                .to_string(),
        );
    }

    Ok(selectors)
}

fn build_export_entries(
    repository: &EntryRepository<'_>,
    encryption_key: &SecretKey,
    selectors: &[(String, String)],
) -> Result<Vec<portable_export::ExportEntry>, String> {
    selectors
        .iter()
        .map(|(platform, user_id)| {
            let entry = repository
                .get(encryption_key, EntrySelector::new(platform, user_id))
                .map_err(|e| e.to_string())?;
            let password = std::str::from_utf8(entry.secret.expose())
                .map_err(|error| error.to_string())?
                .to_string();
            Ok(portable_export::ExportEntry {
                platform: platform.clone(),
                user_id: user_id.clone(),
                password,
                pinned: entry.metadata.pinned,
                created_at: entry.metadata.created_at,
                updated_at: entry.metadata.updated_at,
            })
        })
        .collect()
}

fn import_entries_from_file(
    db: &storage::Database,
    table: &Table,
    encryption_key: &SecretKey,
    path: &str,
    overwrite_all: bool,
    skip_all: bool,
) -> Result<(usize, usize), String> {
    let identity = identity::load_or_create_identity(encryption_key.as_bytes())?;
    let bytes = fs::read(path).map_err(|e| format!("read export: {e}"))?;
    let entries = portable_export::decrypt_export_bytes(&identity, &bytes)?;
    let mut imported = 0;
    let mut skipped = 0;

    for entry in entries {
        let exists = table
            .entry_exists(db, &entry.platform, &entry.user_id)
            .map_err(|e| e.to_string())?;
        let should_import = if exists {
            if skip_all {
                false
            } else if overwrite_all {
                true
            } else {
                match prompt_import_conflict(&entry)? {
                    ImportChoice::Overwrite => true,
                    ImportChoice::Skip => false,
                    ImportChoice::Cancel => return Err("import cancelled".to_string()),
                }
            }
        } else {
            true
        };

        if should_import {
            table
                .import_entry_with_key_result(db, encryption_key.as_bytes(), &entry)
                .map_err(|e| e.to_string())?;
            imported += 1;
        } else {
            skipped += 1;
        }
    }

    Ok((imported, skipped))
}

enum ImportChoice {
    Overwrite,
    Skip,
    Cancel,
}

fn prompt_import_conflict(entry: &portable_export::ExportEntry) -> Result<ImportChoice, String> {
    loop {
        print!(
            "Entry {} / {} already exists. [o]verwrite, [s]kip, [c]ancel: ",
            entry.platform, entry.user_id
        );
        let _ = io::stdout().flush();
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|e| format!("read conflict choice: {e}"))?;
        match input.trim().to_lowercase().as_str() {
            "o" | "overwrite" => return Ok(ImportChoice::Overwrite),
            "s" | "skip" => return Ok(ImportChoice::Skip),
            "c" | "cancel" => return Ok(ImportChoice::Cancel),
            _ => eprintln!("Please enter o, s, or c."),
        }
    }
}
