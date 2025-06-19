use raylib::prelude::*;

use crate::{
    animal::AnimalHandler, camera_controller::CameraController, localization::LocaleHandler, map::{Map, TILE_SCALE, TILE_SIZE}, pause_menu::PauseMenu, player::Player, shop_ui::Canvas, texture_handler::TextureHandler, tutorial::Tutorial, upgrades::UpgradeHandler, worker::WorkerHandler
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
    worker_handler: &mut WorkerHandler,
    animal_handler: &mut AnimalHandler,
    font: &Font,
    selected_tile: (i32, i32),
) {
    let mut d2 = rl.begin_mode2D(camera_controller.camera);

    map.draw(&mut d2, &texture_handler.textures, worker_handler, animal_handler, font);

    if !map.dynamic_data.tiles.contains_key(&selected_tile) {
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
    upgrade_handler: &mut UpgradeHandler,
    map: &Map,
    animal_handler: &AnimalHandler,
    texture_handler: &TextureHandler,
    player: &mut Player,
    pause_menu: &PauseMenu,
    tutorial: &Tutorial,
    font: &Font,
    locale_handler: &LocaleHandler,
    master_volume: f32,
) {
    player.draw_stats(rl, font, locale_handler);

    canvas.draw(rl, map, animal_handler, texture_handler, player, font);
    canvas.update(rl, map, animal_handler, player, font, locale_handler, &upgrade_handler);

    upgrade_handler.draw(rl, texture_handler.textures.get("upgrades").unwrap(), font, player, locale_handler);

    tutorial.draw(rl, font);

    pause_menu.draw(rl, font, master_volume, locale_handler);
}
