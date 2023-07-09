use crate::game_state::GameState::HardMode;
use crate::tile::Tile;
use crate::{ANTITICK_SOUND, SLOT_COUNT};
use rand::prelude::SliceRandom;
use std::time::SystemTime;
use macroquad::audio::play_sound_once;

#[derive(PartialEq, Eq)]
pub enum GameState {
    MainMenu,
    NormalMode,
    HardMode,
    ScoreScreen,
}

/// A struct representing the entire game state.
pub struct TileGameState {
    pub state: GameState,

    /// A vector containing all the tiles that are in the game
    pub tiles: Vec<Tile>,
    /// An array that is the size of the number of slots in the game, which has contents that represent the time since the last slot has been pressed
    pub slot_press_time: [SystemTime; SLOT_COUNT as usize],

    /// The time since the last tile was spawned
    time_since_tile: SystemTime,

    /// The number of seconds delayed between spawning tiles
    pub tile_spawn_time: f32,

    /// The number of tiles the player has hit
    pub tile_hit_count: i32,

    /// The number of lives the player has
    pub lives: i32,

    /// The number of times the player has hit a slot key
    pub slot_clicks: i32,
}

impl Default for TileGameState {
    fn default() -> Self {
        Self {
            state: GameState::MainMenu,
            tiles: vec![],
            slot_press_time: TileGameState::new_slot_press_time(),
            time_since_tile: SystemTime::now(),
            tile_spawn_time: 1.5,
            tile_hit_count: 0,
            lives: 10,
            slot_clicks: 0,
        }
    }
}

impl TileGameState {

    pub fn get_score(&self) -> i32 {
        self.tile_hit_count - self.slot_clicks
    }

    /// Returns a new basic slot press time, which is an array of system times representing the time since that slot has been pressed.
    pub const fn new_slot_press_time() -> [SystemTime; SLOT_COUNT as usize] {
        [SystemTime::UNIX_EPOCH; SLOT_COUNT as usize]
    }

    /// Adds a tile to the g ame state.
    pub fn add_tile(&mut self, speed: f32) {
        self.tiles.push(Tile::random_new(speed));
    }

    /// Renders every tile in the struct.
    pub fn draw_tiles(&self) {
        self.tiles.iter().for_each(|tile| {
            tile.draw();
        });
    }

    /// Ticks all the tiles in the game state.
    pub fn tick_tiles(&mut self) {
        match self.tile_hit_count {
            ..=-1 => {
                self.tile_spawn_time = 1.75;
            }
            0..=5 => {
                self.tile_spawn_time = 1.5;
            }
            6..=10 => {
                self.tile_spawn_time = 1.25;
            }
            11..=20 => {
                self.tile_spawn_time = 1.0;
            }
            21..=30 => {
                self.tile_spawn_time = 0.75;
            }
            31..=50 => {
                self.tile_spawn_time = 0.5;
            }
            51.. => {
                self.tile_spawn_time = 0.25;
            }
        }

        self.tiles.iter_mut().for_each(|tile| tile.tick());
        let time = SystemTime::now()
            .duration_since(self.time_since_tile)
            .unwrap()
            .as_secs_f32();
        if time >= self.tile_spawn_time {
            self.add_tile({
                match self.tile_hit_count {
                    ..=20 => 2.0,
                    21..=40 => {
                        const V: [f32; 2] = [2.0, 4.0];
                        if self.state == HardMode {
                            *V.choose(&mut rand::thread_rng()).unwrap_or(&2.0)
                        } else {
                            4.0
                        }
                    }
                    41.. => {
                        const V: [f32; 3] = [2.0, 4.0, 6.0];
                        if self.state == HardMode {
                            *V.choose(&mut rand::thread_rng()).unwrap_or(&2.0)
                        } else {
                            6.0
                        }
                    }
                }
            });
            self.time_since_tile = SystemTime::now();
        }
    }

    /// Removes all the tiles that are off screen.
    pub fn cleanup_tiles(&mut self) {
        self.tiles = self
            .tiles
            .clone()
            .into_iter()
            .filter(|tile| {
                let distance_state = tile.distance < 600.0;

                if !distance_state {
                    self.tile_hit_count -= 1;
                    self.lives -= 1;
                    play_sound_once(*ANTITICK_SOUND.get().unwrap());
                }

                distance_state
            })
            .collect();
    }
}
