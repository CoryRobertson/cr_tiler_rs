use crate::{get_color, HIT_DISTANCE, MIDDLE_BAR, SLOT_COUNT};
use macroquad::prelude::*;
use macroquad::shapes::draw_rectangle_lines;
use std::sync::atomic::Ordering;

/// The width of the tile for spacing purposes
pub const TILE_WIDTH: f32 = 100.0;
/// The height of the tile, for rendering and spacing, as well as duration that the tile can be hit
pub const TILE_HEIGHT: f32 = 100.0;
/// The gap between each tile slot
pub const TILE_MARGIN: f32 = 10.0;

#[derive(Debug, Clone)]
/// A struct representing the tiles within the game.
pub struct Tile {
    /// The distance of the tile to the bottom of the screen (the y coordinate)
    pub distance: f32,
    /// The slot that the tile is in
    pub slot: u8,
    /// The speed of the tile in units per frame.
    pub speed: f32,
}

impl Tile {
    /// Generates a new random tile.
    pub fn random_new(speed: f32) -> Self {
        Self {
            distance: -TILE_HEIGHT,
            slot: rand::gen_range(0, SLOT_COUNT.load(Ordering::Relaxed)),
            speed,
        }
    }

    /// Ticks the movement of the tile
    pub fn tick(&mut self) {
        self.distance += self.speed;
    }

    /// Returns true if the tile is overlapping the bar
    pub fn is_hit(&self, slot: u8) -> bool {
        if slot != self.slot {
            return false;
        }

        let dist = {
            let middle_y = self.distance + (TILE_HEIGHT / 2.0); // middle of the tile in terms of its y coordinate
            (middle_y - MIDDLE_BAR).abs()
        }; // distance from the bar in terms of absolute value
        dist <= ((TILE_HEIGHT + HIT_DISTANCE) / 2.0) // if the distance to the bar is <= the size of the bar plus the size of the tile
    }

    /// Draws the tile on the screen.
    pub fn draw(&self) {
        #[cfg(debug_assertions)]
        {
            let x = self.slot as f32 * TILE_WIDTH;
            let y = self.distance;
            draw_text(&format!("{},{}", x, y), x, y, 24.0, BLACK);
        }
        let rect_x = (self.slot as f32 * TILE_WIDTH) + (TILE_MARGIN / 2.0);
        let width = TILE_WIDTH - TILE_MARGIN;

        let color = get_color(self.slot as usize);
        let is_hit = self.is_hit(self.slot);

        draw_rectangle(rect_x, self.distance, width, TILE_HEIGHT, {
            Color::new(color.r, color.g, color.b, 0.5)
        });

        draw_rectangle_lines(
            rect_x,
            self.distance,
            width,
            TILE_HEIGHT,
            {
                // make thickness react to when the tile is hit
                match is_hit {
                    true => 6.0,
                    false => 4.0,
                }
            },
            {
                // make color react when tile is hit
                match is_hit {
                    true => BLACK,
                    false => DARKGRAY,
                }
            },
        );
    }
}
