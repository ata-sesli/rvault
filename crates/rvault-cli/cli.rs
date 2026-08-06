use clap::{Parser, Subcommand, ValueEnum};
/// RVault: A modern, secure password manager using encrypted local vaults.
#[derive(Debug, Parser)]
#[command(version, about = "Welcome to RVault!", author = "Ata Sesli")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Gets the password in the specified platform via id in the given vault and copies to the clipboard.
    /// If no vault is given, the pair will be added to the CURRENT_VAULT.
    /// Example Usage: rvault get instagram johndoe
    Get {
        #[arg(short, long)]
        vault: Option<String>,
        platform: String,
        id: String,
    },
    /// Adds id:password pair to the given vault for the given platform
    /// If no vault is given, the pair will be added to the CURRENT_VAULT.
    /// Example Usage: rvault add instagram johndoe:jd1234
    Add {
        #[arg(short, long)]
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
        #[arg(short, long)]
        vault: Option<String>,
        platform: String,
        id: String,
    },
    /// Creates a new vault with the given name.
    /// Example Usage: rvault create my_secret_vault
    Create { vault_name: Option<String> },
    /// Generates a random, unique password under the given constraints
    Generate {
        #[arg(short, long, default_value_t = 12)]
        length: u8,
        #[arg(short, long, default_value_t = false)]
        special_characters: bool,
    },
    /// Starts watching the clipboard and saves everything to the default 'clipboard' vault
    /// Example Usage: rvault watch
    Watch {},
    /// Stops watching the clipboard.
    /// Example Usage: ravult unwatch
    Unwatch {},
    /// Creates or restores encrypted RVault backup files.
    Backup {
        #[command(subcommand)]
        command: BackupCommands,
    },
    /// Prints this device's public RVault recipient code.
    Identity {},
    /// Exports selected entries to an encrypted binary file for a recipient.
    /// Example Usage: rvault export --to rvault1-abc --entry github ata --out github.rvault-export
    Export {
        #[arg(long)]
        to: String,
        #[arg(long, num_args = 2, value_names = ["PLATFORM", "USER_ID"])]
        entry: Option<Vec<String>>,
        #[arg(long, value_name = "PLATFORM:USER_ID")]
        selected: Vec<String>,
        #[arg(long)]
        out: String,
        #[arg(short, long)]
        vault: Option<String>,
    },
    /// Imports an encrypted RVault export file.
    /// Example Usage: rvault import gmail.rvault-export
    Import {
        path: String,
        #[arg(short, long)]
        vault: Option<String>,
        #[arg(long)]
        overwrite_all: bool,
        #[arg(long)]
        skip_all: bool,
    },
    /// Unlocks the vault in order to use it, prompts master password. It automatically locks after a certain amount of time.
    /// Example Usage: rvault unlock
    Unlock {},
    /// Locks the vault after using it. It automatically locks after a certain amount of time.
    /// Example Usage: rvault lock
    Lock {},
    /// Must run if user runs the app for the first time, it prompts and sets the master password.
    /// Example Usage: rvault setup
    Setup {},
    /// Enables or disables RVault browser integration.
    Browser {
        #[command(subcommand)]
        command: BrowserCommands,
    },
    /// Internal native messaging host commands.
    #[command(hide = true)]
    Host {
        #[command(subcommand)]
        command: HostCommands,
    },
}

#[derive(Debug, Subcommand)]
pub enum BackupCommands {
    /// Creates a full encrypted RVault backup file.
    Create {
        #[arg(long)]
        out: String,
    },
    /// Restores a full encrypted RVault backup file.
    Restore {
        path: String,
        #[arg(long)]
        yes: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum BrowserCommands {
    /// Enables RVault browser integration.
    Enable {
        #[arg(long, value_enum, default_value_t = Browser::Helium)]
        browser: Browser,
    },
    /// Disables RVault browser integration.
    Disable {
        #[arg(long, value_enum, default_value_t = Browser::Helium)]
        browser: Browser,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum Browser {
    Helium,
    Chrome,
    Chromium,
    Firefox,
}

#[derive(Debug, Subcommand)]
pub enum HostCommands {
    /// Runs stdio native messaging mode.
    Serve,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn browser_enable_parses_without_extension_id() {
        let cli = Cli::parse_from(["rvault", "browser", "enable"]);

        match cli.command {
            Some(Commands::Browser {
                command: BrowserCommands::Enable { browser },
            }) => assert_eq!(browser, Browser::Helium),
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn browser_enable_parses_explicit_browser() {
        let cli = Cli::parse_from(["rvault", "browser", "enable", "--browser", "firefox"]);

        match cli.command {
            Some(Commands::Browser {
                command: BrowserCommands::Enable { browser },
            }) => assert_eq!(browser, Browser::Firefox),
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn browser_disable_parses_without_extension_id() {
        let cli = Cli::parse_from(["rvault", "browser", "disable"]);

        match cli.command {
            Some(Commands::Browser {
                command: BrowserCommands::Disable { browser },
            }) => assert_eq!(browser, Browser::Helium),
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn host_start_is_not_a_cli_command() {
        let result = Cli::try_parse_from(["rvault", "host", "start"]);

        assert!(result.is_err());
    }

    #[test]
    fn backup_create_parses_output_path() {
        let cli = Cli::parse_from([
            "rvault",
            "backup",
            "create",
            "--out",
            "rvault.rvault-backup",
        ]);

        match cli.command {
            Some(Commands::Backup {
                command: BackupCommands::Create { out },
            }) => assert_eq!(out, "rvault.rvault-backup"),
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn backup_restore_parses_confirmation_flag() {
        let cli = Cli::parse_from([
            "rvault",
            "backup",
            "restore",
            "rvault.rvault-backup",
            "--yes",
        ]);

        match cli.command {
            Some(Commands::Backup {
                command: BackupCommands::Restore { path, yes },
            }) => {
                assert_eq!(path, "rvault.rvault-backup");
                assert!(yes);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn identity_parses_as_top_level_command() {
        let cli = Cli::parse_from(["rvault", "identity"]);

        assert!(matches!(cli.command, Some(Commands::Identity {})));
    }

    #[test]
    fn export_parses_recipient_entry_and_output() {
        let cli = Cli::parse_from([
            "rvault",
            "export",
            "--to",
            "rvault1-recipient",
            "--entry",
            "Gmail",
            "ata@example.com",
            "--out",
            "gmail.rvault-export",
        ]);

        match cli.command {
            Some(Commands::Export { to, entry, out, .. }) => {
                assert_eq!(to, "rvault1-recipient");
                assert_eq!(
                    entry,
                    Some(vec!["Gmail".to_string(), "ata@example.com".to_string()])
                );
                assert_eq!(out, "gmail.rvault-export");
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn import_parses_path_and_conflict_flags() {
        let cli = Cli::parse_from(["rvault", "import", "gmail.rvault-export", "--skip-all"]);

        match cli.command {
            Some(Commands::Import { path, skip_all, .. }) => {
                assert_eq!(path, "gmail.rvault-export");
                assert!(skip_all);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }
}
