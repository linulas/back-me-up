use ts_rs::TS;

use super::folder::Folder;

#[derive(TS)]
#[ts(export)]
pub struct Backup {
    pub client_folder: Folder,
    pub server_folder: Folder,
    pub latest_run: Option<u64>,
}
