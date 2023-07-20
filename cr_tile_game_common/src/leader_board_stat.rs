use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LeaderBoardEntry {
    username: String,
    discriminator: String,
    score: i32,
}

impl Display for LeaderBoardEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}#{}: {}",
            self.username, self.discriminator, self.score
        )
    }
}

impl LeaderBoardEntry {
    pub fn new(username: String, score: i32, discriminator: String) -> Self {
        Self {
            username,
            discriminator,
            score,
        }
    }
    pub fn get_score(&self) -> i32 {
        self.score
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
    pub fn get_list(&self) -> &Vec<LeaderBoardEntry> {
        &self.list
    }

    pub fn sort_list(&mut self) {
        self.list
            .sort_by(|item1, item2| item2.score.cmp(&item1.score));
    }
}
