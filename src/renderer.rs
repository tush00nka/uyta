use raylib::prelude::*;

use crate::{
    camera_controller::CameraController,
    map::{Map, TILE_SCALE, TILE_SIZE},
    pause_menu::PauseMenu,
    player::Player,
    texture_handler::TextureHandler,
    tutorial::Tutorial,
    ui::Canvas,
    worker::Worker,
};

pub fn draw_bg(rl: &mut RaylibDrawHandle, bg_shader: &mut Shader, bg_texture: &Texture2D) {
    rl.clear_background(Color::BLACK);

    rl.draw_shader_mode(bg_shader, |mut shader| {
        shader.draw_texture_ex(bg_texture, Vector2::zero(), 0., 2., Color::WHITE);
    });
}

pub fn draw_for_camera(
    rl: &mut RaylibDrawHandle,
    map: &Map,
    camera_controller: &CameraController,
    texture_handler: &TextureHandler,
    workers: &mut Vec<Worker>,
    font: &Font,
    selected_tile: (i32, i32),
) {
    let mut d2 = rl.begin_mode2D(camera_controller.camera);

    map.draw(&mut d2, &texture_handler.textures, workers, font);

    if !map.tiles.contains_key(&selected_tile) {
        return;
    }

    d2.draw_rectangle_lines_ex(
        Rectangle::new(
            (selected_tile.0 * TILE_SIZE) as f32,
            (selected_tile.1 * TILE_SIZE) as f32,
            TILE_SIZE as f32,
            TILE_SIZE as f32,
        ),
        TILE_SCALE as f32,
        Color::RAYWHITE,
    );
}

pub fn draw_fg(
    rl: &mut RaylibDrawHandle,
    canvas: &mut Canvas,
    map: &Map,
    texture_handler: &TextureHandler,
    player: &Player,
    pause_menu: &PauseMenu,
    tutorial: &Tutorial,
    font: &Font,
    master_volume: f32,
) {
    player.draw_stats(rl, font);

    canvas.draw(rl, &map, &texture_handler, player, font);
    canvas.update(rl, player, font);

    tutorial.draw(rl, font);

    pause_menu.draw(rl, font, master_volume);
}
