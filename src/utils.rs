use raylib::{
    RaylibHandle,
    window::{get_current_monitor, get_monitor_height, get_monitor_width},
};
use serde::de::{self, Error};
use std::fs;

pub fn parse_json<T: de::DeserializeOwned>(path: &str) -> Result<T, serde_json::Error> {
    let res = fs::read_to_string(path);
    match res {
        Ok(s) => return serde_json::from_str(&s),
        Err(_) => return Result::Err(Error::custom("No such file")),
    };
}

pub fn get_game_width(rl: &mut RaylibHandle) -> i32 {
    if rl.is_window_fullscreen() {
        get_monitor_width(get_current_monitor())
    } else {
        rl.get_screen_width()
    }
}

pub fn get_game_height(rl: &mut RaylibHandle) -> i32 {
    if rl.is_window_fullscreen() {
        get_monitor_height(get_current_monitor())
    } else {
        rl.get_screen_height()
    }
}
