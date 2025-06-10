use raylib::{ffi::CheckCollisionPointRec, prelude::*};
use serde::{Deserialize, Serialize};

use crate::{map::TILE_SCALE, utils::{get_game_height, get_game_width, parse_json}};

pub struct Button {
    rect: Rectangle,
    label: String,
    pub state: ButtonState,
}

#[derive(PartialEq)]
pub enum ButtonState {
    Normal,
    Hovered,
    Pressed,
}

#[derive(Copy, Clone, PartialEq)]
pub enum PauseMenuState {
    Main,
    Settings,
}

pub struct PauseMenu {
    pub is_paused: bool,
    pub buttons: Vec<Button>,
    pub state: PauseMenuState,
}

#[derive(Deserialize, Serialize)]
pub struct GameSettigns {
    pub master_volume: f32,
    pub is_fullscreen: bool,
}

impl GameSettigns {
    pub fn new() -> Self {
        let res = parse_json("dynamic/settings.json");
        match res {
            Ok(settings) => return settings,
            Err(_) => {
                return Self {
                    master_volume: 0.5,
                    is_fullscreen: true,
                };
            }
        }
    }

    pub fn save(&self) {
        let serialized = serde_json::to_string_pretty(self).expect("err");
        std::fs::write("dynamic/settings.json", serialized)
            .expect("Couldn't write settings data to json");
    }
}

impl PauseMenu {
    pub fn new(rl: &mut RaylibHandle) -> Self {
        let mut menu = Self {
            is_paused: false,
            buttons: vec![],
            state: PauseMenuState::Main,
        };

        menu.switch_state(rl, PauseMenuState::Main);

        menu
    }

    pub fn switch_state(&mut self, rl: &mut RaylibHandle, state: PauseMenuState) {
        let screen_width = get_game_width(rl) as f32;
        let screen_height = get_game_height(rl) as f32;
        let menu_width = screen_width as f32 * 0.5;
        let menu_height = screen_height as f32 * 0.75;

        match state {
            PauseMenuState::Main => {
                let settings = Button {
                    rect: Rectangle::new(
                        screen_width / 2. - menu_width / 4.,
                        screen_height / 2. - menu_height / 2. + 60.,
                        menu_width / 2.,
                        50.,
                    ),
                    label: "Настройки".to_string(),
                    state: ButtonState::Normal,
                };
                let quit = Button {
                    rect: Rectangle::new(
                        screen_width / 2. - menu_width / 4.,
                        screen_height / 2. - menu_height / 2. + 120.,
                        menu_width / 2.,
                        50.,
                    ),
                    label: "Выйти из игры".to_string(),
                    state: ButtonState::Normal,
                };

                self.buttons = vec![settings, quit];
            }
            PauseMenuState::Settings => {
                let sfx_sub = Button {
                    rect: Rectangle::new(
                        screen_width / 2. - menu_width / 4.,
                        screen_height / 2. - menu_height / 2. + 60.,
                        50.,
                        50.,
                    ),
                    label: "-".to_string(),
                    state: ButtonState::Normal,
                };
                let sfx_add = Button {
                    rect: Rectangle::new(
                        screen_width / 2. + menu_width / 4. - 50.,
                        screen_height / 2. - menu_height / 2. + 60.,
                        50.,
                        50.,
                    ),
                    label: "+".to_string(),
                    state: ButtonState::Normal,
                };
                let fullscreen_label = if rl.is_window_fullscreen() { "В окне".to_string() } else { "Во весь экран".to_string() };
                let fullscreen_toggle = Button {
                    rect: Rectangle::new(
                        screen_width / 2. - menu_width / 4.,
                        screen_height / 2. - menu_height / 2. + 120.,
                        menu_width / 2.,
                        50.,
                    ),
                    label: fullscreen_label,
                    state: ButtonState::Normal,
                };
                let save = Button {
                    rect: Rectangle::new(
                        screen_width / 2. - menu_width / 4.,
                        screen_height / 2. - menu_height / 2. + 180.,
                        menu_width / 2.,
                        50.,
                    ),
                    label: "Сохранить".to_string(),
                    state: ButtonState::Normal,
                };

                self.buttons = vec![sfx_sub, sfx_add, fullscreen_toggle, save];
            }
        }

        self.state = state;
    }

    pub fn toggle_pause(&mut self, rl: &mut RaylibHandle) {
        if rl.is_key_released(KeyboardKey::KEY_ESCAPE) {
            self.is_paused = !self.is_paused;
        }
    }

    pub fn update_buttons(&mut self, rl: &mut RaylibHandle) -> bool {
        if !self.is_paused {
            return false;
        }

        // if rl.is_window_resized() {
        //     self.switch_state(rl, self.state);
        // }

        let mut blocks_mouse = false;

        for i in 0..self.buttons.len() {
            let button = &mut self.buttons[i];
            if unsafe {
                use raylib::ffi::{Rectangle, Vector2};
                let rect = Rectangle {
                    x: button.rect.x,
                    y: button.rect.y,
                    width: button.rect.width,
                    height: button.rect.height,
                };
                let mouse_pos = Vector2 {
                    x: rl.get_mouse_position().x,
                    y: rl.get_mouse_position().y,
                };
                CheckCollisionPointRec(mouse_pos, rect)
            } {
                button.state = ButtonState::Hovered;
                if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
                    button.state = ButtonState::Pressed;
                }
                blocks_mouse = true;
            } else {
                button.state = ButtonState::Normal;
            }
        }

        blocks_mouse
    }

    pub fn draw(&self, rl: &mut RaylibDrawHandle, font: &Font, master_volume: f32) {
        if !self.is_paused {
            return;
        }

        let screen_width = get_game_width(rl);
        let screen_height = get_game_height(rl);

        let menu_width = (screen_width as f32 * 0.5) as i32;
        let menu_height = (screen_height as f32 * 0.75) as i32;

        rl.draw_rectangle(
            screen_width / 2 - menu_width / 2,
            screen_height / 2 - menu_height / 2,
            menu_width,
            menu_height,
            Color::BLACK.alpha(0.8),
        );
        rl.draw_text_ex(
            font,
            "Меню",
            Vector2::new(
                (screen_width / 2 - 24) as f32,
                (screen_height / 2 - menu_height / 2 + 10) as f32,
            ),
            24.,
            0.,
            Color::RAYWHITE,
        );

        for button in self.buttons.iter() {
            let color = match button.state {
                ButtonState::Normal => Color::GRAY,
                ButtonState::Hovered => Color::RAYWHITE,
                ButtonState::Pressed => Color::GRAY,
            };
            rl.draw_rectangle_lines_ex(button.rect, TILE_SCALE as f32, color);
            rl.draw_text_ex(
                font,
                &button.label,
                Vector2::new(
                    button.rect.x + button.rect.width / 2.
                        - button.label.chars().count() as f32 * 6.,
                    button.rect.y + button.rect.height / 2. - 12.,
                ),
                24.,
                0.,
                Color::RAYWHITE,
            );
        }

        if self.state == PauseMenuState::Settings {
            rl.draw_text_ex(
                font,
                &format!(
                    "Общая громкость\n{}%",
                    (master_volume * 100.).round() as usize
                ),
                Vector2::new(
                    screen_width as f32 / 2. - menu_width as f32 / 4. + 60.,
                    screen_height as f32 / 2. - menu_height as f32 / 2. + 60.,
                ),
                24.,
                0.,
                Color::RAYWHITE,
            );
        }
    }
}
