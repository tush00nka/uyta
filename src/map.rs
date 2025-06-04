use raylib::prelude::*;
use serde::Deserialize;
use serde_json;
use std::{collections::HashMap, fs};

pub const MAP_WIDTH: usize = 8;
pub const MAP_HEIGHT: usize = 8;
pub const TILE_PIXEL_SIZE: i32 = 16;
pub const TILE_SCALE: i32 = 4;

pub const TILE_SIZE: i32 = TILE_PIXEL_SIZE * TILE_SCALE;

#[derive(Deserialize)]
pub struct Crop {
    // pub id: usize,
    #[allow(unused)]
    pub name: String,
    pub time_to_grow: usize,
    pub buy_price: usize,
    pub sell_price: usize,
}

#[derive(PartialEq)]
pub enum TileType {
    Grass,
    Farmland { crop: Option<usize>, stage: usize },
}

#[derive(Deserialize)]
pub struct Map {
    pub crops_data: Vec<Crop>,
    #[serde(skip_deserializing)]
    pub tiles: HashMap<(i32, i32), TileType>,
    #[serde(skip_deserializing)]
    pub occupation_map: HashMap<(i32, i32), bool>
}

impl Map {
    pub fn new() -> Self {
        let res = fs::read_to_string("static/crops.json");
        let s = match res {
            Ok(s) => s,
            Err(e) => panic!("Can't read file! {e}"),
        };

        let mut map: Self = serde_json::from_str(&s).expect("Can't parse json!");

        map.tiles = HashMap::new();
        map.occupation_map = HashMap::new();

        let half_width = MAP_WIDTH as i32 / 2;
        let half_height = MAP_HEIGHT as i32 / 2;

        for x in -half_width..half_width {
            for y in -half_height..half_height {
                map.tiles.insert((x, y), TileType::Grass);
            }
        }

        map
    }

    pub fn update_tiles(&mut self) {
        for tile in self.tiles.values_mut() {
            match tile {
                TileType::Farmland { crop, stage } => {
                    if crop.is_none() {
                        continue;
                    }

                    if *stage >= self.crops_data[crop.unwrap()].time_to_grow {
                        // wait for collect
                        continue;
                    }

                    // if !*watered {
                    //     continue;
                    // }

                    *stage += 1;
                    // *watered = false;
                }
                _ => {}
            }
        }
    }

    pub fn draw(&self, rl: &mut RaylibDrawHandle, textures: &HashMap<String, Texture2D>) {
        for (position, tile) in self.tiles.iter() {
            let texture_id = match tile {
                TileType::Grass => "grass",
                TileType::Farmland { .. } => "dirt",
            };

            let pixel_pos = Vector2::new(
                (position.0 * TILE_SIZE) as f32,
                (position.1 * TILE_SIZE) as f32,
            );
            rl.draw_texture_ex(
                textures.get(texture_id).unwrap(),
                pixel_pos,
                0.,
                TILE_SCALE as f32,
                Color::WHITE,
            );

            match tile {
                TileType::Farmland { crop, stage, .. } => {
                    if crop.is_none() {
                        continue;
                    }

                    let source = Rectangle::new(
                        *stage as f32 * TILE_PIXEL_SIZE as f32,
                        0.,
                        TILE_PIXEL_SIZE as f32,
                        TILE_PIXEL_SIZE as f32,
                    );
                    let destination = Rectangle::new(
                        (position.0 * TILE_SIZE) as f32,
                        (position.1 * TILE_SIZE) as f32,
                        TILE_SIZE as f32,
                        TILE_SIZE as f32,
                    );

                    let id: &str = &format!("crop{}", crop.unwrap());

                    rl.draw_texture_pro(
                        textures.get(id).unwrap_or(textures.get("error").unwrap()),
                        source,
                        destination,
                        Vector2::zero(),
                        0.,
                        Color::WHITE,
                    );
                }
                _ => {}
            }
        }
    }
}

impl Default for Map {
    fn default() -> Self {
        Self::new()
    }
}
