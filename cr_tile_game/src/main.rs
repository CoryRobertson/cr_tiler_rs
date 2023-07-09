#![windows_subsystem = "windows"]

use crate::game_state::{GameState, TileGameState};
use crate::tile::TILE_WIDTH;
use macroquad::audio::{load_sound_from_bytes, play_sound_once, set_sound_volume, Sound};
use macroquad::hash;
use macroquad::prelude::*;
use macroquad::ui::root_ui;
use std::iter::Iterator;
use std::process::exit;
use std::sync::OnceLock;
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
const HIT_LENGTH: f32 = 0.125;

pub static TICK_SOUND: OnceLock<Sound> = OnceLock::new();
pub static ANTITICK_SOUND: OnceLock<Sound> = OnceLock::new();
pub static FIRE_ICON: OnceLock<Texture2D> = OnceLock::new();
pub static HEART_ICON: OnceLock<Texture2D> = OnceLock::new();

/// const fn to check if the given index would be a out of bounds when referencing a color for a slot.
/// -> See slot press time update block
const fn slot_count_check(index: usize) -> bool {
    index < SLOT_COUNT as usize
}

#[macroquad::main("cr_tile_game")]
async fn main() {
    let mut state = TileGameState::default();
    let mut tick_vol = 1.0;

    // load textures and sounds
    {
        let heart_icon =
            Texture2D::from_file_with_format(include_bytes!("../assets/heart.png"), None);
        HEART_ICON.set(heart_icon).unwrap();

        let fire_icon =
            Texture2D::from_file_with_format(include_bytes!("../assets/Fire_icon.png"), None);
        FIRE_ICON.set(fire_icon).unwrap();

        let tick = load_sound_from_bytes(include_bytes!(
            "../assets/mixkit-arcade-game-jump-coin-216.wav"
        ))
        .await
        .unwrap();
        TICK_SOUND.set(tick).unwrap();

        let anti_tick = load_sound_from_bytes(include_bytes!("../assets/mixkit-arcade-game-jump-coin-216-rev.wav")).await.unwrap();
        ANTITICK_SOUND.set(anti_tick).unwrap();
    }

    request_new_screen_size((SLOT_COUNT as f32 * 100.0) + 100.0, 600.0);

    loop {
        if is_key_pressed(KeyCode::Escape) {
            exit(0);
        }

        match state.state {
            GameState::MainMenu => {
                clear_background(DARKGRAY);
                if root_ui().button(
                    Vec2::from_slice(&[(screen_width() / 2.0) - 25.0, screen_height() / 2.0]),
                    "Normal Mode",
                ) {
                    state.state = GameState::NormalMode;
                }
                if root_ui().button(
                    Vec2::from_slice(&[
                        (screen_width() / 2.0) - 25.0,
                        (screen_height() / 2.0) + 25.0,
                    ]),
                    "Hard Mode",
                ) {
                    state.state = GameState::HardMode;
                }
                if root_ui().button(None, "Quit") {
                    exit(1);
                }
                if root_ui().button(None, "Test Sound") {
                    play_sound_once(*TICK_SOUND.get().unwrap());
                }
                root_ui().slider(hash!(), "Volume", 0.0..1.0, &mut tick_vol);
                set_sound_volume(*TICK_SOUND.get().unwrap(), tick_vol);

                draw_text("Q, W, E to tap respective slot", 5.0, 400.0, 20.0, BLACK);
                draw_text(
                    "B to go back to main menu, ESC to close game",
                    5.0,
                    420.0,
                    20.0,
                    BLACK,
                );
            }
            GameState::NormalMode | GameState::HardMode => {
                clear_background(GRAY);
                if state.lives < 0 {
                    state.state = GameState::ScoreScreen;
                }

                for a in 0..state.lives {
                    draw_texture(
                        *HEART_ICON.get().unwrap(),
                        SLOT_COUNT as f32 * 100.0,
                        100.0 + (HEART_ICON.get().unwrap().height() * a as f32),
                        WHITE,
                    );
                }

                draw_text(
                    &format!("Score: {}", state.get_score()),
                    310.0,
                    50.0,
                    20.0,
                    BLACK,
                );
                #[cfg(debug_assertions)]
                draw_text(
                    &format!("DEBUG TST: {}", state.tile_spawn_time),
                    220.0,
                    70.0,
                    20.0,
                    BLACK,
                );

                #[cfg(debug_assertions)]
                if is_key_pressed(KeyCode::G) {
                    state.add_tile(4.0);
                }

                #[cfg(debug_assertions)]
                if is_key_pressed(KeyCode::H) {
                    state.lives -= 1;
                }

                #[cfg(debug_assertions)]
                if is_key_pressed(KeyCode::A) {
                    state.tile_hit_count += 10;
                }

                if state.state == GameState::HardMode {
                    draw_texture(
                        *FIRE_ICON.get().unwrap(),
                        SLOT_COUNT as f32 * 100.0,
                        screen_height() - 150.0,
                        WHITE,
                    );
                }

                // draw hit bar and take input for hit bar
                {
                    draw_rectangle(0.0, HIT_BAR, HIT_BAR_WIDTH, HIT_DISTANCE, BLACK); // draw the hit bar

                    // slot press time updates for key presses
                    {
                        if is_key_pressed(KeyCode::Q) && slot_count_check(0) {
                            state.slot_press_time[0] = SystemTime::now();
                            state.slot_clicks += 1;
                        }

                        if is_key_pressed(KeyCode::W) && slot_count_check(1) {
                            state.slot_press_time[1] = SystemTime::now();
                            state.slot_clicks += 1;
                        }

                        if is_key_pressed(KeyCode::E) && slot_count_check(2) {
                            state.slot_press_time[2] = SystemTime::now();
                            state.slot_clicks += 1;
                        }

                        if is_key_pressed(KeyCode::R) && slot_count_check(3) {
                            state.slot_press_time[3] = SystemTime::now();
                            state.slot_clicks += 1;
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
                            state.tiles = state
                                .tiles
                                .clone()
                                .into_iter()
                                .filter(|tile| {
                                    let hit_state = tile.is_hit(index as u8); // state representing if the given tile is hit

                                    if hit_state {
                                        // if the tile was hit, increment the hit count
                                        state.tile_hit_count += 2;
                                        play_sound_once(*TICK_SOUND.get().unwrap())
                                    }

                                    !hit_state // do not keep hit tiles, thus filtering them out when they are hit
                                })
                                .collect();
                        }
                    });
                }

                // state management
                {
                    state.draw_tiles();
                    state.tick_tiles();
                    state.cleanup_tiles();
                }
            }
            GameState::ScoreScreen => {
                clear_background(GRAY);
                draw_text(
                    &format!("Final score: {}", state.get_score()),
                    50.0,
                    50.0,
                    20.0,
                    BLACK,
                );
                draw_text("Press B to go back to main menu", 50.0, 70.0, 20.0, BLACK);
            }
        }

        if is_key_pressed(KeyCode::B) {
            state = TileGameState::default();
        }

        #[cfg(debug_assertions)]
        draw_text(
            &format!("DEBUG {},{}", mouse_position().0, mouse_position().1),
            50.0,
            30.0,
            20.0,
            BLACK,
        );

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
