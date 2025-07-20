use clap::ValueEnum;

#[derive(Debug,Clone,ValueEnum)]
pub enum Encryption {
    Raw,
    
}
#[derive(Debug,Clone,ValueEnum)]
pub enum Hash {
    Raw,
}

fn generate_password(){}