use std::fmt::Display;

use serde::{Serialize, Deserialize};
use ts_rs::TS;

#[derive(TS, Deserialize, Clone)]
#[ts(export)]
pub enum Entity {
    Folder(Folder),
    File(File),
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export)]
pub enum Size {
    B(u64),
    // KB(u64),
    // MB(u64),
    // GB(u64),
}

// impl Size {
//     pub fn in_mb(&self) -> u64 {
//         match self {
//             Self::B(size) => size / (1024 * 1024),
//             Self::KB(size) => size / 1024,
//             Self::MB(size) => *size,
//             Self::GB(size) => *size * 1024,
//         }
//     }
// }

impl Display for Size {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::B(size) => write!(f, "{size} B"),
            // Self::KB(size) => write!(f, "{size} KB"),
            // Self::MB(size) => write!(f, "{size} MB"),
            // Self::GB(size) => write!(f, "{size} GB"),
        }
    }
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export)]
pub struct Folder {
    pub name: String,
    pub path: String,
    pub size: Option<Size>,
}

impl Display for Folder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{0}", self.name)
    }
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export)]
pub struct File {
    pub name: String,
    pub path: String,
    pub size: Option<Size>,
    pub mime_type: Option<String>,
}
