use raylib::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

use crate::{player::Player, utils::parse_json};

pub const CHUNK_WIDTH: usize = 5;
pub const CHUNK_HEIGHT: usize = 5;
pub const TILE_PIXEL_SIZE: i32 = 16;
pub const TILE_SCALE: i32 = 4;

pub const TILE_SIZE: i32 = TILE_PIXEL_SIZE * TILE_SCALE;

#[derive(Deserialize)]
pub struct Crop {
    pub time_to_grow: usize,
    pub sell_price: usize,
}

#[derive(PartialEq)]
pub enum TileType {
    Grass,
    Farmland { crop: usize, stage: usize },
}

#[derive(Deserialize)]
pub struct Map {
    pub crops_data: Vec<Crop>,
    #[serde(skip_deserializing)]
    pub tiles: HashMap<(i32, i32), TileType>,
    #[serde(skip_deserializing)]
    pub occupation_map: HashMap<(i32, i32), bool>,
    #[serde(skip_deserializing)]
    land_expansion_points: Vec<(i32, i32)>,
    #[serde(skip_deserializing)]
    next_expansion_cost: usize,
}

impl Map {
    pub fn new() -> Self {
        let mut map: Self = parse_json("static/crops.json");

        map.tiles = HashMap::new();
        map.occupation_map = HashMap::new();
        map.next_expansion_cost = 1000;

        let half_width = CHUNK_WIDTH as i32 / 2;
        let half_height = CHUNK_HEIGHT as i32 / 2;

        for x in -half_width..=half_width {
            for y in -half_height..=half_height {
                map.tiles.insert((x, y), TileType::Grass);
            }
        }

        let directions = [(0, 1), (1, 0), (0, -1), (-1, 0)];

        for direction in directions {
            map.land_expansion_points.push((
                direction.0 * CHUNK_WIDTH as i32,
                direction.1 * CHUNK_HEIGHT as i32,
            ));
        }

        map
    }

    pub fn update_tiles(&mut self) {
        for tile in self.tiles.values_mut() {
            match tile {
                TileType::Farmland { crop, stage } => {
                    if *stage >= self.crops_data[*crop].time_to_grow {
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

    pub fn buy_land(&mut self, selected_tile: (i32, i32), player: &mut Player) {
        if player.money < self.next_expansion_cost {
            return;
        }

        player.money -= self.next_expansion_cost;
        self.next_expansion_cost *= 3;

        let mut index: Option<usize> = None;

        for expansion_point in self.land_expansion_points.iter() {
            if selected_tile == *expansion_point {
                index = Some(
                    self.land_expansion_points
                        .iter()
                        .position(|current| *current == *expansion_point)
                        .unwrap(),
                );
                break;
            }
        }

        if index.is_none() {
            return;
        }

        let point = self.land_expansion_points[index.unwrap()];
        self.land_expansion_points.remove(index.unwrap());

        let neg_half_width = -(CHUNK_WIDTH as i32 / 2) + point.0;
        let pos_half_width = CHUNK_WIDTH as i32 / 2 + point.0;
        let neg_half_height = -(CHUNK_HEIGHT as i32 / 2) + point.1;
        let pos_half_height = CHUNK_HEIGHT as i32 / 2 + point.1;

        for x in neg_half_width..=pos_half_width {
            for y in neg_half_height..=pos_half_height {
                self.tiles.insert((x, y), TileType::Grass);
            }
        }

        let directions = [(0, 1), (1, 0), (0, -1), (-1, 0)];

        for direction in directions {
            self.land_expansion_points.push((
                direction.0 * CHUNK_WIDTH as i32 + point.0,
                direction.1 * CHUNK_HEIGHT as i32 + point.1,
            ));
        }
    }

    pub fn draw(&self, rl: &mut RaylibDrawHandle, textures: &HashMap<String, Texture2D>) {
        for expansion_point in self.land_expansion_points.iter() {
            rl.draw_texture_ex(
                textures.get("land_expansion").unwrap(),
                Vector2::new(
                    (expansion_point.0 * TILE_SIZE) as f32,
                    (expansion_point.1 * TILE_SIZE) as f32,
                ),
                0.,
                TILE_SCALE as f32,
                Color::WHITE,
            );
            rl.draw_text(
                &format!("{} RUB", self.next_expansion_cost),
                expansion_point.0 * TILE_SIZE + TILE_SIZE,
                expansion_point.1 * TILE_SIZE,
                24,
                Color::RAYWHITE,
            );
        }

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

                    let id: &str = &format!("crop{}", crop);

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
