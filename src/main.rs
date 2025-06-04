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

    let mut camera = Camera2D {
        target: Vector2::zero(),
        offset: Vector2 {
            x: SCREEN_WIDTH as f32 / 2.,
            y: SCREEN_HEIGHT as f32 / 2.,
        },
        zoom: 1.0,
        rotation: 0.0,
    };

    let mut canvas = Canvas::new();

    // canvas.add_node(UiNode::new(
    //     Rectangle {
    //         x: 100.,
    //         y: 100.,
    //         width: 150.,
    //         height: 150.,
    //     },
    //     Color::YELLOW,
    //     Some(UiFeature::Button {
    //         normal: Color::YELLOW,
    //         hovered: Color::ORANGE,
    //         pressed: Color::DARKORANGE,
    //     }),
    // ));

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

        camera.target = Vector2 {
            x: lerp(
                camera.target.x,
                camera_controller.position.x,
                10.0 * rl.get_frame_time(),
            ),
            y: lerp(
                camera.target.y,
                camera_controller.position.y,
                10.0 * rl.get_frame_time(),
            ),
        };

        if rl.is_window_resized() {
            camera.offset = Vector2 {
                x: rl.get_screen_width() as f32 / 2.,
                y: rl.get_screen_height() as f32 / 2.,
            };
        }

        let world_pos = rl.get_screen_to_world2D(rl.get_mouse_position(), camera);

        let selected_tile = (
            (world_pos.x / TILE_SIZE as f32).floor() as i32,
            (world_pos.y / TILE_SIZE as f32).floor() as i32,
        );

        if !canvas.blocks_mouse(rl.get_mouse_position()) {
            if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
                match canvas.mode {
                    MenuMode::Crops => {
                        plant_crops(&canvas, &mut map, &selected_tile, &mut player);
                    }
                    MenuMode::Misc => {
                        if player.money >= 100 && canvas.selected == 0 {
                            workers.push(Worker::new(
                                workers.len(),
                                selected_tile.0,
                                selected_tile.1,
                            ));
                            player.money -= 100;
                        }

                        if canvas.selected == 1 {
                            let Some(tile) = map.tiles.get_mut(&selected_tile) else {
                                return;
                            };

                            match tile {
                                TileType::Grass => {}
                                TileType::Farmland { .. } => {
                                    *tile = TileType::Grass;
                                }
                            }
                        }
                    }
                }
            }
        }

        let money_diff = player.money as isize - player.display_money as isize;
        if money_diff != 0 {
            player.display_money =
                (player.display_money as isize + money_diff / money_diff.abs()) as usize;
        }

        if timer >= tile_update_time {
            timer = 0.;
            map.update_tiles();

            workers.iter_mut().for_each(|worker| {
                player.money += worker.follow_path(&mut map);
            });
        }

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::LIGHTBLUE);

        d.draw_mode2D(camera, |mut d2, _| {
            map.draw(&mut d2, &texture_handler.textures);
            workers.iter_mut().for_each(|worker| {
                worker.draw(&mut d2, texture_handler.textures.get("worker").unwrap())
            });
        });

        // let mut crop_toggle = "".to_string();
        // for crop in map.crops_data.iter() {
        //     crop_toggle += &format!("{} ${}", crop.name, crop.buy_price).to_string();
        //     if crop.id < map.crops_data.len() - 1 {
        //         crop_toggle += "\n";
        //     }
        // }

        // d.gui_toggle_group(
        //     Rectangle::new(2., 60., 92., 48.),
        //     &crop_toggle,
        //     &mut selected_crop,
        // );

        d.draw_rectangle(10, 10, 24 * 4, 28 * 2, Color::BLACK.alpha(0.5));
        d.draw_text(&format!("{} fps", d.get_fps()), 14, 14, 24, Color::GRAY);
        d.draw_text(
            &format!("${}", player.display_money),
            14,
            38,
            24,
            Color::WHITE,
        );

        canvas.update(&mut d);
        canvas.draw(&mut d, &map, &texture_handler);
    }
}

fn plant_crops(canvas: &Canvas, map: &mut Map, selected_tile: &(i32, i32), player: &mut Player) {
    let Some(tile) = map.tiles.get_mut(selected_tile) else {
        return;
    };

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
                let crop_datum = &map.crops_data[canvas.selected];
                if player.money >= crop_datum.buy_price {
                    player.money -= crop_datum.buy_price;
                    // plant the seed
                    *crop = Some(canvas.selected);
                    *stage = 0;
                }
            }
        }
    }
}
