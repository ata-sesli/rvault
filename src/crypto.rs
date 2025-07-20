use clap::ValueEnum;
use rand::seq::{IndexedRandom, SliceRandom};
#[derive(Debug,Clone,ValueEnum)]
pub enum Encryption {
    Raw,
    
}
#[derive(Debug,Clone,ValueEnum)]
pub enum Hash {
    Raw,
}

pub fn generate_password(length: u8,special_characters: bool) -> String{
    let lowercase = b"abcdefghijklmnopqrstuvwxyz";
    let uppercase = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let numbers = b"0123456789";
    let symbols = b"!@#$%^&*()_+-=[]{}|;:'\",.<>/?";

    let mut rng = rand::rng();
    let mut password_chars = Vec::with_capacity(length as usize);
    let mut master_pool = Vec::new();

    master_pool.extend_from_slice(lowercase);
    password_chars.push(*lowercase.choose(&mut rng).unwrap() as char);

    master_pool.extend_from_slice(uppercase);
    password_chars.push(*uppercase.choose(&mut rng).unwrap() as char);

    master_pool.extend_from_slice(numbers);
    password_chars.push(*numbers.choose(&mut rng).unwrap() as char);

    if special_characters {
        master_pool.extend_from_slice(symbols);
        password_chars.push(*symbols.choose(&mut rng).unwrap() as char);
    }

    let remaining_len = length as usize - password_chars.len();
    for _ in 0..remaining_len {
        password_chars.push(*master_pool.choose(&mut rng).unwrap() as char);
    }
    password_chars.shuffle(&mut rng);
    password_chars.into_iter().collect::<String>()
}