use std::{collections::HashMap, fs};

use raylib::prelude::*;

pub struct TextureHandler {
    pub textures: HashMap<String, Texture2D>,
}

impl TextureHandler {
    pub fn new(rl: &mut RaylibHandle, thread: &RaylibThread) -> Self {
        let mut textures = HashMap::new();

        let filenames = fs::read_dir("static/textures/").unwrap();

        for filename in filenames {
            let file = match filename {
                Ok(f) => f,
                Err(e) => panic!("couldn't load this particular texture {e}"),
            };

            let name = file
                .file_name()
                .into_string()
                .unwrap()
                .split('.')
                .next()
                .unwrap()
                .to_string();

            let texture = rl
                .load_texture(&thread, file.path().to_str().unwrap())
                .unwrap();
            textures.insert(name, texture);
        }

        Self { textures }
    }
}
