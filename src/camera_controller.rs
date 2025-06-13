use raylib::prelude::*;

use crate::{
    tutorial::Tutorial, SCREEN_HEIGHT, SCREEN_WIDTH
};

pub struct CameraController {
    pub position: Vector2,
    pub speed: f32,
    target_zoom: f32,
    pub camera: Camera2D,
}

impl CameraController {
    pub fn new() -> Self {
        Self {
            position: Vector2 { x: 0., y: 0. },
            speed: 500.0,
            target_zoom: 1.0, 
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

    pub fn update_position(&mut self, rl: &mut RaylibHandle, tutorial: &mut Tutorial) {
        use raylib::consts::KeyboardKey::*;

        let mut direction = Vector2::zero();

        if rl.is_key_down(KEY_A) {
            tutorial.complete_step(0);
            direction.x = -1.;
        }
        if rl.is_key_down(KEY_D) {
            tutorial.complete_step(0);
            direction.x = 1.;
        }
        if rl.is_key_down(KEY_W) {
            tutorial.complete_step(0);
            direction.y = -1.;
        }
        if rl.is_key_down(KEY_S) {
            tutorial.complete_step(0);
            direction.y = 1.;
        }

        self.position += direction.normalized() * self.speed * rl.get_frame_time();
        self.camera.zoom = lerp(self.camera.zoom, self.target_zoom, 10. * rl.get_frame_time());

        if rl.get_mouse_wheel_move() > 0. {
            self.target_zoom = (self.target_zoom * 1.1).min(2.);
        }
        if rl.get_mouse_wheel_move() < 0. {
            self.target_zoom = (self.target_zoom / 1.1).max(0.5);
        }

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
