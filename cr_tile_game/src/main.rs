#![windows_subsystem = "windows"]

use crate::game_state::{Difficulty, GameState, TileGameState};
use crate::tile::TILE_WIDTH;
use macroquad::audio::{load_sound_from_bytes, play_sound_once, set_sound_volume, Sound};
use macroquad::hash;
use macroquad::prelude::*;
use macroquad::ui::root_ui;
use std::iter::Iterator;
use std::process::exit;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::OnceLock;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

mod game_state;
mod tile;

/// The height that the bar to hit tiles is on the screen
const HIT_BAR: f32 = 550.0;
/// The height of the hit bar, larger hit distance would make hitting tiles easier
const HIT_DISTANCE: f32 = 25.0;
/// The location on the y axis representing the middle of the hit bar, used for distance calculations
const MIDDLE_BAR: f32 = HIT_BAR + (HIT_DISTANCE / 2.0);
/// The number of slots for tiles, the width of the game
static SLOT_COUNT: AtomicU8 = AtomicU8::new(3);
/// The colors representing each tile hit bar
const COLORS: [Color; 5] = [ORANGE, BLUE, PURPLE, PINK, YELLOW];
/// The duration in seconds representing how long a key press is held
const HIT_LENGTH: f32 = 0.125;
/// The keybindings relating to each slot.
const KEY_BINDS: [KeyCode; 5] = [KeyCode::Q, KeyCode::W, KeyCode::E, KeyCode::R, KeyCode::T];

pub(crate) static TICK_SOUND: OnceLock<Sound> = OnceLock::new();
pub(crate) static ANTI_TICK_SOUND: OnceLock<Sound> = OnceLock::new();
pub(crate) static FIRE_ICON: OnceLock<Texture2D> = OnceLock::new();
pub(crate) static HEART_ICON: OnceLock<Texture2D> = OnceLock::new();

/// fn to check if the given index would be a out of bounds when referencing a color for a slot.
/// -> See slot press time update block
fn slot_count_check(index: usize) -> bool {
    index < SLOT_COUNT.load(Ordering::Relaxed) as usize
}

/// Gets the color of the slot, wrapped with the length just in-case ;)
pub const fn get_color(index: usize) -> Color {
    COLORS[index % COLORS.len()]
}

/// The width of the hit bar, not a single hit bar, but the entire bar.
fn get_hit_bar_width() -> f32 {
    TILE_WIDTH * SLOT_COUNT.load(Ordering::Relaxed) as f32
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

        let anti_tick = load_sound_from_bytes(include_bytes!(
            "../assets/mixkit-arcade-game-jump-coin-216-rev.wav"
        ))
        .await
        .unwrap();
        ANTI_TICK_SOUND.set(anti_tick).unwrap();
    }

    // set screen size to a size that will show every tile
    request_new_screen_size(
        (SLOT_COUNT.load(Ordering::Relaxed) as f32 * 100.0) + 100.0,
        600.0,
    );

    loop {
        // quit game key bind
        if is_key_pressed(KeyCode::Escape) {
            exit(0);
        }

        match state.state.clone() {
            GameState::MainMenu => {
                clear_background(DARKGRAY);

                // start game buttons
                {
                    if root_ui().button(
                        Vec2::from_slice(&[(screen_width() / 2.0) - 25.0, screen_height() / 2.0]),
                        "Normal Mode",
                    ) {
                        state.start_game(Difficulty::Normal);
                    }
                    if root_ui().button(
                        Vec2::from_slice(&[
                            (screen_width() / 2.0) - 25.0,
                            (screen_height() / 2.0) + 25.0,
                        ]),
                        "Hard Mode",
                    ) {
                        state.start_game(Difficulty::Hard);
                    }
                }

                if root_ui().button(None, "Quit") {
                    exit(1);
                }
                if root_ui().button(None, "Test Sound") {
                    play_sound_once(*TICK_SOUND.get().unwrap());
                }
                root_ui().slider(hash!(), "Volume", 0.0..1.0, &mut tick_vol); // volume slider
                let slot_count_load = SLOT_COUNT.load(Ordering::Relaxed);

                // block for changing slot count
                {
                    if root_ui().button(None, "+ Slot") && slot_count_load <= 4 {
                        SLOT_COUNT.fetch_add(1, Ordering::Relaxed);
                    }
                    draw_text(
                        &format!("{}", SLOT_COUNT.load(Ordering::Relaxed)),
                        50.0,
                        80.0,
                        20.0,
                        BLACK,
                    );
                    if root_ui().button(None, "- Slot") && slot_count_load >= 2 {
                        SLOT_COUNT.fetch_sub(1, Ordering::Relaxed);
                    }
                }

                set_sound_volume(*TICK_SOUND.get().unwrap(), tick_vol);
                set_sound_volume(*ANTI_TICK_SOUND.get().unwrap(), tick_vol);

                draw_text(
                    "B to go back to main menu, ESC to close game",
                    5.0,
                    420.0,
                    20.0,
                    BLACK,
                );
            }
            GameState::Playing(difficulty) => {
                clear_background(GRAY);
                let bar_width = get_hit_bar_width();
                set_sound_volume(*TICK_SOUND.get().unwrap(), tick_vol);
                set_sound_volume(*ANTI_TICK_SOUND.get().unwrap(), tick_vol);

                // stop the game when the lives are less than 0
                if state.lives < 0 {
                    state.state = GameState::ScoreScreen;
                    state.game_end_time = SystemTime::now();
                }

                // draw each heart for every life the player has
                for a in 0..state.lives {
                    draw_texture(
                        *HEART_ICON.get().unwrap(),
                        SLOT_COUNT.load(Ordering::Relaxed) as f32 * 100.0,
                        100.0 + (HEART_ICON.get().unwrap().height() * a as f32),
                        WHITE,
                    );
                }

                draw_text(
                    &format!("Score: {}", state.get_score()),
                    bar_width,
                    50.0,
                    20.0,
                    BLACK,
                );

                #[cfg(debug_assertions)] // debug info
                {
                    draw_text(
                        &format!("DEBUG TST: {}", state.tile_spawn_time),
                        220.0,
                        70.0,
                        20.0,
                        BLACK,
                    );
                    if is_key_pressed(KeyCode::G) {
                        state.add_tile(4.0);
                    }

                    if is_key_pressed(KeyCode::H) {
                        state.lives -= 1;
                    }

                    if is_key_pressed(KeyCode::A) {
                        state.tile_hit_count += 10;
                    }
                }

                // draw fire when the difficulty is on hard mode
                if difficulty == Difficulty::Hard {
                    draw_texture(
                        *FIRE_ICON.get().unwrap(),
                        SLOT_COUNT.load(Ordering::Relaxed) as f32 * 100.0,
                        screen_height() - 150.0,
                        WHITE,
                    );
                }

                // draw hit bar and take input for hit bar
                {
                    let slot_count = SLOT_COUNT.load(Ordering::Relaxed);
                    draw_rectangle(0.0, HIT_BAR, bar_width, HIT_DISTANCE, BLACK); // draw the hit bar
                    for slot in 0..slot_count {
                        let x_value = (slot as f32 * TILE_WIDTH) // the respective x value of every slot
                            + (TILE_WIDTH / 2.0) // add half a tile width so we can put the text in the middle of a slot
                            - 5.0; // slight magic number to make the text more centered.
                        draw_text(
                            {
                                // render text based on the slot its in
                                match slot {
                                    0 => "Q",
                                    1 => "W",
                                    2 => "E",
                                    3 => "R",
                                    4 => "T",
                                    _ => "???", // unknown slot number ???
                                }
                            },
                            x_value,
                            590.0,
                            20.0,
                            BLACK,
                        );
                        draw_rectangle(
                            (slot as f32 * TILE_WIDTH) - 1.0,
                            0.0,
                            2.0,
                            screen_height(),
                            DARKGRAY,
                        );
                    }
                    draw_rectangle(
                        ((slot_count) as f32 * TILE_WIDTH) - 1.0,
                        0.0,
                        2.0,
                        screen_height(),
                        DARKGRAY,
                    );

                    // slot press time updates for key presses
                    for (index, key) in KEY_BINDS.iter().enumerate() {
                        // iterate through every key bind, checking if the respective key was pressed
                        if is_key_pressed(*key) && slot_count_check(index) {
                            match state.slot_press_time.get_mut(index) {
                                None => {
                                    // if no slot press time exists for this key, add one just in-case
                                    state.slot_press_time.push(SystemTime::now());
                                }
                                Some(time) => {
                                    // if a time does exist, update it
                                    *time = SystemTime::now();
                                }
                            }
                            state.slot_clicks += 1; // increment the slot click count when a slot is clicked
                        }
                    }

                    let slot_time_iter = state.slot_press_time.iter();
                    let combined = slot_time_iter
                        .enumerate()
                        .map(|(index, time)| (get_color(index), time)); // create an iterator that has every color and its respective time it was clicked

                    // iterate over each slot time and color, and if the time listed as the last time the slot was pressed is less than HIT_LENGTH in time, then draw the respective color.
                    combined.enumerate().for_each(|(index, (color, time))| {
                        if SystemTime::now()
                            .duration_since(*time)
                            .unwrap()
                            .as_secs_f32()
                            <= {
                                // change the hit length depending on difficulty, experimental??
                                match difficulty {
                                    Difficulty::Normal => HIT_LENGTH,
                                    Difficulty::Hard => HIT_LENGTH / 10.0,
                                }
                            }
                        {
                            // draw each slot bar
                            let x_value = index as f32 * (bar_width / slot_count as f32);
                            draw_rectangle(
                                x_value,
                                HIT_BAR,
                                bar_width / slot_count as f32,
                                HIT_DISTANCE,
                                color,
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
                    state.tick_game_state();
                    state.cleanup_tiles();
                }

                // draw border around game so it looks pretty :)
                draw_rectangle_lines(0.0, 0.0, screen_width(), screen_height(), 8.0, BLACK);
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
                let time_survived = state
                    .game_end_time
                    .duration_since(state.game_start_time)
                    .unwrap_or_default();
                draw_text(
                    &format!("Time survived: {:.2}s", time_survived.as_secs_f32()),
                    50.0,
                    90.0,
                    20.0,
                    BLACK,
                );
            }
        }

        // allow the game to be exited to the main menu
        if is_key_pressed(KeyCode::B) {
            state = TileGameState::default();
            request_new_screen_size(400.0, 600.0);
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
