//! game_settings is a source file containing the code to create and implement the GameSettings struct
#![warn(missing_docs)]

use cr_program_settings::{load_settings, save_settings};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
/// Struct representing items to save and load on program startup
pub struct GameSettings {
    /// The ip address in the ip field
    pub ip_address: String,
    /// The username the user logs in with
    pub username: String,
    /// The key the user uses to log in with
    pub key: String,
    /// The volume that the tick noises play at
    pub volume: f32,
    /// The number of slots the game uses for gameplay
    pub slot_count: u8,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            ip_address: "localhost:8114".to_string(),
            username: "".to_string(),
            key: "".to_string(),
            volume: 1.0,
            slot_count: 3,
        }
    }
}

impl GameSettings {
    /// Saves the settings to the users home directory.
    pub fn save(&self) {
        let _ = save_settings!(&self);
    }
    /// Loads the settings from the users home directory, also does some restriction checks on the settings that get loaded.
    pub fn load() -> Self {
        let mut settings = load_settings!(GameSettings).unwrap_or_default();

        // restrict specific values so the game cant be broken that easily :P
        settings.volume = settings.volume.clamp(0.0, 1.0);
        settings.slot_count = settings.slot_count.clamp(1, 5);

        settings
    }
}
