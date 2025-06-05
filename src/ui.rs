use raylib::{
    ffi::{CheckCollisionPointRec, MouseButton},
    prelude::{Color, RaylibDraw, RaylibDrawHandle, Rectangle, Vector2},
};
use serde::Deserialize;

use crate::{
    map::{Map, TILE_PIXEL_SIZE},
    texture_handler::TextureHandler,
    utils::parse_json,
};

const UI_BUTTON_SIZE: f32 = 60.;
const UI_GAPS: f32 = 20.;

#[derive(Deserialize)]
pub struct ToolbarItem {
    tooltip: String,
    pub price: usize
}   

#[derive(Deserialize)]
pub struct ToolbarData {
    pub crops: Vec<ToolbarItem>,
    pub misc: Vec<ToolbarItem>,
}

impl ToolbarData {
    fn new() -> Self {
        parse_json("static/toolbar.json")
    }
}

#[derive(PartialEq)]
pub enum MenuMode {
    Crops,
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
            ],
            subcontent: vec![],
            toolbar_data: ToolbarData::new(),
        }
    }

    pub fn draw(&mut self, rl: &mut RaylibDrawHandle, map: &Map, texture_handler: &TextureHandler) {
        // for node in self.content.iter() {
        //     rl.draw_rectangle_rec(node.rect, node.bg_color);
        // }

        // draw mode selection buttons (submenus)
        rl.draw_rectangle_rec(
            self.content[0],
            if self.mode == MenuMode::Crops {
                Color::RAYWHITE
            } else {
                Color::GRAY
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
        rl.draw_rectangle_rec(
            self.content[1],
            if self.mode == MenuMode::Misc {
                Color::RAYWHITE
            } else {
                Color::GRAY
            },
        );
        let position = Vector2::new(self.content[1].x, self.content[1].y);
        rl.draw_texture_ex(
            texture_handler.textures.get("misc_menu").unwrap(),
            position,
            0.,
            UI_BUTTON_SIZE / TILE_PIXEL_SIZE as f32,
            Color::WHITE,
        );
        let submenu_button_amount = match self.mode {
            MenuMode::Crops => map.crops_data.len(),
            MenuMode::Misc => 2,
        };
        self.subcontent.clear();
        for i in 0..submenu_button_amount {
            let rect = Rectangle {
                x: (UI_BUTTON_SIZE + UI_GAPS),
                y: i as f32 * (UI_BUTTON_SIZE + UI_GAPS / 2.) + UI_BUTTON_SIZE + UI_GAPS,
                width: UI_BUTTON_SIZE,
                height: UI_BUTTON_SIZE,
            };

            let color = if self.selected == i {
                Color::RAYWHITE
            } else {
                Color::GRAY
            };
            rl.draw_rectangle_rec(rect, color);
            self.subcontent.push(rect);

            let tooltip_pool: &Vec<ToolbarItem>;

            // todo: refactor hardcoded ui
            'mode_selection: {
                match self.mode {
                    MenuMode::Crops => {
                        tooltip_pool = &self.toolbar_data.crops;

                        let id = format!("crop{i}");
                        rl.draw_texture_pro(
                            texture_handler.textures.get(&id).unwrap(),
                            Rectangle::new(
                                map.crops_data[i].time_to_grow as f32 * TILE_PIXEL_SIZE as f32,
                                0.0,
                                TILE_PIXEL_SIZE as f32,
                                TILE_PIXEL_SIZE as f32,
                            ),
                            rect,
                            Vector2::zero(),
                            0.,
                            Color::WHITE,
                        );
                    }
                    MenuMode::Misc => {
                        tooltip_pool = &self.toolbar_data.misc;

                        if i != 0 {
                            break 'mode_selection;
                        }

                        rl.draw_texture_pro(
                            texture_handler.textures.get("worker").unwrap(),
                            Rectangle::new(
                                0.0,
                                0.0,
                                TILE_PIXEL_SIZE as f32,
                                TILE_PIXEL_SIZE as f32,
                            ),
                            rect,
                            Vector2::zero(),
                            0.,
                            Color::WHITE,
                        );
                    }
                }
            }

            if self.selected != i {
                continue;
            }

            let price = tooltip_pool[i].price;

            // name
            rl.draw_text(
                &tooltip_pool[i].tooltip,
                2 * (UI_BUTTON_SIZE + UI_GAPS) as i32,
                (i as f32 * (UI_BUTTON_SIZE + UI_GAPS / 2.) + UI_BUTTON_SIZE + UI_GAPS) as i32,
                24,
                Color::RAYWHITE,
            );

            // don't draw price if free
            if price <= 0 {
                continue;
            }

            rl.draw_text(
                &format!("{price} RUB"),
                2 * (UI_BUTTON_SIZE + UI_GAPS) as i32,
                (i as f32 * (UI_BUTTON_SIZE + UI_GAPS / 2.) + UI_BUTTON_SIZE + UI_GAPS + 32.)
                    as i32,
                24,
                Color::RAYWHITE,
            );
        }
    }

    pub fn update(&mut self, rl: &mut RaylibDrawHandle) {
        for i in 0..self.content.len() {
            let rect = self.content[i];
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
                // node.bg_color = hovered;
                if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
                    self.mode = match i {
                        0 => MenuMode::Crops,
                        1 => MenuMode::Misc,
                        _ => MenuMode::Misc,
                    };
                    self.selected = 0;
                }
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
                // node.bg_color = hovered;
                if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
                    self.selected = i;
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
