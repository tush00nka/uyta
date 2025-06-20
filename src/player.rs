use std::collections::HashMap;

use raylib::{ffi::{PlaySound, Sound}, prelude::*};
use serde::{Deserialize, Serialize};

use crate::{
    AnimalHandler,
    animal::Animal,
    localization::LocaleHandler,
    map::{Map, TileType},
    pause_menu::GameSettigns,
    shop_ui::{Canvas, MenuMode},
    tutorial::Tutorial,
    utils::{get_game_width, parse_json, shrink_number_for_display},
    worker::{Worker, WorkerHandler},
};

#[derive(Serialize, Deserialize)]
pub struct Player {
    pub money: usize,
    pub alltime_max_money: usize,
    pub level: usize,
    pub exp: usize,
    exp_to_lvl_up: usize,
}

impl Player {
    pub fn new() -> Self {
        if !cfg!(target_arch = "wasm32") {
            let player = parse_json("dynamic/player_save.json");
            match player {
                Ok(_) => return player.unwrap(),
                Err(_) => {}
            }
        }

        Self {
            money: 100,
            alltime_max_money: 100,
            level: 1,
            exp: 0,
            exp_to_lvl_up: 20,
        }
    }

    pub fn update_money(&mut self) {
        if self.alltime_max_money < self.money {
            self.alltime_max_money = self.money;
        }
    }

    pub fn update_exp(&mut self, sounds: &HashMap<String, Sound>) {
        if self.exp >= self.exp_to_lvl_up {
            self.level += 1;
            self.exp = 0;
            self.exp_to_lvl_up = (self.exp_to_lvl_up as f32 * 1.5) as usize;
            if !cfg!(target_arch="wasm32") {
                unsafe { PlaySound(*sounds.get("level_up").unwrap()) };
            }
        }
    }

    pub fn draw_stats(
        &self,
        rl: &mut RaylibDrawHandle,
        font: &Font,
        locale_handler: &LocaleHandler,
        settings: &GameSettigns,
    ) {
        rl.draw_rectangle(10, 10, 130, 28, Color::BLACK.alpha(0.5));

        rl.draw_text_ex(
            font,
            &shrink_number_for_display(self.money as u128, locale_handler, settings),
            Vector2::new(14., 14.),
            24.,
            0.,
            Color::WHITE,
        );

        let screen_width = get_game_width(rl);

        let exp_bar_fill = self.exp as f32 / self.exp_to_lvl_up as f32;
        rl.draw_rectangle(
            screen_width / 4,
            10,
            screen_width / 2,
            24,
            Color::BLACK.alpha(0.5),
        );
        rl.draw_rectangle(
            screen_width / 4,
            10,
            (exp_bar_fill * (screen_width / 2) as f32) as i32,
            24,
            Color::DARKORANGE,
        );
        rl.draw_text_ex(
            font,
            &format!(
                "{} {}",
                locale_handler.language_data.get("level").unwrap(),
                self.level
            ),
            Vector2::new(screen_width as f32 / 4. + 10., 10.),
            24.,
            0.,
            Color::WHITE,
        );

        if check_collision_point_poly(
            rl.get_mouse_position(),
            &[
                Vector2::new(screen_width as f32 / 4., 10.),
                Vector2::new(3. * screen_width as f32 / 4., 10.),
                Vector2::new(3. * screen_width as f32 / 4., 34.),
                Vector2::new(screen_width as f32 / 4., 34.),
            ],
        ) {
            let text = format!("{}/{}", self.exp, self.exp_to_lvl_up);
            rl.draw_text_ex(
                font,
                &text,
                rl.get_mouse_position() + Vector2::new(12., -12.),
                24.,
                0.,
                Color::WHITE,
            );
        }
    }

    pub fn plant_crops(
        &mut self,
        canvas: &mut Canvas,
        map: &mut Map,
        selected_tile: &(i32, i32),
        tutorial: &mut Tutorial,
    ) {
        let Some(tile) = map.dynamic_data.tiles.get_mut(selected_tile) else {
            return;
        };

        if let Some(occ_tile) = map.dynamic_data.occupation_map.get_mut(selected_tile) {
            *occ_tile = false;
        }

        tutorial.complete_step(2);

        match tile {
            TileType::Grass => {
                let amount = canvas
                    .toolbar_data
                    .dynamic_data
                    .crop_amount
                    .get_mut(&canvas.selected)
                    .unwrap();

                let mut price = canvas.toolbar_data.static_data.crops[canvas.selected].price;
                for _ in 0..*amount {
                    price = (price as f32 * 1.1) as usize;
                }
                if self.money >= price {
                    self.money -= price;
                    *amount += 1;
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
                    let amount = canvas
                        .toolbar_data
                        .dynamic_data
                        .crop_amount
                        .get(&canvas.selected)
                        .unwrap();

                    let mut price = canvas.toolbar_data.static_data.crops[canvas.selected].price;
                    for _ in 0..*amount {
                        price = (price as f32 * 1.1) as usize;
                    }

                    if self.money >= price {
                        let replaced_amount = canvas
                            .toolbar_data
                            .dynamic_data
                            .crop_amount
                            .get_mut(crop)
                            .unwrap();
                        *replaced_amount -= 1;

                        let amount = canvas
                            .toolbar_data
                            .dynamic_data
                            .crop_amount
                            .get_mut(&canvas.selected)
                            .unwrap();
                        *amount += 1;

                        self.money -= price;
                        *crop = canvas.selected;
                        *stage = 0;
                    }
                }
            }
            _ => {}
        }
    }

    pub fn plant_trees(&mut self, canvas: &mut Canvas, map: &mut Map, selected_tile: &(i32, i32)) {
        let Some(tile) = map.dynamic_data.tiles.get_mut(selected_tile) else {
            return;
        };

        if let Some(occ_tile) = map.dynamic_data.occupation_map.get_mut(selected_tile) {
            *occ_tile = false;
        }

        match tile {
            TileType::Grass => {
                let amount = canvas
                    .toolbar_data
                    .dynamic_data
                    .tree_amount
                    .get_mut(&canvas.selected)
                    .unwrap();

                let mut price = canvas.toolbar_data.static_data.trees[canvas.selected].price;
                for _ in 0..*amount {
                    price = (price as f32 * 1.1) as usize;
                }
                if self.money >= price {
                    self.money -= price;
                    *amount += 1;
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

    pub fn spawn_animals(
        &mut self,
        canvas: &mut Canvas,
        map: &mut Map,
        selected_tile: &(i32, i32),
        animal_handler: &mut AnimalHandler,
    ) {
        let Some(tile) = map.dynamic_data.tiles.get(selected_tile) else {
            return;
        };

        match tile {
            TileType::Grass => {
                let amount = canvas
                    .toolbar_data
                    .dynamic_data
                    .animal_amount
                    .get_mut(&canvas.selected)
                    .unwrap();

                let mut price = canvas.toolbar_data.static_data.animals[canvas.selected].price;
                for _ in 0..*amount {
                    price = (price as f32 * 1.1) as usize;
                }
                if self.money >= price {
                    self.money -= price;
                    *amount += 1;
                    animal_handler.add_animal(Animal::new(
                        canvas.selected,
                        selected_tile.0,
                        selected_tile.1,
                    ));
                }
            }
            _ => {}
        }
    }

    pub fn perform_misc(
        &mut self,
        canvas: &mut Canvas,
        worker_handler: &mut WorkerHandler,
        selected_tile: &(i32, i32),
        map: &mut Map,
    ) {
        let amount = canvas
            .toolbar_data
            .dynamic_data
            .misc_amount
            .get_mut(&canvas.selected)
            .unwrap();

        let mut price = canvas.toolbar_data.static_data.misc[canvas.selected].price;
        for _ in 0..*amount {
            price = (price as f32 * 1.1) as usize;
        }

        if canvas.selected == 0
            && self.money >= price
            && map.dynamic_data.tiles.contains_key(selected_tile)
        {
            worker_handler.add_worker(Worker::new(selected_tile.0, selected_tile.1));
            self.money -= price;
            *amount += 1;
        }

        if canvas.selected == 1 {
            let Some(tile) = map.dynamic_data.tiles.get_mut(&selected_tile) else {
                return;
            };

            if let Some(occ_tile) = map.dynamic_data.occupation_map.get_mut(&selected_tile) {
                *occ_tile = false;
            }

            match tile {
                TileType::Tree { tree, .. } => {
                    let replaced_amount = canvas
                        .toolbar_data
                        .dynamic_data
                        .tree_amount
                        .get_mut(tree)
                        .unwrap();
                    *replaced_amount -= 1;
                    *tile = TileType::Grass;
                }
                TileType::Farmland { crop, .. } => {
                    let replaced_amount = canvas
                        .toolbar_data
                        .dynamic_data
                        .crop_amount
                        .get_mut(crop)
                        .unwrap();
                    *replaced_amount -= 1;
                    *tile = TileType::Grass;
                }
                _ => {}
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn save(&self) {
        let serialized = serde_json::to_string_pretty(self).expect("err");
        std::fs::create_dir_all("dynamic").expect("Couldn't create dir");
        std::fs::write("dynamic/player_save.json", serialized)
            .expect("Couldn't write player data to json");
    }
}
