use crate::game_state::GameState::Playing;
use crate::tile::Tile;
use crate::{ANTI_TICK_SOUND, SLOT_COUNT};
use cr_tile_game_service::packet::{GameDataPacket, LoginInfo};
use macroquad::audio::play_sound_once;
use macroquad::prelude::request_new_screen_size;
use rand::prelude::SliceRandom;
use std::net::TcpStream;
use std::sync::atomic::Ordering;
use std::time::SystemTime;

#[derive(PartialEq, Eq, Clone)]
/// The state representing the player is doing.
pub enum GameState {
    MainMenu,
    Playing(Difficulty),
    ScoreScreen,
}

#[derive(Clone, Eq, PartialEq)]
pub enum Difficulty {
    Normal,
    Hard,
}

/// A struct representing the entire game state.
pub struct TileGameState {
    /// The state representing what should be going on e.g. at main menu, or playing in normal mode, or seeing the score screen.
    pub state: GameState,

    /// A vector containing all the tiles that are in the game
    pub tiles: Vec<Tile>,
    /// An array that is the size of the number of slots in the game, which has contents that represent the time since the last slot has been pressed
    pub slot_press_time: Vec<SystemTime>,

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

    /// The time that the game started at
    pub game_start_time: SystemTime,

    /// The time that the game ended at
    pub game_end_time: SystemTime,

    pub client: Option<TcpStream>,

    pub login_info: LoginInfo,
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
            game_start_time: SystemTime::UNIX_EPOCH,
            game_end_time: SystemTime::now(),
            client: None,
            login_info: LoginInfo::default(),
        }
    }
}

impl TileGameState {
    pub fn to_packet(&self) -> GameDataPacket {
        GameDataPacket {
            score: self.get_score(),
            login_info: self.login_info.clone(),
        }
    }

    pub fn start_game(&mut self, difficulty: Difficulty, will_connect: bool) {
        let login = self.login_info.clone();
        *self = TileGameState::default();
        if will_connect {
            self.client = Some(TcpStream::connect("localhost:8114").unwrap());
            self.login_info = login;
        }
        if difficulty == Difficulty::Hard {
            self.tile_hit_count = 30;
            self.lives = 5;
        }
        self.state = Playing(difficulty);
        self.game_start_time = SystemTime::now();
        request_new_screen_size(
            (SLOT_COUNT.load(Ordering::Relaxed) as f32 * 100.0) + 100.0,
            600.0,
        );
        for _ in 0..(SLOT_COUNT.load(Ordering::Relaxed) as usize - self.slot_press_time.len()) {
            self.slot_press_time.push(SystemTime::UNIX_EPOCH);
        }
    }

    /// Returns the score of the player
    pub fn get_score(&self) -> i32 {
        self.tile_hit_count - self.slot_clicks
    }

    /// Returns a new basic slot press time, which is an array of system times representing the time since that slot has been pressed.
    pub const fn new_slot_press_time() -> Vec<SystemTime> {
        vec![]
    }

    /// Adds a tile to the g ame state.
    pub fn add_tile(&mut self, speed: f32) {
        self.tiles.push(Tile::random_new(speed));
    }

    /// Renders every tile in the struct.
    pub fn draw_tiles(&self) {
        // draw every tile
        self.tiles.iter().for_each(|tile| {
            tile.draw();
        });
    }

    /// Returns the tile speed that new tiles should.
    fn get_tile_speed(&self) -> f32 {
        match self.tile_hit_count {
            ..=20 => 2.0,
            21..=40 => {
                const V: [f32; 2] = [2.0, 4.0];
                match &self.state {
                    Playing(diff) => match diff {
                        Difficulty::Normal => 4.0,
                        Difficulty::Hard => *V.choose(&mut rand::thread_rng()).unwrap_or(&2.0),
                    },
                    _ => 4.0,
                }
            }
            41.. => {
                const V: [f32; 3] = [2.0, 4.0, 6.0];
                match &self.state {
                    Playing(diff) => match diff {
                        Difficulty::Normal => 6.0,
                        Difficulty::Hard => *V.choose(&mut rand::thread_rng()).unwrap_or(&2.0),
                    },
                    _ => 6.0,
                }
            }
        }
    }

    /// Updates tile spawn time based on players tile hit count.
    fn update_tile_spawn_time(&mut self) {
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
    }

    /// Ticks all the tiles in the game state.
    pub fn tick_game_state(&mut self) {
        self.update_tile_spawn_time(); // update tile spawning rate

        self.tiles.iter_mut().for_each(|tile| tile.tick()); // tick every tile

        let time = SystemTime::now()
            .duration_since(self.time_since_tile)
            .unwrap()
            .as_secs_f32();

        if time >= self.tile_spawn_time {
            // decide if we need to spawn a new tile
            self.add_tile({
                self.get_tile_speed() // get a tile speed from the game state
            });
            self.time_since_tile = SystemTime::now(); // update the time it has been since the last tile was spawned
        }
    }

    /// Removes all the tiles that are off screen.
    pub fn cleanup_tiles(&mut self) {
        self.tiles = self // set the tile list to a new tile list clone
            .tiles
            .clone()
            .into_iter()
            .filter(|tile| {
                // filter out every tile that is below the screen margin
                let distance_state = tile.distance < 600.0; // state determining if the tile is off bottom of screen

                if !distance_state {
                    // if the tile is, then play a sound, reduce the score, and remove a life
                    self.lives -= 1;
                    play_sound_once(*ANTI_TICK_SOUND.get().unwrap());
                }

                distance_state
            })
            .collect();
    }
}
