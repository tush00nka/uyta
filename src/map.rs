use itertools::Itertools;
use raylib::prelude::*;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::collections::HashMap;

use crate::{player::Player, utils::parse_json, worker::WorkerHandler};

pub const CHUNK_WIDTH: usize = 5;
pub const CHUNK_HEIGHT: usize = 5;
pub const TILE_PIXEL_SIZE: i32 = 16;
pub const TILE_SCALE: i32 = 4;

pub const TILE_SIZE: i32 = TILE_PIXEL_SIZE * TILE_SCALE;

#[derive(Deserialize)]
pub struct Crop {
    pub time_to_grow: usize,
    grow_step: usize,
    pub sell_price: usize,
    pub exp: usize,
}

#[derive(Deserialize)]
pub struct Tree {
    pub time_to_grow: usize,
    grow_step: usize,
    pub time_to_fruit: usize,
    pub sell_price: usize,
    pub exp: usize,
}

#[derive(PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize)]
pub enum TileType {
    Grass,
    Tree {
        tree: usize,
        grow: usize,
        stage: usize,
    },
    Farmland {
        crop: usize,
        stage: usize,
    },
}

#[derive(Deserialize)]
pub struct MapStaticData {
    pub crops_data: Vec<Crop>,
    pub tree_data: Vec<Tree>,
}

#[serde_as]
#[derive(Deserialize, Serialize)]
pub struct MapDynamicData {
    #[serde_as(as = "Vec<(_, _)>")]
    pub tiles: HashMap<(i32, i32), TileType>,
    #[serde_as(as = "Vec<(_, _)>")]
    pub occupation_map: HashMap<(i32, i32), bool>,
    #[serde_as(as = "Vec<(_, _)>")]
    land_expansion_points: Vec<(i32, i32)>,
    next_expansion_cost: usize,
}

pub struct Map {
    pub static_data: MapStaticData,
    pub dynamic_data: MapDynamicData,
}

impl Map {
    pub fn new() -> Self {
        let static_data: MapStaticData =
            parse_json("static/tiles.json").expect("Can't deserialize");
        let dynamic_data = parse_json("dynamic/map_save.json");

        let has_save_file = match dynamic_data {
            Ok(_) => true,
            Err(_) => false,
        };

        if has_save_file {
            return Self {
                static_data,
                dynamic_data: dynamic_data.unwrap(),
            };
        }

        let mut dynamic_data = MapDynamicData {
            tiles: HashMap::new(),
            occupation_map: HashMap::new(),
            land_expansion_points: vec![],
            next_expansion_cost: 1000,
        };

        let half_width = CHUNK_WIDTH as i32 / 2;
        let half_height = CHUNK_HEIGHT as i32 / 2;

        for x in -half_width..=half_width {
            for y in -half_height..=half_height {
                dynamic_data.tiles.insert((x, y), TileType::Grass);
            }
        }

        let directions = [(0, 1), (1, 0), (0, -1), (-1, 0)];

        for direction in directions {
            dynamic_data.land_expansion_points.push((
                direction.0 * CHUNK_WIDTH as i32,
                direction.1 * CHUNK_HEIGHT as i32,
            ));
        }

        Self {
            static_data,
            dynamic_data,
        }
    }

    pub fn update_tiles(&mut self) {
        for tile in self.dynamic_data.tiles.values_mut() {
            match tile {
                TileType::Farmland { crop, stage } => {
                    if *stage >= self.static_data.crops_data[*crop].time_to_grow {
                        // wait for collect
                        continue;
                    }

                    *stage += 1;
                }
                TileType::Tree { tree, grow, stage } => {
                    if *stage >= self.static_data.tree_data[*tree].time_to_fruit
                        && *grow >= self.static_data.tree_data[*tree].time_to_grow
                    {
                        continue;
                    }

                    if *grow >= self.static_data.tree_data[*tree].time_to_grow {
                        *stage += 1;
                        continue;
                    }

                    *grow += 1;
                }
                _ => {}
            }
        }
    }

    pub fn buy_land(&mut self, selected_tile: (i32, i32), player: &mut Player) {
        if player.money < self.dynamic_data.next_expansion_cost {
            return;
        }

        let mut index: Option<usize> = None;

        for expansion_point in self.dynamic_data.land_expansion_points.iter() {
            if selected_tile == *expansion_point {
                index = Some(
                    self.dynamic_data
                        .land_expansion_points
                        .iter()
                        .position(|current| *current == selected_tile)
                        .unwrap(),
                );
                break;
            }
        }

        if index.is_none() {
            return;
        }

        player.money -= self.dynamic_data.next_expansion_cost;
        self.dynamic_data.next_expansion_cost *= 3;

        let point = self.dynamic_data.land_expansion_points[index.unwrap()];
        self.dynamic_data
            .land_expansion_points
            .remove(index.unwrap());

        let neg_half_width = -(CHUNK_WIDTH as i32 / 2) + point.0;
        let pos_half_width = CHUNK_WIDTH as i32 / 2 + point.0;
        let neg_half_height = -(CHUNK_HEIGHT as i32 / 2) + point.1;
        let pos_half_height = CHUNK_HEIGHT as i32 / 2 + point.1;

        for x in neg_half_width..=pos_half_width {
            for y in neg_half_height..=pos_half_height {
                self.dynamic_data.tiles.insert((x, y), TileType::Grass);
            }
        }

        let directions = [(0, 1), (1, 0), (0, -1), (-1, 0)];

        for direction in directions {
            let position = (
                direction.0 * CHUNK_WIDTH as i32 + point.0,
                direction.1 * CHUNK_HEIGHT as i32 + point.1,
            );

            if self.dynamic_data.tiles.contains_key(&position)
                || self.dynamic_data.land_expansion_points.contains(&position)
            {
                continue;
            }

            self.dynamic_data.land_expansion_points.push(position);
        }
    }

    pub fn draw(
        &self,
        rl: &mut RaylibDrawHandle,
        textures: &HashMap<String, Texture2D>,
        worker_handler: &mut WorkerHandler,
        font: &Font,
    ) {
        let expansion_texture = textures.get("land_expansion").unwrap();

        for expansion_point in self.dynamic_data.land_expansion_points.iter() {
            rl.draw_texture_ex(
                expansion_texture,
                Vector2::new(
                    (expansion_point.0 * TILE_SIZE) as f32,
                    (expansion_point.1 * TILE_SIZE) as f32,
                ),
                0.,
                TILE_SCALE as f32,
                Color::WHITE,
            );
            rl.draw_text_ex(
                font,
                &format!("{}", self.dynamic_data.next_expansion_cost),
                Vector2::new(
                    (expansion_point.0 * TILE_SIZE
                        + self
                            .dynamic_data
                            .next_expansion_cost
                            .to_string()
                            .chars()
                            .count() as i32
                            * 2) as f32,
                    (expansion_point.1 * TILE_SIZE - TILE_SIZE / 3) as f32,
                ),
                24.,
                0.,
                Color::RAYWHITE,
            );
        }

        let border_texture = textures.get("borders").unwrap();

        for (position, tile) in self.dynamic_data.tiles.iter().sorted() {
            let texture_id = match tile {
                TileType::Grass => "grass",
                TileType::Tree { .. } => "grass",
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

            let directions = [(0, 1), (1, 0), (0, -1), (-1, 0)];

            for direction in directions {
                let pos = (position.0 + direction.0, position.1 + direction.1);
                if !self.dynamic_data.tiles.contains_key(&pos) {
                    rl.draw_texture_pro(
                        border_texture,
                        Rectangle::new(
                            (direction.0 * TILE_PIXEL_SIZE) as f32,
                            (direction.1 * TILE_PIXEL_SIZE) as f32,
                            TILE_PIXEL_SIZE as f32,
                            TILE_PIXEL_SIZE as f32,
                        ),
                        Rectangle::new(
                            (pos.0 * TILE_SIZE) as f32,
                            (pos.1 * TILE_SIZE) as f32,
                            TILE_SIZE as f32,
                            TILE_SIZE as f32,
                        ),
                        Vector2::zero(),
                        0.,
                        Color::WHITE,
                    );
                }
            }
        }

        // two loops bad, but better worker rendering
        for (position, tile) in self.dynamic_data.tiles.iter().sorted() {
            match tile {
                TileType::Farmland { crop, stage, .. } => {
                    let source = Rectangle::new(
                        (*stage / self.static_data.crops_data[*crop].grow_step) as f32
                            * TILE_PIXEL_SIZE as f32,
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
                TileType::Tree { tree, grow, stage } => {
                    let tree_data = &self.static_data.tree_data[*tree];

                    let offset = if *grow < tree_data.time_to_grow {
                        (*grow / tree_data.grow_step) as f32 * TILE_PIXEL_SIZE as f32
                    } else {
                        if *stage >= self.static_data.tree_data[*tree].time_to_fruit {
                            (tree_data.time_to_grow / tree_data.grow_step) as f32
                                * TILE_PIXEL_SIZE as f32
                        } else {
                            ((tree_data.time_to_grow - 1) / tree_data.grow_step) as f32
                                * TILE_PIXEL_SIZE as f32
                        }
                    };

                    let source = Rectangle::new(
                        offset,
                        0.,
                        TILE_PIXEL_SIZE as f32,
                        TILE_PIXEL_SIZE as f32 * 2.,
                    );
                    let destination = Rectangle::new(
                        (position.0 * TILE_SIZE) as f32,
                        (position.1 * TILE_SIZE - TILE_SIZE) as f32,
                        TILE_SIZE as f32,
                        TILE_SIZE as f32 * 2.,
                    );

                    let id = &format!("tree{}", tree);

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

            let worker_texture = textures.get("worker").unwrap();
            worker_handler.workers.iter_mut().for_each(|worker| {
                if worker.position == *position {
                    worker.draw(rl, worker_texture);
                }
            });
        }
    }

    pub fn save(&self) {
        let serialized = serde_json::to_string_pretty(&self.dynamic_data).expect("err");
        std::fs::write("dynamic/map_save.json", serialized)
            .expect("Couldn't write map data to json");
    }
}

impl Default for Map {
    fn default() -> Self {
        Self::new()
    }
}
