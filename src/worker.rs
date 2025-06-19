use std::{
    collections::{HashMap, VecDeque},
    f32::INFINITY,
};

use raylib::{ffi::{PlaySound, Sound}, prelude::*};
use serde::{Deserialize, Serialize};

use crate::{
    animal::AnimalHandler,
    map::{Map, TILE_PIXEL_SIZE, TILE_SIZE, TileType},
    player::Player,
    upgrades::UpgradeHandler,
    utils::parse_json,
};

#[derive(Serialize, Deserialize)]
pub struct WorkerHandler {
    pub workers: Vec<Worker>,
}

impl WorkerHandler {
    pub fn new() -> Self {
        if !cfg!(target_arch = "wasm32") {
            let res = parse_json("dynamic/workers_save.json");
            match res {
                Ok(handler) => return handler,
                Err(_) => {
                    println!("no worker");
                }
            }
        }

        Self {
            workers: vec![Worker::new(0, 0)],
        }
    }

    pub fn add_worker(&mut self, worker: Worker) {
        self.workers.push(worker);
    }

    pub fn advance_workers(
        &mut self,
        player: &mut Player,
        map: &mut Map,
        animal_handler: &AnimalHandler,
        upgrade_handler: &UpgradeHandler,
        sounds: &HashMap<String, Sound<'_>>,
    ) {
        self.workers.iter_mut().for_each(|worker| {
            // feels weird and illegal
            let (money, exp) = worker.follow_path(map, animal_handler, upgrade_handler, &sounds);
            player.money += money;
            player.exp += exp;
        });
    }

    #[cfg(not(target_arch="wasm32"))]
    pub fn save(&self) {
        let serialized = serde_json::to_string_pretty(self).expect("couldn't save workers data");
        std::fs::create_dir_all("dynamic").expect("Couldn't create dir");
        std::fs::write("dynamic/workers_save.json", serialized)
            .expect("Couldn't write map data to json file");
    }
}

#[derive(Serialize, Deserialize)]
pub struct Worker {
    pub position: (i32, i32),
    display_position: (f32, f32),
    path: Vec<(i32, i32)>,
    #[serde(skip_serializing, skip_deserializing)]
    direction: (i32, i32),
}

#[derive(PartialEq)]
pub enum JobType {
    Harvest,
}

impl Worker {
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            position: (x, y),
            display_position: (
                (x * TILE_SIZE) as f32,
                (y * TILE_SIZE - TILE_SIZE / 2) as f32,
            ),
            path: vec![],
            direction: (0, 0),
        }
    }

    fn find_closest_target(&self, map: &mut Map, job: JobType) -> (i32, i32) {
        let target_position;

        match job {
            JobType::Harvest => {
                let mut closest = (INFINITY as i32, INFINITY as i32);
                let mut shortest_distance = INFINITY;

                // get closest crop tile
                for (tile_position, tile) in map.dynamic_data.tiles.iter() {
                    if let Some(occupation_tile) =
                        map.dynamic_data.occupation_map.get(tile_position)
                    {
                        if *occupation_tile {
                            // this tile is taken by other worker
                            continue;
                        }
                    }

                    let tile_position_vec =
                        Vector2::new(tile_position.0 as f32, tile_position.1 as f32);

                    match tile {
                        TileType::Grass => {}
                        TileType::Farmland { crop, stage } => {
                            if *stage >= map.static_data.crops_data[*crop].time_to_grow {
                                let worker_position =
                                    Vector2::new(self.position.0 as f32, self.position.1 as f32);

                                if tile_position_vec.distance_to(worker_position)
                                    < shortest_distance
                                {
                                    closest = *tile_position;
                                    shortest_distance =
                                        tile_position_vec.distance_to(worker_position);
                                }
                            }
                        }
                        TileType::Tree { tree, stage, .. } => {
                            if *stage >= map.static_data.tree_data[*tree].time_to_fruit {
                                let worker_position =
                                    Vector2::new(self.position.0 as f32, self.position.1 as f32);

                                if tile_position_vec.distance_to(worker_position)
                                    < shortest_distance
                                {
                                    closest = *tile_position;
                                    shortest_distance =
                                        tile_position_vec.distance_to(worker_position);
                                }
                            }
                        }
                        TileType::AnimalDrop { .. } => {
                            let worker_position =
                                Vector2::new(self.position.0 as f32, self.position.1 as f32);

                            if tile_position_vec.distance_to(worker_position) < shortest_distance {
                                closest = *tile_position;
                                shortest_distance = tile_position_vec.distance_to(worker_position);
                            }
                        }
                    }
                }

                target_position = closest;
            } // _ => target_position = (0, 0),
        }

        map.dynamic_data
            .occupation_map
            .insert(target_position, true);
        target_position
    }

    pub fn follow_path(
        &mut self,
        map: &mut Map,
        animal_handler: &AnimalHandler,
        upgrade_handler: &UpgradeHandler,
        sounds: &HashMap<String, Sound<'_>>,
    ) -> (usize, usize) {
        if let Some(next_position) = self.path.get(0) {
            self.position = *next_position;
            self.path.remove(0);
            return (0, 0);
        }

        let tile = map.dynamic_data.tiles.get_mut(&self.position).unwrap();
        let mut money = 0;
        let mut exp = 0;

        // harvest
        match tile {
            TileType::Farmland { crop, stage } => {
                if *stage >= map.static_data.crops_data[*crop].time_to_grow {
                    let multiplier = {
                        let mut temp = 1;
                        if upgrade_handler
                            .dynamic_data
                            .purchased_upgrades
                            .contains(&(*crop * 3))
                        {
                            temp *= 2;
                        }
                        if upgrade_handler
                            .dynamic_data
                            .purchased_upgrades
                            .contains(&(*crop * 3 + 1))
                        {
                            temp *= 2;
                        }
                        if upgrade_handler
                            .dynamic_data
                            .purchased_upgrades
                            .contains(&(*crop * 3 + 2))
                        {
                            temp *= 2;
                        }

                        temp
                    };

                    money = map.static_data.crops_data[*crop].sell_price * multiplier;
                    exp = map.static_data.crops_data[*crop].exp * multiplier;
                    *stage = 0;
                    // free this tile from work
                    if let Some(occupation_tile) =
                        map.dynamic_data.occupation_map.get_mut(&self.position)
                    {
                        *occupation_tile = false;
                    };
                    if !cfg!(target_arch="wasm32") {
                        let rand = rand::random_range(0..5);
                    // let sound = sounds.get(&format!("harvest{rand}")).unwrap();
                    // sound.set_pitch(rand::random_range(0.9..1.1));
                    // sound.play();
                        unsafe { PlaySound(*sounds.get(&format!("harvest{rand}")).unwrap()) };
                    }
                }
            }
            TileType::Tree { tree, stage, .. } => {
                if *stage >= map.static_data.tree_data[*tree].time_to_fruit {
                    // we're basically offsetting the upgrade thingy, so uhh, still kinda hardcoded but idc
                    let crops_len = map.static_data.crops_data.len();
                    let multiplier = {
                        let mut temp = 1;
                        if upgrade_handler
                            .dynamic_data
                            .purchased_upgrades
                            .contains(&(crops_len * 3 + *tree * 3))
                        {
                            temp *= 2;
                        }
                        if upgrade_handler
                            .dynamic_data
                            .purchased_upgrades
                            .contains(&(crops_len * 3 + *tree * 3 + 1))
                        {
                            temp *= 2;
                        }
                        if upgrade_handler
                            .dynamic_data
                            .purchased_upgrades
                            .contains(&(crops_len * 3 + *tree * 3 + 2))
                        {
                            temp *= 2;
                        }

                        temp
                    };

                    money = map.static_data.tree_data[*tree].sell_price * multiplier;
                    exp = map.static_data.tree_data[*tree].exp * multiplier;
                    *stage = 0;

                    if let Some(occupation_tile) =
                        map.dynamic_data.occupation_map.get_mut(&self.position)
                    {
                        *occupation_tile = false;
                    };
                    let rand = rand::random_range(0..5);
                    let sound = sounds.get(&format!("harvest{rand}")).unwrap();
                    sound.set_pitch(rand::random_range(0.9..1.1));
                    sound.play();
                }
            }
            TileType::AnimalDrop { animal } => {
                let crops_len = map.static_data.crops_data.len();
                let trees_len = map.static_data.tree_data.len();
                let multiplier = {
                    let mut temp = 1;
                    if upgrade_handler
                        .dynamic_data
                        .purchased_upgrades
                        .contains(&(crops_len * 3 + trees_len * 3 + *animal * 3))
                    {
                        temp *= 2;
                    }
                    if upgrade_handler
                        .dynamic_data
                        .purchased_upgrades
                        .contains(&(crops_len * 3 + trees_len * 3 + *animal * 3 + 1))
                    {
                        temp *= 2;
                    }
                    if upgrade_handler
                        .dynamic_data
                        .purchased_upgrades
                        .contains(&(crops_len * 3 + trees_len * 3 + *animal * 3 + 2))
                    {
                        temp *= 2;
                    }

                    temp
                };

                money =
                    animal_handler.static_data.animal_data[*animal as usize].drop_cost * multiplier;
                exp = animal_handler.static_data.animal_data[*animal as usize].exp * multiplier;

                if let Some(occupation_tile) =
                    map.dynamic_data.occupation_map.get_mut(&self.position)
                {
                    *occupation_tile = false;
                };

                let sound = sounds.get(&format!("grass")).unwrap();
                sound.set_pitch(rand::random_range(0.9..1.1));
                sound.play();

                map.dynamic_data
                    .tiles
                    .insert(self.position, TileType::Grass);
            }
            _ => {}
        }

        self.find_path(map, JobType::Harvest);
        return (money, exp);
    }

    pub fn find_path(&mut self, map: &mut Map, job: JobType) -> Option<Vec<(i32, i32)>> {
        let start_position = self.position;
        let target_position = self.find_closest_target(map, job);

        if !map.dynamic_data.tiles.contains_key(&start_position)
            || !map.dynamic_data.tiles.contains_key(&target_position)
        {
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

                if visited.contains_key(&next_position)
                    || !map.dynamic_data.tiles.contains_key(&next_position)
                {
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
            10. * rl.get_frame_time(),
        );
        self.display_position.1 = lerp(
            self.display_position.1,
            pixel_position.y,
            10. * rl.get_frame_time(),
        );

        let pixel_position = Vector2::new(self.display_position.0, self.display_position.1);

        // rl.draw_texture_ex(texture, pixel_position, 0., TILE_SCALE as f32, Color::WHITE);

        if let Some(next_position) = self.path.get(0) {
            self.direction = (
                next_position.0 - self.position.0,
                next_position.1 - self.position.1,
            );
        }

        let texture_index = match self.direction {
            (0, 1) => 0,
            (0, -1) => 1,
            (1, 0) => 2,
            (-1, 0) => 3,
            _ => 4,
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
