use std::{
    collections::{HashMap, VecDeque},
    f32::INFINITY,
};

use raylib::prelude::*;

use crate::map::{Map, TILE_PIXEL_SIZE, TILE_SIZE, TileType};

pub struct Worker {
    #[allow(unused)]
    id: usize,
    position: (i32, i32),
    display_position: (f32, f32),
    path: Vec<(i32, i32)>,
}

#[derive(PartialEq)]
pub enum JobType {
    Harvest,
}

impl Worker {
    pub fn new(id: usize, x: i32, y: i32) -> Self {
        Self {
            id,
            position: (x, y),
            display_position: ((x * TILE_SIZE) as f32, (y * TILE_SIZE - TILE_SIZE) as f32),
            path: vec![],
        }
    }

    fn find_closest_target(&self, map: &mut Map, job: JobType) -> (i32, i32) {
        let target_position;

        match job {
            JobType::Harvest => {
                let mut closest = (INFINITY as i32, INFINITY as i32);
                let mut shortest_distance = INFINITY;

                // get closest crop tile
                for (tile_position, tile) in map.tiles.iter() {
                    if let Some(occupation_tile) = map.occupation_map.get(tile_position) {
                        if *occupation_tile {
                            // this tile is taken by other worker
                            continue;
                        }
                    }

                    match tile {
                        TileType::Grass => {}
                        TileType::Farmland { crop, stage } => {
                            if *stage >= map.crops_data[*crop].time_to_grow
                            {
                                // existing ready to harvest crop
                                let crop_position =
                                    Vector2::new(tile_position.0 as f32, tile_position.1 as f32);
                                let worker_position =
                                    Vector2::new(self.position.0 as f32, self.position.1 as f32);

                                if crop_position.distance_to(worker_position) < shortest_distance {
                                    closest = *tile_position;
                                    shortest_distance = crop_position.distance_to(worker_position);
                                }
                            }
                        }
                    }
                }

                target_position = closest;
            }
            // _ => target_position = (0, 0),
        }

        map.occupation_map.insert(target_position, true);
        target_position
    }

    pub fn follow_path(&mut self, map: &mut Map) -> (usize, usize) {
        let Some(next_position) = self.path.get(0) else {
            let tile = map.tiles.get_mut(&self.position).unwrap();
            let mut money = 0;
            let mut exp = 0;

            match tile {
                TileType::Farmland { crop, stage } => {
                    if *stage >= map.crops_data[*crop].time_to_grow {
                        // successfully complete task
                        money = map.crops_data[*crop].sell_price;
                        exp = *crop + 1; // higher crop_id == more exp
                        *stage = 0;
                        // free this tile from work
                        if let Some(occupation_tile) = map.occupation_map.get_mut(&self.position) {
                            *occupation_tile = false;
                        };
                    }
                }
                _ => {}
            }

            self.find_path(map, JobType::Harvest);
            return (money, exp);
        };

        self.position = *next_position;
        self.path.remove(0);
        return (0, 0);
    }

    pub fn find_path(&mut self, map: &mut Map, job: JobType) -> Option<Vec<(i32, i32)>> {
        let start_position = self.position;
        let target_position = self.find_closest_target(map, job);

        if !map.tiles.contains_key(&start_position) || !map.tiles.contains_key(&target_position) {
            return None;
        }

        let mut queue = VecDeque::new();
        queue.push_back(start_position);

        let mut visited = HashMap::new();
        visited.insert(start_position, None); // (position, parent)

        let directions = [(0, 1), (0, -1), (1, 0), (-1, 0)];

        while let Some(current_position) = queue.pop_front() {
            if current_position == target_position {
                let mut path = vec![];
                let mut pos = current_position;

                while let Some(parent) = visited[&pos] {
                    path.push(pos);
                    pos = parent;
                }

                path.push(start_position);
                path.reverse();
                self.path = path.clone();
                return Some(path);
            }

            for direction in directions.iter() {
                let next_position = (
                    current_position.0 + direction.0,
                    current_position.1 + direction.1,
                );

                if visited.contains_key(&next_position) || !map.tiles.contains_key(&next_position) {
                    continue;
                }

                visited.insert(next_position, Some(current_position));
                queue.push_back(next_position);
            }
        }

        None
    }

    pub fn draw(&mut self, rl: &mut RaylibDrawHandle, texture: &Texture2D) {
        let pixel_position = Vector2::new(
            (self.position.0 * TILE_SIZE) as f32,
            (self.position.1 * TILE_SIZE) as f32,
        );

        self.display_position.0 = lerp(
            self.display_position.0,
            pixel_position.x,
            5. * rl.get_frame_time(),
        );
        self.display_position.1 = lerp(
            self.display_position.1,
            pixel_position.y,
            5. * rl.get_frame_time(),
        );

        let pixel_position = Vector2::new(self.display_position.0, self.display_position.1);

        // rl.draw_texture_ex(texture, pixel_position, 0., TILE_SCALE as f32, Color::WHITE);

        let mut direction = (0, 1);
        if let Some(next_position) = self.path.get(0) {
            direction = (
                next_position.0 - self.position.0,
                next_position.1 - self.position.1,
            );
        }

        let texture_index = match direction {
            (0, 1) => 0,
            (0, -1) => 1,
            (1, 0) => 2,
            (-1, 0) => 3,
            _ => 1,
        };

        let source = Rectangle {
            x: (texture_index * TILE_PIXEL_SIZE) as f32,
            y: 0.,
            width: TILE_PIXEL_SIZE as f32,
            height: TILE_PIXEL_SIZE as f32,
        };

        let destination = Rectangle {
            x: pixel_position.x,
            y: pixel_position.y,
            width: TILE_SIZE as f32,
            height: TILE_SIZE as f32,
        };

        rl.draw_texture_pro(
            texture,
            source,
            destination,
            Vector2::zero(),
            0.,
            Color::WHITE,
        );
    }
}
