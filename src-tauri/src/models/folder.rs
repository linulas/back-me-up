use std::fmt::Display;

use ts_rs::TS;

#[derive(TS)]
#[ts(export)]
pub enum Size {
    KB(u64),
}

impl Display for Size {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::KB(size) => write!(f, "{size} KB"),
        }
    }
}

#[derive(TS)]
#[ts(export)]
pub struct Folder {
    pub name: String,
    pub path: String,
    pub size: Option<Size>,
}
