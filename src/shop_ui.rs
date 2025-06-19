use std::collections::HashMap;

use raylib::{
    ffi::{CheckCollisionPointRec, MouseButton},
    prelude::{Color, RaylibDraw, RaylibDrawHandle, Rectangle, Vector2},
    text::Font,
};
use serde::{Deserialize, Serialize};

use crate::{
    UI_BUTTON_SIZE, UI_GAPS,
    animal::AnimalHandler,
    localization::LocaleHandler,
    map::{Map, TILE_PIXEL_SIZE},
    player::Player,
    texture_handler::TextureHandler,
    upgrades::UpgradeHandler,
    utils::{parse_json, shrink_number_for_display},
};

#[derive(Deserialize)]
pub struct ToolbarItem {
    pub tooltip: String,
    unlock_level: usize,
    pub price: usize,
}
#[derive(Deserialize)]
pub struct ToolbarStatic {
    pub crops: Vec<ToolbarItem>,
    pub trees: Vec<ToolbarItem>,
    pub animals: Vec<ToolbarItem>,
    pub misc: Vec<ToolbarItem>,
}

#[derive(Deserialize, Serialize)]
pub struct ToolbarDynamic {
    pub crop_amount: HashMap<usize, usize>,
    pub tree_amount: HashMap<usize, usize>,
    pub animal_amount: HashMap<usize, usize>,
    pub misc_amount: HashMap<usize, usize>,
}

impl ToolbarDynamic {
    fn new(static_data: &ToolbarStatic) -> Self {
        let mut crop_amount = HashMap::new();
        for i in 0..static_data.crops.len() {
            crop_amount.insert(i, 0);
        }
        let mut tree_amount = HashMap::new();
        for i in 0..static_data.trees.len() {
            tree_amount.insert(i, 0);
        }
        let mut animal_amount = HashMap::new();
        for i in 0..static_data.animals.len() {
            animal_amount.insert(i, 0);
        }
        let mut misc_amount = HashMap::new();
        for i in 0..static_data.misc.len() {
            misc_amount.insert(i, 0);
        }

        Self {
            crop_amount,
            tree_amount,
            animal_amount,
            misc_amount,
        }
    }
}

pub struct ToolbarData {
    pub static_data: ToolbarStatic,
    pub dynamic_data: ToolbarDynamic,
}

impl ToolbarData {
    fn new(language_code: String) -> Self {
        let static_data =
            parse_json(&format!("static/{}/toolbar.json", language_code)).expect("no toolbar");

        let res = parse_json("dynamic/toolbar_save.json");

        let dynamic_data = match res {
            Ok(dynamic_data) => dynamic_data,
            Err(_) => ToolbarDynamic::new(&static_data),
        };

        Self {
            static_data,
            dynamic_data,
        }
    }

    pub fn get_price_for_crop(&self, index: usize) -> usize {
        let mut price = self.static_data.crops[index].price;
        for _ in 0..*self.dynamic_data.crop_amount.get(&index).unwrap() {
            price = (price as f32 * 1.1) as usize;
        }
        price
    }

    pub fn get_price_for_tree(&self, index: usize) -> usize {
        let mut price = self.static_data.trees[index].price;
        for _ in 0..*self.dynamic_data.tree_amount.get(&index).unwrap() {
            price = (price as f32 * 1.1) as usize;
        }
        price
    }

    pub fn get_price_for_animal(&self, index: usize) -> usize {
        let mut price = self.static_data.animals[index].price;
        for _ in 0..*self.dynamic_data.animal_amount.get(&index).unwrap() {
            price = (price as f32 * 1.1) as usize;
        }
        price
    }

    pub fn get_price_for_misc(&self, index: usize) -> usize {
        let mut price = self.static_data.misc[index].price;
        for _ in 0..*self.dynamic_data.misc_amount.get(&index).unwrap() {
            price = (price as f32 * 1.1) as usize;
        }
        price
    }

    fn reload_static(&mut self, language_code: String) {
        let static_data = parse_json(&format!("static/{}/toolbar.json", language_code))
            .expect("no upgrade data??");
        self.static_data = static_data;
    }

    pub fn save(&self) {
        let serialized = serde_json::to_string_pretty(&self.dynamic_data).expect("err");
        std::fs::create_dir_all("dynamic").expect("Couldn't create dir");
        std::fs::write("dynamic/toolbar_save.json", serialized)
            .expect("Couldn't write toolbar data to json");
    }
}

#[derive(PartialEq)]
pub enum MenuMode {
    Crops,
    Trees,
    Animals,
    Misc,
}

pub struct Canvas {
    pub mode: MenuMode,
    pub selected: usize,
    content: Vec<Rectangle>,
    subcontent: Vec<Rectangle>,
    pub toolbar_data: ToolbarData,
}

impl Canvas {
    pub fn new(language_code: String) -> Self {
        Self {
            mode: MenuMode::Crops,
            selected: 0,
            content: vec![
                Rectangle::new(
                    10.,
                    UI_BUTTON_SIZE + UI_GAPS,
                    UI_BUTTON_SIZE,
                    UI_BUTTON_SIZE,
                ),
                Rectangle::new(
                    10.,
                    2. * UI_BUTTON_SIZE + UI_GAPS * 1.5,
                    UI_BUTTON_SIZE,
                    UI_BUTTON_SIZE,
                ),
                Rectangle::new(
                    10.,
                    3. * UI_BUTTON_SIZE + UI_GAPS * 2.,
                    UI_BUTTON_SIZE,
                    UI_BUTTON_SIZE,
                ),
                Rectangle::new(
                    10.,
                    4. * UI_BUTTON_SIZE + UI_GAPS * 2.5,
                    UI_BUTTON_SIZE,
                    UI_BUTTON_SIZE,
                ),
            ],
            subcontent: vec![],
            toolbar_data: ToolbarData::new(language_code),
        }
    }

    pub fn reload_toolbar_static(&mut self, language_code: String) {
        self.toolbar_data.reload_static(language_code);
    }

    pub fn draw(
        &mut self,
        rl: &mut RaylibDrawHandle,
        map: &Map,
        animal_handler: &AnimalHandler,
        texture_handler: &TextureHandler,
        player: &Player,
        font: &Font,
    ) {
        // draw mode selection buttons (submenus)
        rl.draw_rectangle_rec(
            self.content[0],
            if self.mode == MenuMode::Crops {
                Color::RAYWHITE.alpha(0.9)
            } else {
                Color::BLACK.alpha(0.5)
            },
        );
        let position = Vector2::new(self.content[0].x, self.content[0].y);
        rl.draw_texture_ex(
            texture_handler.textures.get("crop_menu").unwrap(),
            position,
            0.,
            UI_BUTTON_SIZE / TILE_PIXEL_SIZE as f32,
            Color::WHITE,
        );

        let position = Vector2::new(self.content[1].x, self.content[1].y);
        let color = if self.toolbar_data.static_data.trees[0].unlock_level > player.level {
            Color::GRAY
        } else {
            Color::WHITE
        };
        rl.draw_rectangle_rec(
            self.content[1],
            if self.mode == MenuMode::Trees {
                Color::RAYWHITE.alpha(0.9)
            } else {
                Color::BLACK.alpha(0.5)
            },
        );
        rl.draw_texture_ex(
            texture_handler.textures.get("tree_menu").unwrap(),
            position,
            0.,
            UI_BUTTON_SIZE / TILE_PIXEL_SIZE as f32,
            color,
        );

        let position = Vector2::new(self.content[2].x, self.content[2].y);
        let color = if self.toolbar_data.static_data.animals[0].unlock_level > player.level {
            Color::GRAY
        } else {
            Color::WHITE
        };
        rl.draw_rectangle_rec(
            self.content[2],
            if self.mode == MenuMode::Animals {
                Color::RAYWHITE.alpha(0.9)
            } else {
                Color::BLACK.alpha(0.5)
            },
        );
        rl.draw_texture_ex(
            texture_handler.textures.get("animals_menu").unwrap(),
            position,
            0.,
            UI_BUTTON_SIZE / TILE_PIXEL_SIZE as f32,
            color,
        );

        let position = Vector2::new(self.content[3].x, self.content[3].y);
        let color = if self.toolbar_data.static_data.misc[0].unlock_level > player.level {
            Color::GRAY
        } else {
            Color::WHITE
        };
        rl.draw_rectangle_rec(
            self.content[3],
            if self.mode == MenuMode::Misc {
                Color::RAYWHITE.alpha(0.9)
            } else {
                Color::BLACK.alpha(0.5)
            },
        );
        rl.draw_texture_ex(
            texture_handler.textures.get("misc_menu").unwrap(),
            position,
            0.,
            UI_BUTTON_SIZE / TILE_PIXEL_SIZE as f32,
            color,
        );

        let submenu_button_amount = match self.mode {
            MenuMode::Crops => map.static_data.crops_data.len(),
            MenuMode::Misc => 2,
            MenuMode::Trees => map.static_data.tree_data.len(),
            MenuMode::Animals => animal_handler.static_data.animal_data.len(),
        };
        self.subcontent.clear();

        rl.draw_rectangle(
            (UI_BUTTON_SIZE + UI_GAPS) as i32,
            (UI_BUTTON_SIZE + UI_GAPS) as i32,
            UI_GAPS as i32 / 2,
            UI_BUTTON_SIZE as i32 * 4 + (UI_GAPS * 1.5) as i32,
            Color::BLACK.alpha(0.5),
        );

        for i in 0..submenu_button_amount {
            let rect = Rectangle {
                x: (UI_BUTTON_SIZE + UI_GAPS * 2.),
                y: i as f32 * (UI_BUTTON_SIZE + UI_GAPS / 2.) + UI_BUTTON_SIZE + UI_GAPS,
                width: UI_BUTTON_SIZE,
                height: UI_BUTTON_SIZE,
            };

            let color = if self.selected == i {
                Color::RAYWHITE.alpha(0.9)
            } else {
                Color::BLACK.alpha(0.5)
            };

            rl.draw_rectangle_rec(rect, color);
            self.subcontent.push(rect);

            let tooltip_pool;
            let amount_pool;
            let texture;
            let source;
            match self.mode {
                MenuMode::Crops => {
                    tooltip_pool = &self.toolbar_data.static_data.crops;
                    amount_pool = &self.toolbar_data.dynamic_data.crop_amount;
                    texture = texture_handler.textures.get(&format!("crop{i}")).unwrap();
                    source = Rectangle::new(
                        (texture.width - TILE_PIXEL_SIZE) as f32,
                        0.0,
                        TILE_PIXEL_SIZE as f32,
                        TILE_PIXEL_SIZE as f32,
                    );
                }
                MenuMode::Trees => {
                    tooltip_pool = &self.toolbar_data.static_data.trees;
                    amount_pool = &self.toolbar_data.dynamic_data.tree_amount;
                    texture = texture_handler.textures.get(&format!("tree{i}")).unwrap();
                    source = Rectangle::new(
                        (texture.width - TILE_PIXEL_SIZE) as f32,
                        0.0,
                        TILE_PIXEL_SIZE as f32,
                        TILE_PIXEL_SIZE as f32,
                    );
                }
                MenuMode::Animals => {
                    tooltip_pool = &self.toolbar_data.static_data.animals;
                    amount_pool = &self.toolbar_data.dynamic_data.animal_amount;
                    texture = texture_handler.textures.get(&format!("animal{i}")).unwrap();
                    source =
                        Rectangle::new(0.0, 0.0, TILE_PIXEL_SIZE as f32, TILE_PIXEL_SIZE as f32);
                }
                MenuMode::Misc => {
                    tooltip_pool = &self.toolbar_data.static_data.misc;
                    amount_pool = &self.toolbar_data.dynamic_data.misc_amount;
                    texture = texture_handler.textures.get(&format!("misc{i}")).unwrap();
                    source =
                        Rectangle::new(0.0, 0.0, TILE_PIXEL_SIZE as f32, TILE_PIXEL_SIZE as f32);
                }
            }

            let color = if tooltip_pool[i].unlock_level > player.level {
                Color::GRAY
            } else {
                Color::WHITE
            };

            rl.draw_texture_pro(
                texture,
                source,
                rect,
                Vector2::zero(),
                0.,
                color,
            );

            if *amount_pool.get(&i).unwrap() <= 0 {
                continue;
            }

            rl.draw_text_pro(
                font,
                &amount_pool.get(&i).unwrap().to_string(),
                Vector2::new(rect.x, rect.y),
                Vector2::zero(),
                0.,
                16.,
                0.,
                if self.selected == i {
                    Color::BLACK
                } else {
                    Color::RAYWHITE
                },
            );

            // if self.selected != i {
            //     continue;
            // }

            // let price = if tooltip_pool[i].price > 0 {
            //     tooltip_pool[i].price.to_string()
            // } else {
            //     "".to_string()
            // };

            // rl.draw_text_ex(
            //     font,
            //     &format!("{}\n{}", tooltip_pool[i].tooltip, price),
            //     Vector2::new(
            //         2. * (UI_BUTTON_SIZE + UI_GAPS) + UI_GAPS,
            //         i as f32 * (UI_BUTTON_SIZE + UI_GAPS / 2.) + UI_BUTTON_SIZE + UI_GAPS,
            //     ),
            //     UI_BUTTON_SIZE / 2.,
            //     0.,
            //     Color::RAYWHITE,
            // );
        }
    }

    pub fn update(
        &mut self,
        rl: &mut RaylibDrawHandle,
        map: &Map,
        animal_handler: &AnimalHandler,
        player: &Player,
        font: &Font,
        locale_handler: &LocaleHandler,
        upgrade_handler: &UpgradeHandler,
    ) {
        for i in 0..self.content.len() {
            let rect = self.content[i];
            if unsafe { CheckCollisionPointRec(rl.get_mouse_position().into(), rect.into()) } {
                let (pool, mode, label) = match i {
                    0 => (
                        &self.toolbar_data.static_data.crops,
                        MenuMode::Crops,
                        locale_handler.language_data.get("plants").unwrap(),
                    ),
                    1 => (
                        &self.toolbar_data.static_data.trees,
                        MenuMode::Trees,
                        locale_handler.language_data.get("trees").unwrap(),
                    ),
                    2 => (
                        &self.toolbar_data.static_data.animals,
                        MenuMode::Animals,
                        locale_handler.language_data.get("animals").unwrap(),
                    ),
                    _ => (
                        &self.toolbar_data.static_data.misc,
                        MenuMode::Misc,
                        locale_handler.language_data.get("misc").unwrap(),
                    ),
                };

                if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
                    if pool[0].unlock_level <= player.level {
                        self.mode = mode;
                        self.selected = 0;
                    }
                }

                let x = rl.get_mouse_position().x;
                let y = rl.get_mouse_position().y - UI_BUTTON_SIZE / 2.;
                let tooltip_text = if pool[0].unlock_level > player.level {
                    format!(
                        "{} {}",
                        locale_handler.language_data.get("locked").unwrap(),
                        pool[0].unlock_level
                    )
                } else {
                    label.to_string()
                };

                let tooltip_rect = Rectangle::new(
                    x,
                    y,
                    tooltip_text.lines().next().unwrap().chars().count() as f32 * UI_BUTTON_SIZE
                        / 3.5,
                    tooltip_text.lines().count() as f32 * UI_BUTTON_SIZE / 2.,
                );

                rl.draw_rectangle_rec(tooltip_rect, Color::BLACK.alpha(0.75));
                rl.draw_text_ex(
                    font,
                    &tooltip_text,
                    Vector2::new(x + 5., y),
                    UI_BUTTON_SIZE / 2.,
                    0.,
                    Color::RAYWHITE,
                );
            }
        }

        for i in 0..self.subcontent.len() {
            let rect = self.subcontent[i];
            if unsafe {
                use raylib::ffi::{Rectangle, Vector2};
                let rect = Rectangle {
                    x: rect.x,
                    y: rect.y,
                    width: rect.width,
                    height: rect.height,
                };
                let mouse_pos = Vector2 {
                    x: rl.get_mouse_position().x,
                    y: rl.get_mouse_position().y,
                };
                CheckCollisionPointRec(mouse_pos, rect)
            } {
                let (toolbar_item, amount, output_price, output_exp) = match self.mode {
                    MenuMode::Crops => (
                        &self.toolbar_data.static_data.crops[i],
                        self.toolbar_data.dynamic_data.crop_amount.get(&i).unwrap(),
                        map.static_data.crops_data[i].sell_price
                            * upgrade_handler.get_multiplier_for_crop(i),
                        map.static_data.crops_data[i].exp
                            * upgrade_handler.get_multiplier_for_crop(i),
                    ),
                    MenuMode::Trees => (
                        &self.toolbar_data.static_data.trees[i],
                        self.toolbar_data.dynamic_data.tree_amount.get(&i).unwrap(),
                        map.static_data.tree_data[i].sell_price
                            * upgrade_handler.get_multiplier_for_tree(
                                i,
                                self.toolbar_data.static_data.crops.len(),
                            ),
                        map.static_data.tree_data[i].exp
                            * upgrade_handler.get_multiplier_for_tree(
                                i,
                                self.toolbar_data.static_data.crops.len(),
                            ),
                    ),
                    MenuMode::Animals => (
                        &self.toolbar_data.static_data.animals[i],
                        self.toolbar_data
                            .dynamic_data
                            .animal_amount
                            .get(&i)
                            .unwrap(),
                        animal_handler.static_data.animal_data[i].drop_cost
                            * upgrade_handler.get_multiplier_for_animal(
                                i,
                                self.toolbar_data.static_data.crops.len(),
                                self.toolbar_data.static_data.trees.len(),
                            ),
                        animal_handler.static_data.animal_data[i].exp
                            * upgrade_handler.get_multiplier_for_animal(
                                i,
                                self.toolbar_data.static_data.crops.len(),
                                self.toolbar_data.static_data.trees.len(),
                            ),
                    ),
                    MenuMode::Misc => (
                        &self.toolbar_data.static_data.misc[i],
                        self.toolbar_data.dynamic_data.misc_amount.get(&i).unwrap(),
                        0,
                        0,
                    ),
                };

                let tooltip_text = if toolbar_item.unlock_level > player.level {
                    format!(
                        "{} {}",
                        locale_handler.language_data.get("locked").unwrap(),
                        toolbar_item.unlock_level
                    )
                } else {
                    if toolbar_item.price <= 0 {
                        format!("{}", toolbar_item.tooltip)
                    } else {
                        let mut price = toolbar_item.price;
                        for _ in 0..*amount {
                            price = (price as f32 * 1.1) as usize;
                        }

                        format!(
                            "{}\n{}",
                            toolbar_item.tooltip,
                            shrink_number_for_display(price, locale_handler),
                        )
                    }
                };

                let tooltip_extra = if output_price > 0 {
                    format!(
                        "{} {}\n{} {}",
                        output_price,
                        locale_handler.language_data.get("per_harvest").unwrap(),
                        output_exp,
                        locale_handler.language_data.get("exp_per_harvest").unwrap(),
                    )
                } else {
                    "".to_string()
                };

                let tooltip_extra = if output_price > 0 {
                    format!(
                        "{} {}\n{} {}",
                        output_price,
                        locale_handler.language_data.get("per_harvest").unwrap(),
                        output_exp,
                        locale_handler.language_data.get("exp_per_harvest").unwrap(),
                    )
                } else {
                    "".to_string()
                };

                let x = rl.get_mouse_position().x;
                let y = rl.get_mouse_position().y
                    - (UI_BUTTON_SIZE / 2.
                        * (tooltip_text.lines().count() + tooltip_extra.lines().count()) as f32);
                let longest_line = tooltip_text
                    .lines()
                    .chain(tooltip_extra.lines())
                    .max_by(|&a, &b| a.chars().count().cmp(&b.chars().count()))
                    .unwrap();
                let tooltip_rect = Rectangle::new(
                    x,
                    y,
                    longest_line.chars().count() as f32 * UI_BUTTON_SIZE / 3.5,
                    (tooltip_text.lines().count() + tooltip_extra.lines().count()) as f32
                        * UI_BUTTON_SIZE
                        / 2.
                        + 5.,
                );

                rl.draw_rectangle_rec(tooltip_rect, Color::BLACK.alpha(0.75));
                rl.draw_text_ex(
                    font,
                    &tooltip_text,
                    Vector2::new(x + 5., y),
                    UI_BUTTON_SIZE / 2.,
                    0.,
                    Color::RAYWHITE,
                );
                rl.draw_text_ex(
                    font,
                    &tooltip_extra,
                    Vector2::new(
                        x + 5.,
                        y + tooltip_rect.height
                            - tooltip_extra.lines().count() as f32 * UI_BUTTON_SIZE / 2.,
                    ),
                    UI_BUTTON_SIZE / 2. - 8.,
                    0.,
                    Color::DARKGRAY,
                );

                if toolbar_item.unlock_level <= player.level {
                    if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
                        self.selected = i;
                    }
                }
            }
        }
    }

    pub fn blocks_mouse(&self, mouse_position: Vector2) -> bool {
        for node in self.content.iter() {
            // stupid unsafe conversion
            if unsafe {
                use raylib::ffi::{Rectangle, Vector2};
                let rect = Rectangle {
                    x: node.x,
                    y: node.y,
                    width: node.width,
                    height: node.height,
                };
                let mouse_pos = Vector2 {
                    x: mouse_position.x,
                    y: mouse_position.y,
                };
                CheckCollisionPointRec(mouse_pos, rect)
            } {
                return true;
            }
        }

        for node in self.subcontent.iter() {
            // stupid unsafe conversion
            if unsafe {
                use raylib::ffi::{Rectangle, Vector2};
                let rect = Rectangle {
                    x: node.x,
                    y: node.y,
                    width: node.width,
                    height: node.height,
                };
                let mouse_pos = Vector2 {
                    x: mouse_position.x,
                    y: mouse_position.y,
                };
                CheckCollisionPointRec(mouse_pos, rect)
            } {
                return true;
            }
        }

        false
    }
}
