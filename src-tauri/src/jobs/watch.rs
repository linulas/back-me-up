use std::path::Path;

use chrono::{DateTime, Local};
use notify::{Event, RecursiveMode, RecommendedWatcher, Watcher};

use super::Job;

pub type Callback = dyn Fn(&Event, &Job, &mut DateTime<Local>) -> Result<(), notify::Error>;
pub type HandleCallbackError = dyn Fn(notify::Error, Event, &Job);

pub fn job_recursive(job: Job) -> notify::Result<()> {
    let path = Path::new(&job.backup.client_folder.path);
    let (sender, reciever) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(sender, notify::Config::default())?;
    let mut last_modified: DateTime<Local> = Path::new(&job.backup.client_folder.path)
        .metadata()?
        .modified()?
        .into();

    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    for res in reciever {
        match res {
            Ok(event) => {
                if let Err(e) = (job.callback)(&event, &job, &mut last_modified) {
                    (job.handle_callback_error)(e, event, &job);
                }
            }
            Err(e) => println!("watch error: {e:?}"),
        }
    }

    Ok(())
}
