use crate::leader_board_stat::LeaderBoardList;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientPacket {
    SubmitDataPacket(GameDataPacket),
    GetLeaderBoardsList,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerPacket {
    LeaderBoard(LeaderBoardList),
    ErrorState,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct GameDataPacket {
    pub score: i32,
    pub login_info: LoginInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginInfo {
    pub user_name: String,
    pub key: String,
}

impl LoginInfo {
    pub fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new(); // FIXME: change this hasher to something else, this will not work between releases
        self.user_name.hash(&mut hasher);
        self.key.hash(&mut hasher);
        hasher.finish()
    }
}

impl Default for LoginInfo {
    fn default() -> Self {
        Self {
            user_name: "".to_string(),
            key: "".to_string(),
        }
    }
}

impl Display for GameDataPacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.score)
    }
}
