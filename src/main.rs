use raylib::prelude::*;

const SCREEN_WIDTH: i32 = 1280;
const SCREEN_HEIGHT: i32 = 720;

mod texture_handler;
use crate::texture_handler::TextureHandler;

mod map;
use crate::map::{Map, TILE_SIZE, TileType};

mod camera_controller;
use crate::camera_controller::CameraController;

mod player;
use crate::player::Player;

mod worker;
use crate::ui::{Canvas, MenuMode};
use crate::worker::Worker;

mod ui;

mod utils;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .resizable()
        .title("Уйта")
        .build();

    let texture_handler = TextureHandler::new(&mut rl, &thread);
    let mut camera_controller = CameraController::new();
    let mut map = Map::new();
    let mut player = Player::new();
    let mut workers = vec![Worker::new(0, 0, 0)];
    let mut canvas = Canvas::new();

    rl.set_target_fps(
        get_monitor_refresh_rate(get_current_monitor())
            .try_into()
            .unwrap(),
    );

    let tile_update_time = 0.5;
    let mut timer = 0.0;

    while !rl.window_should_close() {
        timer += rl.get_frame_time();

        camera_controller.update_position(&mut rl);

        handle_input(
            &mut rl,
            &camera_controller,
            &canvas,
            &mut map,
            &mut player,
            &mut workers,
        );

        player.update_money();
        player.update_exp();

        // call on tick
        if timer >= tile_update_time {
            timer = 0.;

            map.update_tiles();

            workers.iter_mut().for_each(|worker| {
                // feels weird and illegal
                let (money, exp) = worker.follow_path(&mut map);
                player.money += money;
                player.exp += exp;
            });
        }

        draw(
            &thread,
            &mut rl,
            &mut canvas,
            &map,
            &camera_controller,
            &texture_handler,
            &mut workers,
            &player,
        );
    }
}

fn handle_input(
    rl: &mut RaylibHandle,
    camera_controller: &CameraController,
    canvas: &Canvas,
    map: &mut Map,
    player: &mut Player,
    workers: &mut Vec<Worker>,
) {
    let world_pos = rl.get_screen_to_world2D(rl.get_mouse_position(), camera_controller.camera);

    let selected_tile = (
        (world_pos.x / TILE_SIZE as f32).floor() as i32,
        (world_pos.y / TILE_SIZE as f32).floor() as i32,
    );

    if !canvas.blocks_mouse(rl.get_mouse_position())
        && rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
    {
        match canvas.mode {
            MenuMode::Crops => {
                plant_crops(&canvas, map, &selected_tile, player);
            }
            MenuMode::Misc => {
                perform_misc(player, &canvas, workers, &selected_tile, map);
            }
        }

        map.buy_land(selected_tile, player);
    }
}

fn draw(
    thread: &RaylibThread,
    rl: &mut RaylibHandle,
    canvas: &mut Canvas,
    map: &Map,
    camera_controller: &CameraController,
    texture_handler: &TextureHandler,
    workers: &mut Vec<Worker>,
    player: &Player,
) {
    let mut d = rl.begin_drawing(&thread);
    d.clear_background(Color::LIGHTBLUE);

    d.draw_mode2D(camera_controller.camera, |mut d2, _| {
        map.draw(&mut d2, &texture_handler.textures);
        workers.iter_mut().for_each(|worker| {
            worker.draw(&mut d2, texture_handler.textures.get("worker").unwrap())
        });
    });

    d.draw_rectangle(10, 10, 24 * 4, 28, Color::BLACK.alpha(0.5));
    // d.draw_text(&format!("{} fps", d.get_fps()), 14, 14, 24, Color::GRAY);
    d.draw_text(
        &format!("{}", player.display_money),
        14,
        14,
        24,
        Color::WHITE,
    );

    let exp_bar_fill = player.exp as f32 / player.exp_to_lvl_up as f32;
    d.draw_rectangle(
        d.get_screen_width() / 4,
        10,
        d.get_screen_width() / 2,
        24,
        Color::GRAY,
    );
    d.draw_rectangle(
        d.get_screen_width() / 4,
        10,
        (exp_bar_fill * (d.get_screen_width() / 2) as f32) as i32,
        24,
        Color::DARKORANGE,
    );
    d.draw_text(
        &format!("Level {}", player.level),
        d.get_screen_width() / 4 + 10,
        10,
        24,
        Color::WHITE,
    );

    canvas.draw(&mut d, &map, &texture_handler, player);
    canvas.update(&mut d, player);
}

fn plant_crops(canvas: &Canvas, map: &mut Map, selected_tile: &(i32, i32), player: &mut Player) {
    let Some(tile) = map.tiles.get_mut(selected_tile) else {
        return;
    };

    if let Some(occ_tile) = map.occupation_map.get_mut(selected_tile) {
        *occ_tile = false;
    }

    match tile {
        TileType::Grass => {
            *tile = TileType::Farmland {
                crop: None,
                stage: 0,
            };
        }
        TileType::Farmland { crop, stage } => {
            if canvas.mode != MenuMode::Crops {
                return;
            }

            if crop.is_none() || crop.unwrap() != canvas.selected {
                let price = canvas.toolbar_data.crops[canvas.selected].price;
                if player.money >= price {
                    player.money -= price;
                    // plant the seed
                    *crop = Some(canvas.selected);
                    *stage = 0;
                }
            }
        }
    }
}

fn perform_misc(
    player: &mut Player,
    canvas: &Canvas,
    workers: &mut Vec<Worker>,
    selected_tile: &(i32, i32),
    map: &mut Map,
) {
    if canvas.selected == 0 && player.money >= 100 && map.tiles.contains_key(selected_tile) {
        workers.push(Worker::new(workers.len(), selected_tile.0, selected_tile.1));
        player.money -= canvas.toolbar_data.misc[canvas.selected].price;
    }

    if canvas.selected == 1 {
        let Some(tile) = map.tiles.get_mut(&selected_tile) else {
            return;
        };

        if let Some(occ_tile) = map.occupation_map.get_mut(&selected_tile) {
            *occ_tile = false;
        }

        match tile {
            TileType::Grass => {}
            TileType::Farmland { .. } => {
                *tile = TileType::Grass;
            }
        }
    }
}
