use std::fs;

use crate::{clipboard::copy_text, error::{ConfigError, DatabaseError}};
use rusqlite::Connection;
use directories::ProjectDirs;
use serde::{Deserialize,Serialize};

const CURRENT_DB_PATH: &str = "RVAULT_CURRENT_DB_PATH";
const CURRENT_VAULT_NAME: &str = "RVAULT_CURRENT_VAULT_NAME";

#[derive(Serialize,Deserialize,Debug)]
pub struct Config {
    version: f32,
    master_password_hash: Option<String>,
    default_vault: Option<String>

}
impl Default for Config {
    fn default() -> Self {
        Config { version: 0.1, master_password_hash: None, default_vault: None }
    }
}
impl Config {
    pub fn new() -> Result<Self,ConfigError>{
        if let Some(project_dirs) = ProjectDirs::from("io.github","ata-sesli","RVault"){
            let config_dir = project_dirs.config_dir();
            let _ = fs::create_dir_all(config_dir);
            let config_file_path = config_dir.join("config.json");
            if config_file_path.exists(){
                println!("Config file found. Loading...");
                let config_str = fs::read_to_string(config_file_path)?;
                let config: Config = serde_json::from_str(&config_str)?;
                Ok(config)
            }
            else {
                println!("No config file found. Creating a default one...");
                let config = Config::default();
                Ok(config)
            }
        }
        else {
            Err(ConfigError::Path)
        }
    }
    pub fn save_config(&self) -> Result<(),ConfigError> {
        if let Some(project_dirs) = ProjectDirs::from("io.github","ata-sesli","RVault"){
            let config_dir = project_dirs.config_dir();
            let _ = fs::create_dir_all(config_dir);
            let config_file_path = config_dir.join("config.json");
            let json_string = serde_json::to_string(&self)?;
            fs::write(config_file_path, json_string)?;
            Ok(())
        }
        else {
            Err(ConfigError::Path)
        }
    }
}
pub struct Database {
    connection: Connection
}
impl Database {
    pub fn new() -> Result<Self,DatabaseError>{
        if let Some(project_dirs) = ProjectDirs::from("io.github","ata-sesli","RVault"){
            let project_dirs = project_dirs.data_dir();
            let database_dir = project_dirs.join("databases");
            let _ = std::fs::create_dir_all(&database_dir)?;
            let final_path = database_dir.join("default_vault.sqlite");
            let db_exists = final_path.exists();
            let connection = Connection::open(&final_path)?;
            if db_exists {
                println!("Successfully opened existing database!");
            }
            else {
                println!("Successfully created new database!");
            }
            unsafe {
                std::env::set_var(CURRENT_DB_PATH, final_path);
            }
            Ok(Self { connection })
        }
        else {
            Err(DatabaseError::Path)
        }
    }
}

pub struct Table {
    table_name: String 
}
impl Table {
    pub fn new(db: &Database,table_name: Option<String>) -> Result<Self,DatabaseError>{
        let connection = &db.connection;
        let full_table_name = match table_name {
            Some(name) => {
                if Self::is_valid_identifier(&name){
                    println!("Error: Invalid table name '{}'", name);
                    return Err(DatabaseError::Sqlite(rusqlite::Error::InvalidParameterName(name)));
                } else {
                    name
                }
            },
            None => String::from("main")
        };
        let query = format!(
            "CREATE TABLE IF NOT EXISTS {} (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                platform TEXT NOT NULL,
                user_id TEXT NOT NULL,
                password TEXT NOT NULL
                )",
            full_table_name
        );
        match connection.execute(&query,[]){
            Ok(updated) => {
                println!("Table has been created successfully!");
                unsafe {
                    std::env::set_var(CURRENT_VAULT_NAME, "main");
                }
                Ok(Self {
                    table_name: full_table_name,
                })
            },
            Err(e) => {
                eprintln!("Update failed: {e}");
                Err(DatabaseError::Sqlite(rusqlite::Error::InvalidParameterName(full_table_name)))
            }   
        }
    }
    pub fn add_entry(&self,db: &Database, platform: String,id_and_password: String){
        let (user_id,password) = id_and_password.split_once(':').unwrap();
        let query = format!(
            "INSERT INTO {} (platform,user_id,password)
             VALUES (?1,?2,?3)
            ", &self.table_name
        );
        let _ = db.connection.execute(&query, [platform.to_string(),user_id.to_string(),password.to_string()]);
        println!("Account {} in {} has been added successfully!",user_id,platform);
    }
    pub fn remove_entry(&self,db: &Database, platform: String, user_id: String){
        let query = format!(
            "DELETE FROM {}
             WHERE platform = (?1) AND user_id = (?2)
            ",&self.table_name
        );
        let _ = db.connection.execute(&query, [platform.to_string(),user_id.to_string()]);
        println!("Account {} in {} has been removed successfully!",user_id,platform);
    }
    pub fn get_password(&self,db: &Database, platform: String, user_id: String) -> Result<(),DatabaseError>{
        let query: String = format!(
            "SELECT password FROM {}
             WHERE platform = (?1) AND user_id = (?2)
            ",&self.table_name
        );
        let password_result = db.connection.query_row(&query, [platform.to_string(),user_id.to_string()],
        |row| row.get::<_,String>(0));
        match password_result {
            Ok(password) => {
                let _ = copy_text(password);
                println!("Password has been copied! You can use it now.");
                Ok(())
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                eprintln!("Error: No entry found for user '{}' on platform '{}'.", user_id, platform);
                Err(DatabaseError::from(rusqlite::Error::QueryReturnedNoRows))
            }
            Err(e) => {
                eprintln!("Database query failed: {e}");
                Err(DatabaseError::from(e))
            }
        }
        
    }
    fn is_valid_identifier(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_')
}
}