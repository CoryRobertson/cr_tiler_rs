use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameDataPacket {
    pub score: i32,
    pub login_info: LoginInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginInfo {
    pub user_name: String,
    pub key: String,
}

impl Default for LoginInfo {
    fn default() -> Self {
        Self {
            user_name: "".to_string(),
            key: "".to_string(),
        }
    }
}

impl Default for GameDataPacket {
    fn default() -> Self {
        Self {
            score: 0,
            login_info: LoginInfo::default(),
        }
    }
}

impl Display for GameDataPacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.score)
    }
}
