use raylib::{ffi::CheckCollisionPointRec, prelude::*};

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

pub struct PauseMenu {
    pub is_paused: bool,
    pub buttons: Vec<Button>,
}

impl PauseMenu {
    pub fn new(rl: &mut RaylibHandle) -> Self {
        let screen_width = rl.get_screen_width() as f32;
        let screen_height = rl.get_screen_height() as f32;
        let menu_width = screen_width as f32 * 0.5;
        let menu_height = screen_height as f32 * 0.75;

        let settings = Button {
            rect: Rectangle::new(
                screen_width / 2. - menu_width / 4.,
                screen_height / 2. - menu_height / 2. + 60.,
                menu_width / 2.,
                50.,
            ),
            label: "Settings".to_string(),
            state: ButtonState::Normal,
        };
        let quit = Button {
            rect: Rectangle::new(
                screen_width / 2. - menu_width / 4.,
                screen_height / 2. - menu_height / 2. + 120.,
                menu_width / 2.,
                50.,
            ),
            label: "Quit".to_string(),
            state: ButtonState::Normal,
        };

        Self {
            is_paused: false,
            buttons: vec![settings, quit],
        }
    }

    pub fn toggle_pause(&mut self, rl: &mut RaylibHandle) {
        if rl.is_key_released(KeyboardKey::KEY_ESCAPE) {
            self.is_paused = !self.is_paused;
            println!("pause: {}", self.is_paused);
        }
    }

    pub fn update_buttons(&mut self, rl: &mut RaylibHandle) -> bool {
        if !self.is_paused {
            return false;
        }

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
                if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
                    button.state = ButtonState::Pressed;
                }
                blocks_mouse = true;
            } else {
                button.state = ButtonState::Normal;
            }
        }

        blocks_mouse
    }

    pub fn draw(&self, rl: &mut RaylibDrawHandle) {
        if !self.is_paused {
            return;
        }

        let screen_width = rl.get_screen_width();
        let screen_height = rl.get_screen_height();
        let menu_width = (screen_width as f32 * 0.5) as i32;
        let menu_height = (screen_height as f32 * 0.75) as i32;

        rl.draw_rectangle(
            screen_width / 2 - menu_width / 2,
            screen_height / 2 - menu_height / 2,
            menu_width,
            menu_height,
            Color::RAYWHITE.alpha(0.5),
        );
        rl.draw_text(
            "Menu",
            screen_width / 2 - 24,
            screen_height / 2 - menu_height / 2 + 10,
            24,
            Color::BLACK,
        );

        for button in self.buttons.iter() {
            let color = match button.state {
                ButtonState::Normal => Color::DARKGRAY,
                ButtonState::Hovered => Color::GRAY,
                ButtonState::Pressed => Color::WHITE,
            };
            rl.draw_rectangle_rec(button.rect, color);
            rl.draw_text(
                &button.label,
                button.rect.x as i32,
                button.rect.y as i32,
                24,
                Color::RAYWHITE,
            );
        }
    }
}
