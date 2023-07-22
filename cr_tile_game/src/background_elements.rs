//! background_elements is a source file that handles drawing the background of the main menu
#![warn(missing_docs)]

use crate::tile::{TILE_HEIGHT, TILE_WIDTH};
use macroquad::prelude::{draw_rectangle, draw_rectangle_lines, Color, BLACK};
use macroquad::rand::gen_range;
use macroquad::window::{screen_height, screen_width};
use rayon::prelude::*;

/// The number of tiles that get drawn
const BACKGROUND_TILE_COUNT: i32 = 20;

#[derive(PartialEq, Clone)]
/// Struct representing a singular tile drawn in the background
struct BackgroundTile {
    /// X coordinate
    x: f32,
    /// Y coordinate
    y: f32,
    /// X velocity
    x_vel: f32,
    /// Y velocity
    y_vel: f32,
    /// Color of the tile
    color: Color,
}

impl Eq for BackgroundTile {}

#[derive(PartialEq, Eq, Clone)]
/// Represents a list of background tiles, mostly a wrapper with functions to manipulate the list
pub struct BackgroundTileList {
    list: Vec<BackgroundTile>,
}

impl BackgroundTile {
    /// Creates a new background tile, random values are used
    fn new() -> Self {
        Self {
            x: gen_range(0.0, screen_width()),
            y: gen_range(-screen_height(), -screen_height() - TILE_HEIGHT),
            x_vel: gen_range(-1.0, 1.0),
            y_vel: gen_range(1.0, 5.0),
            color: Color::new(
                gen_range(0.0, 1.0),
                gen_range(0.0, 1.0),
                gen_range(0.0, 1.0),
                gen_range(0.2, 1.0),
            ),
        }
    }
    /// Steps the physics of the background tile
    fn step(&mut self) {
        self.x += self.x_vel;
        self.y += self.y_vel;
    }

    /// Resets the tile to a new randomized tile
    fn reset(&mut self) {
        *self = BackgroundTile::new();
    }

    /// Draws the tile on the screen
    fn draw(&self) {
        draw_rectangle(self.x, self.y, TILE_WIDTH, TILE_HEIGHT, self.color);

        draw_rectangle_lines(self.x, self.y, TILE_WIDTH, TILE_HEIGHT, 4.0, BLACK);
    }
}

impl BackgroundTileList {
    /// Generates a new background tile list, all tiles are randomized, the number of tiles in the list is a constant
    pub fn new() -> Self {
        Self {
            list: (0..BACKGROUND_TILE_COUNT)
                .map(|_| BackgroundTile::new())
                .collect(),
        }
    }

    /// Steps the physics of all of the tiles on screen.
    pub fn step(&mut self) {
        self.list.par_iter_mut().for_each(|tile| {
            tile.step();
            if tile.x < (0.0 - TILE_WIDTH) || tile.x > screen_width() {
                tile.reset();
            }
            if tile.y > screen_height() {
                tile.reset();
            }
        });
    }

    /// Draws every tile on the screen
    pub fn draw_all(&self) {
        self.list.iter().for_each(|tile| {
            tile.draw();
        });
    }
}
