use raylib::prelude::*;

use crate::{
    animal::AnimalHandler,
    camera_controller::CameraController,
    localization::LocaleHandler,
    map::{Map, TILE_SCALE, TILE_SIZE},
    pause_menu::{GameSettigns, PauseMenu},
    player::Player,
    shop_ui::Canvas,
    texture_handler::TextureHandler,
    tutorial::Tutorial,
    upgrades::UpgradeHandler,
    worker::WorkerHandler,
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
    settings: &GameSettigns,
    locale_handler: &LocaleHandler,
) {
    let mut d2 = rl.begin_mode2D(camera_controller.camera);

    map.draw(
        &mut d2,
        &texture_handler.textures,
        worker_handler,
        animal_handler,
        font,
        settings,
        locale_handler,
    );

    if !map.dynamic_data.tiles.contains_key(&selected_tile) {
        return;
    }

    // draw tile selection box
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
    selected_tile: (i32, i32),
    settings: &GameSettigns,
) {
    draw_placing_tooltip(rl, font, map, canvas, upgrade_handler, selected_tile);

    player.draw_stats(rl, font, locale_handler, settings);

    canvas.draw(rl, map, animal_handler, texture_handler, player, font);
    canvas.update(
        rl,
        map,
        animal_handler,
        player,
        font,
        locale_handler,
        &upgrade_handler,
        settings,
    );

    upgrade_handler.draw(
        rl,
        texture_handler.textures.get("upgrades").unwrap(),
        font,
        player,
        locale_handler,
        settings,
    );

    tutorial.draw(rl, font);

    pause_menu.draw(rl, font, master_volume, locale_handler);
}

fn draw_placing_tooltip(
    rl: &mut RaylibDrawHandle,
    font: &Font,
    map: &Map,
    canvas: &Canvas,
    upgrade_handler: &UpgradeHandler,
    selected_tile: (i32, i32),
) {
    if map.dynamic_data.tiles.contains_key(&selected_tile)
        && !canvas.blocks_mouse(rl.get_mouse_position())
        && !upgrade_handler.ui_blocks_mouse
    {
        let sel = canvas.selected;
        let toolbar_static = &canvas.toolbar_data.static_data;
        let (label, price) = match canvas.mode {
            crate::shop_ui::MenuMode::Crops => (
                toolbar_static.crops[sel].tooltip.clone(),
                canvas.toolbar_data.get_price_for_crop(sel),
            ),
            crate::shop_ui::MenuMode::Trees => (
                toolbar_static.trees[sel].tooltip.clone(),
                canvas.toolbar_data.get_price_for_tree(sel),
            ),
            crate::shop_ui::MenuMode::Animals => (
                toolbar_static.animals[sel].tooltip.clone(),
                canvas.toolbar_data.get_price_for_animal(sel),
            ),
            crate::shop_ui::MenuMode::Beekeeping => (
                toolbar_static.beekeeping[sel].tooltip.clone(),
                canvas.toolbar_data.get_price_for_beekeeping(sel),
            ),
            crate::shop_ui::MenuMode::Misc => (
                toolbar_static.misc[sel].tooltip.clone(),
                canvas.toolbar_data.get_price_for_misc(sel),
            ),
        };

        let text = if price > 0 {
            label.to_owned() + "\n" + &price.to_string()
        } else {
            label.to_string()
        };

        let position = rl.get_mouse_position() + Vector2::new(0., -48.);

        rl.draw_rectangle_v(
            position,
            Vector2::new(
                text.lines()
                    .max_by(|&a, &b| a.chars().count().cmp(&b.chars().count()))
                    .unwrap()
                    .chars()
                    .count() as f32
                    * 12.,
                48.,
            ),
            Color::BLACK.alpha(0.75),
        );

        rl.draw_text_ex(font, &text, position, 24., 0., Color::WHITE);
    }
}
