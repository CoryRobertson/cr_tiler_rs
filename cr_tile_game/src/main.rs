use std::process::exit;
use std::thread::sleep;
use std::time::{Duration, SystemTime};
use macroquad::prelude::*;

const HIT_BAR: f32 = 550.0;
const HIT_DISTANCE: f32 = 25.0;
const TILE_WIDTH: f32 = 100.0;
const TILE_HEIGHT: f32 = 50.0;
const MIDDLE_BAR: f32 = HIT_BAR + (HIT_DISTANCE/2.0);
const SLOT_COUNT: u8 = 3;

#[derive(Debug,Clone)]
struct Tile {
    distance: f32,
    slot: u8,
    speed: f32,
}

impl Tile {

    fn random_new() -> Self {
        Self {
            distance: 350.0,
            slot: rand::gen_range(0,SLOT_COUNT),
            speed: 1.0,
        }
    }

    fn tick(&mut self) {
        self.distance += self.speed;
    }

    fn is_hit(&self) -> bool {

        let dist = {
            let middle_y = self.distance + (TILE_HEIGHT / 2.0);

            (middle_y - MIDDLE_BAR).abs()
        };

        dist <= ((TILE_HEIGHT) / 2.0)
    }

    fn draw(&self) {
        #[cfg(debug_assertions)]
        {
            let x = self.slot*100;
            let y = self.distance;
            draw_text(&format!("{},{}",x,y), x as f32, y,24.0, BLACK);
        }


        draw_rectangle((self.slot * 100) as f32, self.distance, TILE_WIDTH, TILE_HEIGHT,{
            if self.is_hit() { GREEN } else { RED }
        });
    }
}

struct TileGameState {
    tiles: Vec<Tile>,
    slot_press_time: [SystemTime ; SLOT_COUNT as usize],
}

impl Default for TileGameState {
    fn default() -> Self {
        Self{ tiles: vec![], slot_press_time: TileGameState::new_slot_press_time() }
    }
}

impl TileGameState {

    const fn new_slot_press_time() -> [SystemTime ; SLOT_COUNT as usize] {
        [SystemTime::UNIX_EPOCH ; SLOT_COUNT as usize]
    }

    fn add_tile(&mut self) {
        self.tiles.push(Tile::random_new());
    }

    fn draw_tiles(&self) {
        self.tiles.iter()
            .for_each(|tile| {
                tile.draw();
            });
    }

    fn tick_tiles(&mut self) {
        self.tiles.iter_mut()
            .for_each(|tile| {
            tile.tick()
        });
    }

    fn cleanup_tiles(&mut self) {
        self.tiles = self.tiles
            .clone()
            .into_iter()
            .filter(|tile| tile.distance < 600.0).collect();
    }
}

#[macroquad::main("cr_tile_game")]
async fn main() {
    let mut state = TileGameState::default();
    loop {
        clear_background(GRAY);

        if is_key_pressed(KeyCode::Escape) {
            exit(0);
        }

        if is_key_pressed(KeyCode::B) {
            state.add_tile();
        }

        // draw hit bar
        {
            draw_rectangle(0.0, HIT_BAR, 300.0, HIT_DISTANCE, BLACK);

            if is_key_pressed(KeyCode::Q) {
                state.slot_press_time[0] = SystemTime::now();
                // draw_rectangle(0.0, HIT_BAR,100.0,HIT_DISTANCE,ORANGE);
            }
            if is_key_pressed(KeyCode::W) {
                state.slot_press_time[1] = SystemTime::now();
                // draw_rectangle(100.0, HIT_BAR,100.0,HIT_DISTANCE,BLUE);
            }
            if is_key_pressed(KeyCode::E) {
                state.slot_press_time[2] = SystemTime::now();
                // draw_rectangle(200.0, HIT_BAR,100.0,HIT_DISTANCE,PURPLE);
            }

            const COLORS: [Color ; SLOT_COUNT as usize] = [ORANGE,BLUE,PURPLE];
            let color_iter = COLORS.iter();
            let mut slot_time_iter = state.slot_press_time.iter();
            let combined = color_iter
                .map(|color|
                    (color,slot_time_iter.next().unwrap())
                );
            combined
                .enumerate()
                .for_each(|(index,(color,time),)| {
                if SystemTime::now()
                    .duration_since(*time)
                    .unwrap()
                    .as_secs_f32() < 0.25 {
                    draw_rectangle(index as f32 * 100.0, HIT_BAR, 100.0, HIT_DISTANCE, *color);
                }
            });


        }

        draw_text(&format!("{},{}",mouse_position().0,mouse_position().1), 50.0,50.0,20.0,BLACK);

        state.draw_tiles();



        state.tick_tiles();

        state.cleanup_tiles();


        frame_delay().await;
        next_frame().await
    }
}

async fn frame_delay() {
    let minimum_frame_time = 1. / 60.;
    let frame_time = get_frame_time();
    //println!("Frame time: {}ms", frame_time * 1000.);
    if frame_time < minimum_frame_time {
        let time_to_sleep = (minimum_frame_time - frame_time) * 1000.;
        //println!("Sleep for {}ms", time_to_sleep);
        sleep(Duration::from_millis(time_to_sleep as u64));
    }
}