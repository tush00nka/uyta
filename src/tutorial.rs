use raylib::prelude::*;

use crate::{player::Player, utils::parse_json};

struct TutorialStep {
    label: String,
    completed: bool,
}

impl TutorialStep {
    fn new(label: String) -> Self {
        Self {
            label,
            completed: false,
        }
    }
}

pub struct Tutorial {
    steps: Vec<TutorialStep>,
    hidden: bool,
}

impl Tutorial {
    pub fn new() -> Self {
        let player: Result<Player, serde_json::Error> = parse_json("dynamic/player_save.json");

        let hidden = match player {
            Ok(_) => true,
            Err(_) => false,
        };

        Self {
            steps: vec![
                TutorialStep::new("Перемещайте камеру при помощи [W, A, S, D]".to_string()),
                TutorialStep::new("Посадите морковь на острове при помощи [ЛКМ]".to_string()),
            ],
            hidden
        }
    }

    pub fn complete_step(&mut self, index: usize) {
        self.steps[index].completed = true;
    }

    pub fn draw(&self, rl: &mut RaylibDrawHandle, font: &Font) {
        if self.hidden {
            return;
        }

        let mut text = "".to_string();

        let mut all_completed = true;

        for step in self.steps.iter() {
            let mark = if step.completed {
                '+'
            } else {
                all_completed = false;
                ' '
            };
            text += &format!("[{}] {}\n", mark, step.label);
        }

        if all_completed {
            let label = "Вы прошли обучение! Нажмите [F1], чтобы скрыть подсказки";
            rl.draw_text_ex(
                font,
                label,
                Vector2::new(
                    10.,
                    rl.get_screen_height() as f32 - 24. * self.steps.len() as f32 - 34.,
                ),
                24.,
                0.,
                Color::DARKORANGE,
            );
        }

        rl.draw_text_ex(
            font,
            &text,
            Vector2::new(
                10.,
                rl.get_screen_height() as f32 - 24. * self.steps.len() as f32 - 10.,
            ),
            24.,
            0.,
            Color::RAYWHITE,
        );
    }

    pub fn close_tutorial(&mut self, rl: &mut RaylibHandle) {
        if self.hidden {
            return;
        }
        
        for step in self.steps.iter() {
            if !step.completed {
                return;
            }
        }

        if rl.is_key_pressed(KeyboardKey::KEY_F1) {
            self.hidden = true;
        }
    }
}
