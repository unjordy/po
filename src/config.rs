use std::io::prelude::*;
use std::io;
use std::path;
use std::fs::File;
use rustc_serialize::json;
use regex::Regex;

#[derive(RustcEncodable, RustcDecodable)]
struct Config {
    token: String,
    user: String
}

#[derive(Debug, PartialEq)]
pub enum ReadError {
    NoConfig,
    JsonError,
    FileError(io::Error),
    InvalidApiToken(String),
    InvalidUserKey(String)
}

#[derive(Debug, PartialEq)]
pub enum WriteError {
    InvalidApiToken(String),
    InvalidUserKey(String),
    FileError(io::Error)
}

fn valid_token(token: &str) -> bool {
    let re = Regex::new(r"[A-Za-z0-9]").unwrap();
    token.len() == 30 && re.is_match(token)
}

pub fn read(path: &path::Path) -> Result<(String, String), ReadError> {
    let file = File::open(path);

    match file {
        Ok(mut f) => {
            let mut buf = String::new();
            match f.read_to_string(&mut buf) {
                Ok(_) => {
                    let config: Config = json::decode(&buf).unwrap();
                    Ok((config.token, config.user))
                },
                Err(e) => Err(ReadError::FileError(e))
            }
        },
        Err(_) => Err(ReadError::NoConfig)
    }
}

pub fn write(token: &str, user: &str,
             path: &path::Path) -> Result<(), WriteError> {
    if !valid_token(token) {
        Err(WriteError::InvalidApiToken(token.to_string()))
    }
    else if !valid_token(user) {
        Err(WriteError::InvalidUserKey(user.to_string()))
    }
    else {
        let config = Config {
            token: token.to_string(),
            user: user.to_string()
        };
        let config_json = json::encode(&config).unwrap();

        let file = File::create(path);
        match file {
            Ok(mut f) => {
                match f.write_all(config_json.into_bytes().as_slice()) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(WriteError::FileError(e))
                }
            },
            Err(e) => Err(WriteError::FileError(e))
        }
    }
}
