use crate::models::app;
use crate::models::backup::Backup;

pub mod backup;
mod watch;

pub struct Job {
    backup: Backup,
    callback: &'static watch::Callback,
    handle_callback_error: &'static watch::HandleCallbackError,
    config: app::Config,
}
