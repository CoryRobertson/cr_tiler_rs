use crate::game_state::TileGameState;
use crate::tile::TILE_WIDTH;
use macroquad::prelude::*;
use std::process::exit;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

mod game_state;
mod tile;

/// The height that the bar to hit tiles is on the screen
const HIT_BAR: f32 = 550.0;
/// The height of the hit bar, larger hit distance would make hitting tiles easier
const HIT_DISTANCE: f32 = 25.0;
/// The width of the hit bar
const HIT_BAR_WIDTH: f32 = TILE_WIDTH * SLOT_COUNT as f32;
/// The location on the y axis representing the middle of the hit bar, used for distance calculations
const MIDDLE_BAR: f32 = HIT_BAR + (HIT_DISTANCE / 2.0);
/// The number of slots for tiles, the width of the game
const SLOT_COUNT: u8 = 3;
/// The colors representing each tile hit bar
const COLORS: [Color; SLOT_COUNT as usize] = [ORANGE, BLUE, PURPLE];
/// The duration in seconds representing how long a key press is held
const HIT_LENGTH: f32 = 0.4;

/// const fn to check if the given index would be a out of bounds when referencing a color for a slot.
/// -> See slot press time update block
const fn slot_count_check(index: usize) -> bool {
    index < SLOT_COUNT as usize
}

#[macroquad::main("cr_tile_game")]
async fn main() {
    let mut state = TileGameState::default();
    loop {
        clear_background(GRAY);

        if is_key_pressed(KeyCode::Escape) {
            exit(0);
        }

        #[cfg(debug_assertions)]
        if is_key_pressed(KeyCode::B) {
            state.add_tile();
        }

        // draw hit bar and take input for hit bar
        {
            draw_rectangle(0.0, HIT_BAR, HIT_BAR_WIDTH, HIT_DISTANCE, BLACK); // draw the hit bar

            // slot press time updates for key presses
            {
                if is_key_pressed(KeyCode::Q) && slot_count_check(0) {
                    state.slot_press_time[0] = SystemTime::now();
                }

                if is_key_pressed(KeyCode::W) && slot_count_check(1) {
                    state.slot_press_time[1] = SystemTime::now();
                }

                if is_key_pressed(KeyCode::E) && slot_count_check(2) {
                    state.slot_press_time[2] = SystemTime::now();
                }

                if is_key_pressed(KeyCode::R) && slot_count_check(3) {
                    state.slot_press_time[3] = SystemTime::now();
                }
            }

            // make iterators for color and slot time
            let color_iter = COLORS.iter();
            let mut slot_time_iter = state.slot_press_time.iter();
            // combine both iterators laterally -> (color,slot time)
            let combined = color_iter.map(|color| (color, slot_time_iter.next().unwrap()));

            // iterate over each slot time and color, and if the time listed as the last time the slot was pressed is less than HIT_LENGTH in time, then draw the respective color.
            combined.enumerate().for_each(|(index, (color, time))| {
                if SystemTime::now()
                    .duration_since(*time)
                    .unwrap()
                    .as_secs_f32()
                    < HIT_LENGTH
                {
                    draw_rectangle(
                        index as f32 * (HIT_BAR_WIDTH / SLOT_COUNT as f32),
                        HIT_BAR,
                        HIT_BAR_WIDTH / SLOT_COUNT as f32,
                        HIT_DISTANCE,
                        *color,
                    );

                    // remove tiles which are hit and hit on the slot that is hitting them.
                    state.tiles = state.tiles.clone().into_iter()
                        .filter(|tile| {
                            (!tile.is_hit(index as u8))
                            // (!tile.is_hit(index) || index != tile.slot as usize)
                        })
                        .collect();
                }
            });
        }

        #[cfg(debug_assertions)]
        draw_text(
            &format!("{},{}", mouse_position().0, mouse_position().1),
            50.0,
            50.0,
            20.0,
            BLACK,
        );

        // state management
        {
            state.draw_tiles();
            state.tick_tiles();
            state.cleanup_tiles();
        }

        frame_delay().await; // wait enough time to limit the fps of the game to 60 fps.

        next_frame().await
    }
}

/// Delays the frame to be 60 fps
async fn frame_delay() {
    let minimum_frame_time = 1. / 60.;
    let frame_time = get_frame_time();
    if frame_time < minimum_frame_time {
        let time_to_sleep = (minimum_frame_time - frame_time) * 1000.;
        sleep(Duration::from_millis(time_to_sleep as u64));
    }
}
