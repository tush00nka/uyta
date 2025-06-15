use raylib::{
    ffi::{CheckCollisionPointRec, MouseButton},
    prelude::{Color, RaylibDraw, RaylibDrawHandle, Rectangle, Vector2},
    text::Font,
};
use serde::Deserialize;

use crate::{
    animal::AnimalHandler,
    map::{Map, TILE_PIXEL_SIZE},
    player::Player,
    texture_handler::TextureHandler,
    utils::parse_json,
};

const UI_BUTTON_SIZE: f32 = 60.;
const UI_GAPS: f32 = 20.;

#[derive(Deserialize)]
pub struct ToolbarItem {
    tooltip: String,
    unlock_level: usize,
    pub price: usize,
}

#[derive(Deserialize)]
pub struct ToolbarData {
    pub crops: Vec<ToolbarItem>,
    pub trees: Vec<ToolbarItem>,
    pub animals: Vec<ToolbarItem>,
    pub misc: Vec<ToolbarItem>,
}

impl ToolbarData {
    fn new() -> Self {
        parse_json("static/toolbar.json").expect("no toolbar")
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
    pub fn new() -> Self {
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
            toolbar_data: ToolbarData::new(),
        }
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
        let color = if self.toolbar_data.trees[0].unlock_level > player.level {
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
        let color = if self.toolbar_data.animals[0].unlock_level > player.level {
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
        let color = if self.toolbar_data.misc[0].unlock_level > player.level {
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

            let tooltip_pool: &Vec<ToolbarItem>;

            // todo: refactor hardcoded ui
            match self.mode {
                MenuMode::Crops => {
                    tooltip_pool = &self.toolbar_data.crops;

                    let color = if tooltip_pool[i].unlock_level > player.level {
                        Color::GRAY
                    } else {
                        Color::WHITE
                    };

                    let id = format!("crop{i}");
                    rl.draw_texture_pro(
                        texture_handler.textures.get(&id).unwrap(),
                        Rectangle::new(
                            -TILE_PIXEL_SIZE as f32,
                            0.0,
                            TILE_PIXEL_SIZE as f32,
                            TILE_PIXEL_SIZE as f32,
                        ),
                        rect,
                        Vector2::zero(),
                        0.,
                        color,
                    );
                }
                MenuMode::Trees => {
                    tooltip_pool = &self.toolbar_data.trees;
                    let color = if tooltip_pool[i].unlock_level > player.level {
                        Color::GRAY
                    } else {
                        Color::WHITE
                    };

                    let id = format!("tree{i}");
                    rl.draw_texture_pro(
                        texture_handler.textures.get(&id).unwrap(),
                        Rectangle::new(
                            -TILE_PIXEL_SIZE as f32,
                            0.0,
                            TILE_PIXEL_SIZE as f32,
                            TILE_PIXEL_SIZE as f32,
                        ),
                        rect,
                        Vector2::zero(),
                        0.,
                        color,
                    );
                }
                MenuMode::Animals => {
                    tooltip_pool = &self.toolbar_data.animals;

                    let color = if tooltip_pool[i].unlock_level > player.level {
                        Color::GRAY
                    } else {
                        Color::WHITE
                    };

                    let id = format!("animal{i}");
                    rl.draw_texture_pro(
                        texture_handler.textures.get(&id).unwrap(),
                        Rectangle::new(0.0, 0.0, TILE_PIXEL_SIZE as f32, TILE_PIXEL_SIZE as f32),
                        rect,
                        Vector2::zero(),
                        0.,
                        color,
                    );
                }
                MenuMode::Misc => {
                    tooltip_pool = &self.toolbar_data.misc;

                    let color = if tooltip_pool[i].unlock_level > player.level {
                        Color::GRAY
                    } else {
                        Color::WHITE
                    };

                    let id = format!("misc{i}");
                    rl.draw_texture_pro(
                        texture_handler.textures.get(&id).unwrap(),
                        Rectangle::new(0.0, 0.0, TILE_PIXEL_SIZE as f32, TILE_PIXEL_SIZE as f32),
                        rect,
                        Vector2::zero(),
                        0.,
                        color,
                    );
                }
            }

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

    pub fn update(&mut self, rl: &mut RaylibDrawHandle, player: &Player, font: &Font) {
        for i in 0..self.content.len() {
            let rect = self.content[i];
            if unsafe { CheckCollisionPointRec(rl.get_mouse_position().into(), rect.into()) } {
                let (pool, mode, label) = match i {
                    0 => (&self.toolbar_data.crops, MenuMode::Crops, "Растения".to_string()),
                    1 => (&self.toolbar_data.trees, MenuMode::Trees, "Деревья".to_string()),
                    2 => (&self.toolbar_data.animals, MenuMode::Animals, "Животные".to_string()),
                    _ => (&self.toolbar_data.misc, MenuMode::Misc, "Прочее".to_string()),
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
                    format!("Откроется на уровне {}", pool[0].unlock_level)
                } else {
                    label
                };

                let tooltip_rect = Rectangle::new(
                    x,
                    y,
                    tooltip_text.chars().count() as f32 * UI_BUTTON_SIZE / 3.5,
                    tooltip_text.lines().count() as f32 * UI_BUTTON_SIZE / 2.,
                );

                rl.draw_rectangle_rec(
                    tooltip_rect,
                    Color::BLACK.alpha(0.5),
                );
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
                let pool: &Vec<ToolbarItem> = match self.mode {
                    MenuMode::Crops => &self.toolbar_data.crops,
                    MenuMode::Trees => &self.toolbar_data.trees,
                    MenuMode::Animals => &self.toolbar_data.animals,
                    MenuMode::Misc => &self.toolbar_data.misc,
                };


                let tooltip_text = if pool[i].unlock_level > player.level {
                    format!("Откроется на уровне {}", pool[i].unlock_level)
                } else {
                    if pool[i].price <= 0 {
                        format!("{}", pool[i].tooltip)
                    }
                    else {
                        format!("{}\n{}", pool[i].tooltip, pool[i].price)
                    }
                };

                let x = rl.get_mouse_position().x;
                let y = rl.get_mouse_position().y - (UI_BUTTON_SIZE / 2. * tooltip_text.lines().count() as f32);
                let tooltip_rect = Rectangle::new(
                    x,
                    y,
                    tooltip_text.chars().count() as f32 * UI_BUTTON_SIZE / 3.5,
                    tooltip_text.lines().count() as f32 * UI_BUTTON_SIZE / 2.,
                );

                rl.draw_rectangle_rec(
                    tooltip_rect,
                    Color::BLACK.alpha(0.5),
                );
                rl.draw_text_ex(
                    font,
                    &tooltip_text,
                    Vector2::new(x + 5., y),
                    UI_BUTTON_SIZE / 2.,
                    0.,
                    Color::RAYWHITE,
                );

                if pool[i].unlock_level <= player.level {
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
