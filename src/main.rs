pub mod cli;
pub mod crypto;
pub mod storage;
pub mod error;
pub mod account;
pub mod clipboard;
use cli::Cli;
use clap::Parser;
use storage::Database;
use cli::Commands;
use storage::Table;
fn main() {
    let args = Cli::parse();
    println!("{:?}",args.command);
    match args.command {
        Commands::Create { vault_name } => {
            let db = storage::Database::new().unwrap();
            let _ = Table::new(&db, vault_name).unwrap();
        }
        Commands::Add { vault, platform, id_and_password } => {
            let db = storage::Database::new().unwrap();
            let table = Table::new(&db, vault).unwrap();
            table.add_entry(&db, platform, id_and_password);
        }
        Commands::Remove { vault, platform, id } => {
            let db = storage::Database::new().unwrap();
            let table = Table::new(&db, vault).unwrap();
            table.remove_entry(&db, platform, id);
        }
        Commands::Get { vault, platform, id } => {
            let db = storage::Database::new().unwrap();
            let table = Table::new(&db, vault).unwrap();
            let _ = table.get_password(&db, platform, id);
        }
        Commands::Generate { length, special_characters } => {
            let final_password = crypto::generate_password(length, special_characters);
            clipboard::copy_text(final_password);
            println!("Generated password has been copied! You can use it now.");
        }   
        _ => todo!()
    }
}