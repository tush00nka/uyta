use raylib::prelude::*;

use crate::map::{MAP_HEIGHT, MAP_WIDTH, TILE_SIZE};

pub struct CameraController {
    pub position: Vector2,
    pub speed: f32,
}

impl CameraController {
    pub fn new() -> Self {
        Self {
            position: Vector2 { x: 0., y: 0. },
            speed: 500.0,
        }
    }

    pub fn update_position(&mut self, rl: &mut RaylibHandle) {
        use raylib::consts::KeyboardKey::*;

        let mut direction = Vector2::zero();

        if rl.is_key_down(KEY_A) {
            direction.x = -1.;
        }
        if rl.is_key_down(KEY_D) {
            direction.x = 1.;
        }
        if rl.is_key_down(KEY_W) {
            direction.y = -1.;
        }
        if rl.is_key_down(KEY_S) {
            direction.y = 1.;
        }

        self.position += direction.normalized() * self.speed * rl.get_frame_time();
        self.position.x = self
            .position
            .x
            .min((MAP_WIDTH as i32 / 2 * TILE_SIZE) as f32)
            .max((-(MAP_WIDTH as i32) / 2 * TILE_SIZE) as f32);
        self.position.y = self
            .position
            .y
            .min((MAP_HEIGHT as i32 / 2 * TILE_SIZE) as f32)
            .max((-(MAP_HEIGHT as i32) / 2 * TILE_SIZE) as f32);
    }
}
