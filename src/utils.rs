use crate::crypt::{dh::KeyPair, self};
use std::{fs,io::{self, Write}};

#[derive(Debug)]
pub enum AppError {
    Io(io::Error),
    Parse(std::num::ParseIntError),
}

impl From<io::Error> for AppError {
    fn from(e: io::Error) -> AppError {
        AppError::Io(e)
    }
}
impl From<std::num::ParseIntError> for AppError {
    fn from(e: std::num::ParseIntError) -> AppError {
        AppError::Parse(e)
    }
}

pub fn get_inp() -> Result<String, AppError> {
    let mut inp = String::new();
    io::stdin().read_line(&mut inp)?;
    inp = inp.trim().into();
    Ok(inp)
}

pub fn read_key(path: &str) -> Result<KeyPair, AppError> {
    let f = fs::read_to_string(path)?;
    let mut lines = f.lines();
    let private = lines.next().unwrap().trim().parse()?;
    let public = lines.next().unwrap().trim().parse()?;
    let modulus = lines.next().unwrap().trim().parse()?;
    let gen = lines.next().unwrap().trim().parse()?;

    Ok(KeyPair {private, public, modulus, gen})
}

pub fn save_key(path: &str, key: &KeyPair) -> Result<(), AppError>{
    let mut f = fs::File::create(path)?;
    let content = format!("{}\n{}\n{}\n{}",
                              key.private.to_string(), key.public.to_string(),
                              key.modulus.to_string(), key.gen.to_string());
    f.write(content.as_bytes())?;

    Ok(())
}

pub fn customize_and_generate_key() -> KeyPair {
    println!("What prime modulus do you want to use? (leave blank for 2315981)");
    let mod_s = get_inp().unwrap();
    let mut modulus = 2315981;
    if mod_s != "" { modulus = mod_s.parse().unwrap(); }

    println!("What generator do you want to use? (leave blank for 772197)");
    let gen_s = get_inp().unwrap();
    let mut generator = 772197;
    if gen_s != "" { generator = gen_s.parse().unwrap(); }

    crypt::dh::gen_key_pair(modulus, generator)
}
