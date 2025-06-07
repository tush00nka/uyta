use std::fs;
use serde::de::{self, Error};

pub fn parse_json<T: de::DeserializeOwned>(path: &str) -> Result<T, serde_json::Error> {
    let res = fs::read_to_string(path);
    match res {
        Ok(s) => return serde_json::from_str(&s),
        Err(_) => {
            return Result::Err(Error::custom("No such file"))
        },
    };
}
