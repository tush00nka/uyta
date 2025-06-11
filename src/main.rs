use std::collections::HashMap;
use std::fs;

use raylib::ffi::{
    CloseAudioDevice, GetMasterVolume, InitAudioDevice, LoadSound, SetMasterVolume, Sound,
};
use raylib::prelude::*;

mod texture_handler;
use crate::pause_menu::{ButtonState, GameSettigns, PauseMenu, PauseMenuState};
use crate::texture_handler::TextureHandler;

mod map;
use crate::map::{Map, TILE_PIXEL_SIZE, TILE_SIZE};

mod camera_controller;
use crate::camera_controller::CameraController;

mod player;
use crate::player::Player;

mod worker;
use crate::tutorial::Tutorial;
use crate::ui::{Canvas, MenuMode};
use crate::worker::WorkerHandler;

mod pause_menu;
mod ui;

mod renderer;
mod tutorial;
mod utils;

const SCREEN_WIDTH: i32 = 1280;
const SCREEN_HEIGHT: i32 = 720;

const TILE_UPDATE_TIME: f32 = 0.5;

fn init_shader(shader: &mut Shader) {
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

    let screen_size = Vector2::new(SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32);
    shader.set_shader_value(shader.get_shader_location("size"), screen_size);
    shader.set_shader_value(freq_xloc, freq_x);
    shader.set_shader_value(freq_yloc, freq_y);
    shader.set_shader_value(amp_xloc, amp_x);
    shader.set_shader_value(amp_yloc, amp_y);
    shader.set_shader_value(speed_xloc, speed_x);
    shader.set_shader_value(speed_yloc, speed_y);
}

fn main() {
    let (mut rl, thread) = if cfg!(target_arch = "wasm32") {
        raylib::init().size(SCREEN_WIDTH, SCREEN_HEIGHT).title("Уйта").build()
    } else {
        raylib::init()
            .size(SCREEN_WIDTH, SCREEN_HEIGHT)
            .resizable()
            .title("Уйта")
            .build()
    };

    if !cfg!(target_arch = "wasm32") {
        unsafe { InitAudioDevice() };
    }

    // let rl_audio = RaylibAudio::init_audio_device().expect("error init audio device");
    let mut sounds = HashMap::new();
    if !cfg!(target_arch = "wasm32") {
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

            let sound: Sound =
                unsafe { LoadSound(file.path().to_str().unwrap().as_ptr() as *const i8) };
            sounds.insert(name, sound);
        }
    }

    let texture_handler = TextureHandler::new(&mut rl, &thread);
    let mut camera_controller = CameraController::new();
    let mut map = Map::new();
    let mut player = Player::new();
    let mut worker_handler = WorkerHandler::new();
    let mut canvas = Canvas::new();

    let mut game_settings = GameSettigns::new();
    // rl_audio.set_master_volume(game_settings.master_volume);

    // if game_settings.is_fullscreen && !rl.is_window_fullscreen() {
    //     rl.toggle_fullscreen();
    // }

    let mut pause_menu = PauseMenu::new(&mut rl);
    let mut tutorial = Tutorial::new();

    let font = rl
        .load_font_ex(
            &thread,
            "static/tilita.ttf",
            32,
            Some("АБВГДЕЁЖЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯабвгдеёжзийклмнопрстуфхцчшщъыьэюя0123456789+-%[]()FWASD,.!?"),
        )
        .expect("no font???");

    if cfg!(target_arch = "wasm32") {
        rl.set_target_fps(60);
    } else {
        rl.set_target_fps(
            get_monitor_refresh_rate(get_current_monitor())
                .try_into()
                .unwrap(),
        );
    }

    rl.set_exit_key(None);

    let image = Image::gen_image_checked(
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        TILE_PIXEL_SIZE,
        TILE_PIXEL_SIZE,
        Color::CYAN.alpha(0.7),
        Color::CYAN.alpha(0.6),
    );

    let bg_texture = rl.load_texture_from_image(&thread, &image).expect("err");

    let shader_path = if cfg!(target_arch = "wasm32") {
        "static/shaders/wave100.fs"
    } else {
        "static/shaders/wave330.fs"
    };

    let mut shader = rl.load_shader(&thread, None, Some(shader_path));
    init_shader(&mut shader);

    let seconds_loc = shader.get_shader_location("seconds");
    let mut seconds = 0.;

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
                if pause_menu.buttons[0].state == ButtonState::Pressed {
                    pause_menu.switch_state(&mut rl, PauseMenuState::Settings);
                }

                if !cfg!(target_arch = "wasm32") {
                    if pause_menu.buttons[1].state == ButtonState::Pressed {
                        break;
                    }
                }
            }
            PauseMenuState::Settings => {
                if pause_menu.buttons[0].state == ButtonState::Pressed {
                    unsafe {
                        let new_volume = (GetMasterVolume() - 0.1).max(0.);
                        SetMasterVolume(new_volume);
                        game_settings.master_volume = new_volume;
                    }
                    // let new_volume = (rl_audio.get_master_volume() - 0.1).max(0.);
                    // rl_audio.set_master_volume(new_volume);
                    // game_settings.master_volume = new_volume;
                }
                if pause_menu.buttons[1].state == ButtonState::Pressed {
                    unsafe {
                        let new_volume = (GetMasterVolume() + 0.1).min(1.);
                        SetMasterVolume(new_volume);
                        game_settings.master_volume = new_volume;
                    }
                    // let new_volume = (rl_audio.get_master_volume() + 0.1).min(1.);
                    // rl_audio.set_master_volume(new_volume);
                    // game_settings.master_volume = new_volume;
                }
                if !cfg!(target_arch = "wasm32") {
                    if pause_menu.buttons[2].state == ButtonState::Pressed {
                        rl.toggle_fullscreen();
                        game_settings.is_fullscreen = rl.is_window_fullscreen();
                    }
                }
                if pause_menu.buttons[3].state == ButtonState::Pressed {
                    pause_menu.switch_state(&mut rl, PauseMenuState::Main);

                    if !cfg!(target_arch = "wasm32") {
                        game_settings.save();
                    }
                }
            }
        }

        camera_controller.update_position(&mut rl, &mut tutorial);

        let world_pos = rl.get_screen_to_world2D(rl.get_mouse_position(), camera_controller.camera);
        let selected_tile = (
            (world_pos.x / TILE_SIZE as f32).floor() as i32,
            (world_pos.y / TILE_SIZE as f32).floor() as i32,
        );

        if !pause_blocks_mouse {
            handle_input(
                &mut rl,
                &canvas,
                &mut map,
                &mut player,
                &mut worker_handler,
                selected_tile,
                &mut tutorial,
            );
        }

        player.update_money();
        player.update_exp(&sounds);

        tutorial.close_tutorial(&mut rl);

        // call on tick
        if timer >= TILE_UPDATE_TIME {
            timer = 0.;

            map.update_tiles();

            worker_handler.advance_workers(&mut player, &mut map, &sounds);
        }

        let mut d = rl.begin_drawing(&thread);

        renderer::draw_bg(&mut d, &mut shader, &bg_texture);
        renderer::draw_for_camera(
            &mut d,
            &map,
            &camera_controller,
            &texture_handler,
            &mut worker_handler,
            &font,
            selected_tile,
        );
        renderer::draw_fg(
            &mut d,
            &mut canvas,
            &map,
            &texture_handler,
            &player,
            &pause_menu,
            &tutorial,
            &font,
            // rl_audio.get_master_volume(),
            unsafe { GetMasterVolume() },
        );
    }

    if !cfg!(target_arch = "wasm32") {
        unsafe { CloseAudioDevice() };
    }

    #[cfg(not(target_arch = "wasm32"))]
    save_game(&worker_handler, &player, &map);
}

#[cfg(not(target_arch = "wasm32"))]
fn save_game(worker_handler: &WorkerHandler, player: &Player, map: &Map) {
    worker_handler.save();
    player.save();
    map.save();
}

fn handle_input(
    rl: &mut RaylibHandle,
    canvas: &Canvas,
    map: &mut Map,
    player: &mut Player,
    worker_handler: &mut WorkerHandler,
    selected_tile: (i32, i32),
    tutorial: &mut Tutorial,
) {
    if !canvas.blocks_mouse(rl.get_mouse_position())
        && rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
    {
        match canvas.mode {
            MenuMode::Crops => {
                player.plant_crops(canvas, map, &selected_tile, tutorial);
            }
            MenuMode::Trees => {
                player.plant_trees(canvas, map, &selected_tile);
            }
            MenuMode::Misc => {
                player.perform_misc(canvas, worker_handler, &selected_tile, map);
            }
        }

        map.buy_land(selected_tile, player);
    }
}
