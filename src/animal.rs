use std::collections::HashMap;

use raylib::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    map::{Map, TILE_PIXEL_SIZE, TILE_SIZE, TileType},
    utils::parse_json,
};

#[derive(Serialize, Deserialize)]
pub struct Animal {
    animal_type: usize,
    drop_bar: usize,
    pub position: (i32, i32),
    display_position: (f32, f32),
    #[serde(skip_serializing, skip_deserializing)]
    direction: (i32, i32),
}

#[derive(Deserialize)]
pub struct AnimalData {
    time_to_drop: usize,
    pub drop_cost: usize,
}

#[derive(Deserialize)]
pub struct AnimalStatic {
    pub animal_data: Vec<AnimalData>,
}

#[derive(Serialize, Deserialize)]
pub struct AnimalDynamic {
    pub animals: Vec<Animal>,
}

pub struct AnimalHandler {
    pub static_data: AnimalStatic,
    pub dynamic_data: AnimalDynamic,
}

impl AnimalHandler {
    pub fn new() -> Self {
        let res = parse_json("dynamic/animals_save.json");

        let static_data = parse_json("static/animals.json").expect("no animals??");

        match res {
            Ok(dynamic_data) => Self {
                static_data,
                dynamic_data,
            },
            Err(_) => {
                println!("no animal");

                Self {
                    static_data,
                    dynamic_data: AnimalDynamic { animals: vec![] },
                }
            }
        }
    }

    pub fn add_animal(&mut self, animal: Animal) {
        self.dynamic_data.animals.push(animal);
    }

    pub fn move_animals(&mut self, map: &mut Map) {
        for animal in self.dynamic_data.animals.iter_mut() {
            animal.move_randomly(
                map,
                &self.static_data.animal_data[animal.animal_type as usize],
            );
        }
    }

    pub fn save(&self) {
        let serialized =
            serde_json::to_string_pretty(&self.dynamic_data).expect("couldn't save animals data");
        std::fs::write("dynamic/animals_save.json", serialized)
            .expect("Couldn't write map data to json file");
    }
}

impl Animal {
    pub fn new(animal_type: usize, x: i32, y: i32) -> Self {
        Self {
            animal_type,
            drop_bar: 0,
            position: (x, y),
            display_position: ((x * TILE_SIZE) as f32, (y * TILE_SIZE) as f32),
            direction: (0, 1),
        }
    }

    fn move_randomly(&mut self, map: &mut Map, animal_data: &AnimalData) {
        let directions = [(0, 1), (1, 0), (0, -1), (-1, 0)];
        self.direction = directions[rand::random_range(0..4)];

        let new_pos = (
            self.position.0 + self.direction.0,
            self.position.1 + self.direction.1,
        );

        if !map.dynamic_data.tiles.contains_key(&new_pos) {
            return;
        }

        if *map.dynamic_data.tiles.get(&new_pos).unwrap() != TileType::Grass {
            return;
        }

        self.drop_bar += 1;
        if self.drop_bar >= animal_data.time_to_drop {
            self.drop_bar = 0;
            map.dynamic_data.tiles.insert(
                new_pos,
                TileType::AnimalDrop {
                    animal: self.animal_type,
                },
            );
            return;
        }

        self.position = new_pos;
    }

    pub fn draw(&mut self, rl: &mut RaylibDrawHandle, textures: &HashMap<String, Texture2D>) {
        let pixel_position = Vector2::new(
            (self.position.0 * TILE_SIZE) as f32,
            (self.position.1 * TILE_SIZE) as f32,
        );

        self.display_position.0 = lerp(
            self.display_position.0,
            pixel_position.x,
            10. * rl.get_frame_time(),
        );
        self.display_position.1 = lerp(
            self.display_position.1,
            pixel_position.y,
            10. * rl.get_frame_time(),
        );

        let pixel_position = Vector2::new(self.display_position.0, self.display_position.1);

        let texture_index = match self.direction {
            (0, 1) => 0,
            (0, -1) => 1,
            (1, 0) => 2,
            (-1, 0) => 3,
            _ => 4,
        };

        let source = Rectangle {
            x: (texture_index * TILE_PIXEL_SIZE) as f32,
            y: 0.,
            width: TILE_PIXEL_SIZE as f32,
            height: TILE_PIXEL_SIZE as f32,
        };

        let destination = Rectangle {
            x: pixel_position.x,
            y: pixel_position.y,
            width: TILE_SIZE as f32,
            height: TILE_SIZE as f32,
        };

        rl.draw_texture_pro(
            textures
                .get(&format!("animal{}", self.animal_type as usize))
                .unwrap(),
            source,
            destination,
            Vector2::zero(),
            0.,
            Color::WHITE,
        );
    }
}
