use crate::tile::Tile;
use crate::SLOT_COUNT;
use std::time::SystemTime;

/// A struct representing the entire game state.
pub struct TileGameState {
    /// A vector containing all the tiles that are in the game
    pub tiles: Vec<Tile>,
    /// An array that is the size of the number of slots in the game, which has contents that represent the time since the last slot has been pressed
    pub slot_press_time: [SystemTime; SLOT_COUNT as usize],

    time_since_tile: SystemTime,

    tile_spawn_time: f32,
}

impl Default for TileGameState {
    fn default() -> Self {
        Self {
            tiles: vec![],
            slot_press_time: TileGameState::new_slot_press_time(),
            time_since_tile: SystemTime::now(),
            tile_spawn_time: 1.5,
        }
    }
}

impl TileGameState {
    /// Returns a new basic slot press time, which is an array of system times representing the time since that slot has been pressed.
    pub const fn new_slot_press_time() -> [SystemTime; SLOT_COUNT as usize] {
        [SystemTime::UNIX_EPOCH; SLOT_COUNT as usize]
    }

    /// Adds a tile to the g ame state.
    pub fn add_tile(&mut self) {
        self.tiles.push(Tile::random_new());
    }

    /// Renders every tile in the struct.
    pub fn draw_tiles(&self) {
        self.tiles.iter().for_each(|tile| {
            tile.draw();
        });
    }

    /// Ticks all the tiles in the game state.
    pub fn tick_tiles(&mut self) {
        self.tiles.iter_mut().for_each(|tile| tile.tick());
        let time = SystemTime::now().duration_since(self.time_since_tile).unwrap().as_secs_f32();
        if time >= self.tile_spawn_time {
            self.add_tile();
            self.time_since_tile = SystemTime::now();
        }
    }

    /// Removes all the tiles that are off screen.
    pub fn cleanup_tiles(&mut self) {
        self.tiles = self
            .tiles
            .clone()
            .into_iter()
            .filter(|tile| tile.distance < 600.0)
            .collect();
    }
}
