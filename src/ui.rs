use raylib::{
    ffi::{CheckCollisionPointRec, MouseButton},
    prelude::{Color, RaylibDraw, RaylibDrawHandle, Rectangle, Vector2},
};

use crate::{
    map::{Map, TILE_PIXEL_SIZE},
    texture_handler::TextureHandler,
};

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
}

impl Canvas {
    pub fn new() -> Self {
        Self {
            mode: MenuMode::Crops,
            selected: 0,
            content: vec![
                Rectangle::new(10., 70., 50., 50.),
                Rectangle::new(70., 70., 50., 50.),
            ],
            subcontent: vec![],
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
            50. / TILE_PIXEL_SIZE as f32,
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
            50. / TILE_PIXEL_SIZE as f32,
            Color::WHITE,
        );
        let submenu_button_amount = match self.mode {
            MenuMode::Crops => map.crops_data.len(),
            MenuMode::Misc => 2,
        };
        self.subcontent.clear();
        for i in 0..submenu_button_amount {
            let rect = Rectangle {
                x: (i * 60) as f32 + 10.,
                y: 130.,
                width: 50.,
                height: 50.,
            };
            let color = if self.selected == i {
                Color::RAYWHITE
            } else {
                Color::GRAY
            };
            rl.draw_rectangle_rec(rect, color);
            self.subcontent.push(rect);

            // todo: refactor hardcoded ui 
            match self.mode {
                MenuMode::Crops => {
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
                    if i != 0 {
                        continue;
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
