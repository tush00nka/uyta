use std::collections::HashMap;
use std::fs;

use raylib::prelude::*;

const SCREEN_WIDTH: i32 = 1280;
const SCREEN_HEIGHT: i32 = 720;

mod texture_handler;
use crate::pause_menu::{ButtonState, PauseMenu, PauseMenuState};
use crate::texture_handler::TextureHandler;

mod map;
use crate::map::{Map, TILE_PIXEL_SIZE, TILE_SCALE, TILE_SIZE, TileType};

mod camera_controller;
use crate::camera_controller::CameraController;

mod player;
use crate::player::Player;

mod worker;
use crate::ui::{Canvas, MenuMode};
use crate::worker::Worker;

mod pause_menu;
mod ui;

mod utils;

fn init_shader(shader: &mut Shader, rl: &mut RaylibHandle) {
    let freq_xloc = shader.get_shader_location("freqX");
    let freq_yloc = shader.get_shader_location("freqY");
    let amp_xloc = shader.get_shader_location("ampX");
    let amp_yloc = shader.get_shader_location("ampY");
    let speed_xloc = shader.get_shader_location("speedX");
    let speed_yloc = shader.get_shader_location("speedY");

    let freq_x = 20.0;
    let freq_y = 10.0;
    let amp_x = 0.5;
    let amp_y = 2.0;
    let speed_x = 5.0;
    let speed_y = 2.0;

    let screen_size = Vector2::new(rl.get_screen_width() as f32, rl.get_screen_height() as f32);
    shader.set_shader_value(shader.get_shader_location("size"), screen_size);
    shader.set_shader_value(freq_xloc, freq_x);
    shader.set_shader_value(freq_yloc, freq_y);
    shader.set_shader_value(amp_xloc, amp_x);
    shader.set_shader_value(amp_yloc, amp_y);
    shader.set_shader_value(speed_xloc, speed_x);
    shader.set_shader_value(speed_yloc, speed_y);
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .resizable()
        .title("Уйта")
        .build();

    let rl_audio = RaylibAudio::init_audio_device().expect("error init audio device");

    let mut sounds = HashMap::new();

    let filenames = fs::read_dir("static/sfx/").unwrap();
    for filename in filenames {
        let file = match filename {
            Ok(f) => f,
            Err(e) => panic!("couldn't load this particular sfx {e}"),
        };

        let name = file
            .file_name()
            .into_string()
            .unwrap()
            .split('.')
            .next()
            .unwrap()
            .to_string();

        let sound: Sound = rl_audio
            .new_sound(file.path().to_str().unwrap())
            .expect("error loading this particular sound");
        sounds.insert(name, sound);
    }

    let texture_handler = TextureHandler::new(&mut rl, &thread);
    let mut camera_controller = CameraController::new();
    let mut map = Map::new();
    let mut player = Player::new();
    let mut workers = vec![Worker::new(0, 0, 0)];
    let mut canvas = Canvas::new();
    let mut pause_menu = PauseMenu::new(&mut rl);

    let font = rl
        .load_font_ex(
            &thread,
            "static/tilita.ttf",
            32,
            Some("АБВГДЕЁЖЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯабвгдеёжзийклмнопрстуфхцчшщъыьэюя0123456789+-"),
        )
        .expect("no font???");

    rl.set_target_fps(
        get_monitor_refresh_rate(get_current_monitor())
            .try_into()
            .unwrap(),
    );

    rl.set_exit_key(None);

    let image = Image::gen_image_checked(
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        TILE_PIXEL_SIZE,
        TILE_PIXEL_SIZE,
        Color::CYAN.alpha(0.7),
        Color::CYAN.alpha(0.6),
    );

    // let bg_texture = rl
    //     .load_texture(&thread, "static/textures/water.png")
    //     .expect("err");

    let bg_texture = rl.load_texture_from_image(&thread, &image).expect("err");

    let mut shader = rl.load_shader(&thread, None, Some("static/shaders/wave.fs"));
    init_shader(&mut shader, &mut rl);

    let seconds_loc = shader.get_shader_location("seconds");
    let mut seconds = 0.;

    let tile_update_time = 0.5;
    let mut timer = 0.0;

    while !rl.window_should_close() {
        seconds += rl.get_frame_time();
        timer += rl.get_frame_time();

        shader.set_shader_value(seconds_loc, seconds);

        pause_menu.toggle_pause(&mut rl);
        let pause_blocks_mouse = pause_menu.update_buttons(&mut rl);

        match pause_menu.state {
            PauseMenuState::Main => {
                // todo: replace with a map maybe
                if pause_menu.buttons[1].state == ButtonState::Pressed {
                    break;
                }

                if pause_menu.buttons[0].state == ButtonState::Pressed {
                    pause_menu.switch_state(&mut rl, PauseMenuState::Settings);
                }
            }
            PauseMenuState::Settings => {
                if pause_menu.buttons[0].state == ButtonState::Pressed {
                    rl_audio.set_master_volume((rl_audio.get_master_volume() - 0.1).min(0.));
                }
                if pause_menu.buttons[1].state == ButtonState::Pressed {
                    rl_audio.set_master_volume((rl_audio.get_master_volume() + 0.1).max(1.));
                }
                if pause_menu.buttons[2].state == ButtonState::Pressed {
                    rl.toggle_fullscreen();
                }
                if pause_menu.buttons[3].state == ButtonState::Pressed {
                    pause_menu.switch_state(&mut rl, PauseMenuState::Main);
                }
            }
        }

        camera_controller.update_position(&mut rl);

        if !pause_blocks_mouse {
            handle_input(
                &mut rl,
                &camera_controller,
                &canvas,
                &mut map,
                &mut player,
                &mut workers,
            );
        }

        player.update_money();
        player.update_exp(&sounds);

        // call on tick
        if timer >= tile_update_time {
            timer = 0.;

            map.update_tiles();

            workers.iter_mut().for_each(|worker| {
                // feels weird and illegal
                let (money, exp) = worker.follow_path(&mut map, &sounds);
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
            &pause_menu,
            &font,
            &mut shader,
            &bg_texture,
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
            MenuMode::Trees => {
                plant_trees(canvas, map, &selected_tile, player);
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
    pause_menu: &PauseMenu,
    font: &Font,
    bg_shader: &mut Shader,
    bg_texture: &Texture2D,
) {
    let mut d = rl.begin_drawing(&thread);
    d.clear_background(Color::BLACK);

    // d.draw_texture(texture, 0, 0, Color::WHITE);

    let world_pos = d.get_screen_to_world2D(d.get_mouse_position(), camera_controller.camera);
    let selected_tile = (
        (world_pos.x / TILE_SIZE as f32).floor() as i32,
        (world_pos.y / TILE_SIZE as f32).floor() as i32,
    );

    d.draw_shader_mode(bg_shader, |mut shader| {
        shader.draw_texture_ex(bg_texture, Vector2::zero(), 0., 2., Color::WHITE);
        // shader.draw_texture(bg_texture, bg_texture.width, 0, Color::WHITE);
    });

    d.draw_mode2D(camera_controller.camera, |mut d2, _| {
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
    });

    d.draw_rectangle(10, 10, 130, 28, Color::BLACK.alpha(0.5));
    d.draw_text_ex(
        font,
        &format!("{}", player.display_money),
        Vector2::new(14., 14.),
        24.,
        0.,
        Color::WHITE,
    );

    let exp_bar_fill = player.exp as f32 / player.exp_to_lvl_up as f32;
    d.draw_rectangle(
        d.get_screen_width() / 4,
        10,
        d.get_screen_width() / 2,
        24,
        Color::BLACK.alpha(0.5),
    );
    d.draw_rectangle(
        d.get_screen_width() / 4,
        10,
        (exp_bar_fill * (d.get_screen_width() / 2) as f32) as i32,
        24,
        Color::DARKORANGE,
    );
    d.draw_text_ex(
        font,
        &format!("Уровень {}", player.level),
        Vector2::new(d.get_screen_width() as f32 / 4. + 10., 10.),
        24.,
        0.,
        Color::WHITE,
    );

    canvas.draw(&mut d, &map, &texture_handler, player, font);
    canvas.update(&mut d, player, font);

    pause_menu.draw(&mut d, font);
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
            let price = canvas.toolbar_data.crops[canvas.selected].price;
            if player.money >= price {
                player.money -= price;
                *tile = TileType::Farmland {
                    crop: canvas.selected,
                    stage: 0,
                };
            }
        }
        TileType::Farmland { crop, stage } => {
            if canvas.mode != MenuMode::Crops {
                return;
            }

            if *crop != canvas.selected {
                let price = canvas.toolbar_data.crops[canvas.selected].price;
                if player.money >= price {
                    player.money -= price;
                    // plant the seed
                    *crop = canvas.selected;
                    *stage = 0;
                }
            }
        }
        TileType::Tree { .. } => {}
    }
}

fn plant_trees(canvas: &Canvas, map: &mut Map, selected_tile: &(i32, i32), player: &mut Player) {
    let Some(tile) = map.tiles.get_mut(selected_tile) else {
        return;
    };

    if let Some(occ_tile) = map.occupation_map.get_mut(selected_tile) {
        *occ_tile = false;
    }

    match tile {
        TileType::Grass => {
            let price = canvas.toolbar_data.trees[canvas.selected].price;
            if player.money >= price {
                player.money -= price;
                *tile = TileType::Tree {
                    tree: canvas.selected,
                    grow: 0,
                    stage: 0,
                };
            }
        }
        _ => {}
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
            TileType::Tree { .. } => {
                *tile = TileType::Grass;
            }
            TileType::Farmland { .. } => {
                *tile = TileType::Grass;
            }
        }
    }
}
