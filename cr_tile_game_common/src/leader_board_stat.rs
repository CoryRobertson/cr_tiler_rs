use serde::{Deserialize, Serialize};

#[derive(Serialize,Deserialize,Clone,Debug)]
pub struct LeaderBoardEntry {
    username: String,
    discriminator: String,
    score: i32,
}

impl LeaderBoardEntry {
    pub fn new(username: String, score: i32, discriminator: String) -> Self {
        Self { username, discriminator, score }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderBoardList {
    list: Vec<LeaderBoardEntry>,
}

impl LeaderBoardList {
    pub fn new(list: Vec<LeaderBoardEntry>) -> Self {
        Self { list }
    }
}