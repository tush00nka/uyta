use raylib::prelude::*;

use crate::{
    SCREEN_HEIGHT, SCREEN_WIDTH,
};

pub struct CameraController {
    pub position: Vector2,
    pub speed: f32,
    pub camera: Camera2D,
}

impl CameraController {
    pub fn new() -> Self {
        Self {
            position: Vector2 { x: 0., y: 0. },
            speed: 500.0,
            camera: Camera2D {
                target: Vector2::zero(),
                offset: Vector2 {
                    x: SCREEN_WIDTH as f32 / 2.,
                    y: SCREEN_HEIGHT as f32 / 2.,
                },
                zoom: 1.0,
                rotation: 0.0,
            },
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
        // self.position.x = self
        //     .position
        //     .x
        //     .min((MAP_WIDTH as i32 / 2 * TILE_SIZE) as f32)
        //     .max((-(MAP_WIDTH as i32) / 2 * TILE_SIZE) as f32);
        // self.position.y = self
        //     .position
        //     .y
        //     .min((MAP_HEIGHT as i32 / 2 * TILE_SIZE) as f32)
        //     .max((-(MAP_HEIGHT as i32) / 2 * TILE_SIZE) as f32);

        self.camera.target = Vector2 {
            x: lerp(
                self.camera.target.x,
                self.position.x,
                10.0 * rl.get_frame_time(),
            ),
            y: lerp(
                self.camera.target.y,
                self.position.y,
                10.0 * rl.get_frame_time(),
            ),
        };

        if rl.is_window_resized() {
            self.camera.offset = Vector2 {
                x: rl.get_screen_width() as f32 / 2.,
                y: rl.get_screen_height() as f32 / 2.,
            };
        }
    }
}
