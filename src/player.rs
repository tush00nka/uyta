use std::collections::HashMap;

use raylib::{audio::Sound, prelude::*};

use crate::{
    map::{Map, TileType},
    ui::{Canvas, MenuMode},
    worker::Worker,
};

pub struct Player {
    pub money: usize,
    display_money: usize,
    pub level: usize,
    pub exp: usize,
    exp_to_lvl_up: usize,
}

impl Player {
    pub fn new() -> Self {
        Self {
            money: 100,
            display_money: 0,
            level: 1,
            exp: 0,
            exp_to_lvl_up: 20,
        }
    }

    pub fn update_money(&mut self) {
        let money_diff = self.money as isize - self.display_money as isize;
        if money_diff == 0 {
            return;
        }

        self.display_money = (self.display_money as isize + money_diff / money_diff.abs()) as usize;
    }

    pub fn update_exp(&mut self, sounds: &HashMap<String, Sound<'_>>) {
        if self.exp >= self.exp_to_lvl_up {
            self.level += 1;
            self.exp = 0;
            self.exp_to_lvl_up *= 3;
            sounds.get("level_up").unwrap().play();
        }
    }

    pub fn draw_stats(&self, rl: &mut RaylibDrawHandle, font: &Font) {
        rl.draw_rectangle(10, 10, 130, 28, Color::BLACK.alpha(0.5));
        rl.draw_text_ex(
            font,
            &format!("{}", self.display_money),
            Vector2::new(14., 14.),
            24.,
            0.,
            Color::WHITE,
        );

        let exp_bar_fill = self.exp as f32 / self.exp_to_lvl_up as f32;
        rl.draw_rectangle(
            rl.get_screen_width() / 4,
            10,
            rl.get_screen_width() / 2,
            24,
            Color::BLACK.alpha(0.5),
        );
        rl.draw_rectangle(
            rl.get_screen_width() / 4,
            10,
            (exp_bar_fill * (rl.get_screen_width() / 2) as f32) as i32,
            24,
            Color::DARKORANGE,
        );
        rl.draw_text_ex(
            font,
            &format!("Уровень {}", self.level),
            Vector2::new(rl.get_screen_width() as f32 / 4. + 10., 10.),
            24.,
            0.,
            Color::WHITE,
        );
    }

    pub fn plant_crops(&mut self, canvas: &Canvas, map: &mut Map, selected_tile: &(i32, i32)) {
        let Some(tile) = map.tiles.get_mut(selected_tile) else {
            return;
        };

        if let Some(occ_tile) = map.occupation_map.get_mut(selected_tile) {
            *occ_tile = false;
        }

        match tile {
            TileType::Grass => {
                let price = canvas.toolbar_data.crops[canvas.selected].price;
                if self.money >= price {
                    self.money -= price;
                    *tile = TileType::Farmland {
                        crop: canvas.selected,
                        stage: 0,
                    };
                }
            }
            TileType::Farmland { crop, stage } => {
                if canvas.mode != MenuMode::Crops {
                    return;
                }

                if *crop != canvas.selected {
                    let price = canvas.toolbar_data.crops[canvas.selected].price;
                    if self.money >= price {
                        self.money -= price;
                        // plant the seed
                        *crop = canvas.selected;
                        *stage = 0;
                    }
                }
            }
            TileType::Tree { .. } => {}
        }
    }

    pub fn plant_trees(&mut self, canvas: &Canvas, map: &mut Map, selected_tile: &(i32, i32)) {
        let Some(tile) = map.tiles.get_mut(selected_tile) else {
            return;
        };

        if let Some(occ_tile) = map.occupation_map.get_mut(selected_tile) {
            *occ_tile = false;
        }

        match tile {
            TileType::Grass => {
                let price = canvas.toolbar_data.trees[canvas.selected].price;
                if self.money >= price {
                    self.money -= price;
                    *tile = TileType::Tree {
                        tree: canvas.selected,
                        grow: 0,
                        stage: 0,
                    };
                }
            }
            _ => {}
        }
    }

    pub fn perform_misc(
        &mut self,
        canvas: &Canvas,
        workers: &mut Vec<Worker>,
        selected_tile: &(i32, i32),
        map: &mut Map,
    ) {
        if canvas.selected == 0 && self.money >= 100 && map.tiles.contains_key(selected_tile) {
            workers.push(Worker::new(workers.len(), selected_tile.0, selected_tile.1));
            self.money -= canvas.toolbar_data.misc[canvas.selected].price;
        }

        if canvas.selected == 1 {
            let Some(tile) = map.tiles.get_mut(&selected_tile) else {
                return;
            };

            if let Some(occ_tile) = map.occupation_map.get_mut(&selected_tile) {
                *occ_tile = false;
            }

            match tile {
                TileType::Grass => {}
                TileType::Tree { .. } => {
                    *tile = TileType::Grass;
                }
                TileType::Farmland { .. } => {
                    *tile = TileType::Grass;
                }
            }
        }
    }
}
