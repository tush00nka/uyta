use raylib::{ffi::CheckCollisionPointRec, prelude::*};
use serde::{Deserialize, Serialize};

use crate::{
    UI_BUTTON_SIZE, UI_GAPS,
    localization::LocaleHandler,
    map::TILE_PIXEL_SIZE,
    player::Player,
    utils::{get_game_width, parse_json, shrink_number_for_display},
};

#[derive(Deserialize)]
pub struct UpgradeData {
    pub label: String,
    pub description: String,
    pub cost: usize,
}

#[derive(Deserialize)]
pub struct UpgradeStatic {
    pub upgrade_data: Vec<UpgradeData>,
}

#[derive(Serialize, Deserialize)]
pub struct UpgradeDynamic {
    pub purchased_upgrades: Vec<usize>,
}

pub struct UpgradeHandler {
    pub static_data: UpgradeStatic,
    pub dynamic_data: UpgradeDynamic,
}

impl UpgradeHandler {
    pub fn new(language_code: String) -> Self {
        let static_data = parse_json(&format!("static/{}/upgrades.json", language_code))
            .expect("no upgrade data??");

        let res = parse_json("dynamic/upgrades_save.json");
        let dynamic_data = match res {
            Ok(dynamic_data) => dynamic_data,
            Err(_) => UpgradeDynamic {
                purchased_upgrades: vec![],
            },
        };

        Self {
            static_data,
            dynamic_data,
        }
    }

    pub fn reload_static(&mut self, language_code: String) {
        let static_data = parse_json(&format!("static/{}/upgrades.json", language_code))
            .expect("no upgrade data??");
        self.static_data = static_data;
    }

    pub fn get_multiplier_for_crop(&self, crop: usize) -> usize {
        let mut temp = 1;
        if self.dynamic_data.purchased_upgrades.contains(&(crop * 3)) {
            temp *= 2;
        }
        if self.dynamic_data.purchased_upgrades.contains(&(crop * 3 + 1)) {
            temp *= 2;
        }
        if self.dynamic_data.purchased_upgrades.contains(&(crop * 3 + 2)) {
            temp *= 2;
        }

        temp
    }

    pub fn get_multiplier_for_tree(&self, tree: usize, crops_len: usize) -> usize {
        let mut temp = 1;
        if self.dynamic_data.purchased_upgrades.contains(&(crops_len * 3 + tree * 3)) {
            temp *= 2;
        }
        if self.dynamic_data.purchased_upgrades.contains(&(crops_len * 3 + tree * 3 + 1)) {
            temp *= 2;
        }
        if self.dynamic_data.purchased_upgrades.contains(&(crops_len * 3 + tree * 3 + 2)) {
            temp *= 2;
        }

        temp
    }

    pub fn get_multiplier_for_animal(&self, animal: usize, crops_len: usize, trees_len: usize) -> usize {
        let mut temp = 1;
        if self.dynamic_data.purchased_upgrades.contains(&(crops_len*3 + trees_len*3 + animal*3)) {
            temp *= 2;
        }
        if self.dynamic_data.purchased_upgrades.contains(&(crops_len*3 + trees_len*3 + animal*3 + 1)) {
            temp *= 2;
        }
        if self.dynamic_data.purchased_upgrades.contains(&(crops_len*3 + trees_len*3 +animal*3 + 2)) {
            temp *= 2;
        }

        temp
    }

    pub fn draw(
        &mut self,
        rl: &mut RaylibDrawHandle,
        texture: &Texture2D,
        font: &Font,
        player: &mut Player,
        locale_handler: &LocaleHandler,
    ) {
        let mut offset = 0;
        for i in 0..self.static_data.upgrade_data.len() {
            if self.static_data.upgrade_data[i].cost / 2 > player.alltime_max_money {
                offset += 1;
                continue;
            }

            if self.dynamic_data.purchased_upgrades.contains(&i) {
                offset += 1;
                continue;
            }

            let i = i as i32;
            let scale = UI_BUTTON_SIZE + UI_GAPS / 2.;
            let button_rect = Rectangle::new(
                ((i - offset) % 3) as f32 * scale + get_game_width(rl) as f32 - 3. * scale,
                ((i - offset) / 3) as f32 * scale + UI_GAPS / 2.,
                UI_BUTTON_SIZE,
                UI_BUTTON_SIZE,
            );

            let source = Rectangle::new(
                (i % 3 * TILE_PIXEL_SIZE) as f32,
                (i / 3 * TILE_PIXEL_SIZE) as f32,
                TILE_PIXEL_SIZE as f32,
                TILE_PIXEL_SIZE as f32,
            );

            rl.draw_rectangle_rec(button_rect, Color::BLACK.alpha(0.5));
            rl.draw_texture_pro(
                texture,
                source,
                button_rect,
                Vector2::zero(),
                0.,
                Color::WHITE,
            );
        }

        offset = 0;
        for i in 0..self.static_data.upgrade_data.len() {
            if self.static_data.upgrade_data[i].cost / 2 > player.alltime_max_money {
                offset += 1;
                continue;
            }

            if self.dynamic_data.purchased_upgrades.contains(&i) {
                offset += 1;
                continue;
            }

            let data = &self.static_data.upgrade_data[i];

            let i = i as i32;
            let scale = UI_BUTTON_SIZE + UI_GAPS / 2.;
            let button_rect = Rectangle::new(
                ((i - offset) % 3) as f32 * scale + get_game_width(rl) as f32 - 3. * scale,
                ((i - offset) / 3) as f32 * scale + UI_GAPS / 2.,
                UI_BUTTON_SIZE,
                UI_BUTTON_SIZE,
            );

            if !unsafe {
                CheckCollisionPointRec(rl.get_mouse_position().into(), button_rect.into())
            } {
                continue;
            }

            let x = rl.get_mouse_position().x
                - data.description.chars().count() as f32 / 2. * UI_BUTTON_SIZE / 4.;
            let y = rl.get_mouse_position().y;

            let tooltip_rect = Rectangle::new(
                x,
                y,
                data.description.chars().count() as f32 / 2. * UI_BUTTON_SIZE / 4.,
                UI_BUTTON_SIZE * 2.,
            );

            rl.draw_rectangle_rec(tooltip_rect, Color::BLACK.alpha(0.75));
            rl.draw_text_ex(
                font,
                &data.label,
                Vector2::new(x + 5., y),
                UI_BUTTON_SIZE / 2.,
                0.,
                Color::RAYWHITE,
            );
            rl.draw_text_ex(
                font,
                &data.description,
                Vector2::new(x + 5., y + UI_BUTTON_SIZE / 2.),
                UI_BUTTON_SIZE / 3.,
                0.,
                Color::DARKGRAY,
            );
            rl.draw_text_ex(
                font,
                &shrink_number_for_display(data.cost as u128, locale_handler),
                Vector2::new(x + 5., y + UI_BUTTON_SIZE + UI_BUTTON_SIZE / 3.),
                UI_BUTTON_SIZE / 2.,
                0.,
                Color::RAYWHITE,
            );

            if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
                if player.money >= data.cost {
                    player.money -= data.cost;
                    self.dynamic_data.purchased_upgrades.push(i as usize);
                }
            }
        }
    }

    pub fn save(&self) {
        let serialized = serde_json::to_string_pretty(&self.dynamic_data).expect("err");
        std::fs::create_dir_all("dynamic").expect("Couldn't create dir");
        std::fs::write("dynamic/upgrades_save.json", serialized)
            .expect("Couldn't write upgrades data to json");
    }
}
