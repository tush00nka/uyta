use std::fs;
use serde::de;

pub fn parse_json<T: de::DeserializeOwned>(path: &str) -> T {
    let res = fs::read_to_string(path);
    let s= match res {
        Ok(s) => s,
        Err(e) => panic!("Can't read file! {e}"),
    };

    serde_json::from_str(&s).expect("Can't parse json!")
}
