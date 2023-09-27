use crate::models::app::{self, Config};
use crate::models::backup::Backup;
use crate::ssh;
use log::{error, info, warn};
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, SendError, Sender};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread;
use ts_rs::TS;

const IS_AVAILABLE_SHOULD_LOCK: &str = "could not lock field 'is_available'";

pub mod backup;
pub mod fs;
pub mod maintenance;

pub type Id = String;
pub type WorkerId = usize;
pub type Channel = (Sender<ThreadAction>, Receiver<ThreadAction>);
pub type Active = HashMap<Id, WorkerId>;
pub type Failed = HashMap<Id, WorkerId>;

#[derive(Debug, Serialize)]
pub enum Error {
    App(app::Error),
    Ssh(ssh::Error),
    NotFound(String),
    Send(String),
    Terminate(String),
    Pattern(String),
    Command(String),
    Failed(String),
}

impl From<PoisonError<std::sync::MutexGuard<'_, PathBuf>>> for Error {
    fn from(e: PoisonError<std::sync::MutexGuard<PathBuf>>) -> Self {
        Self::App(app::Error::from(e))
    }
}

impl From<ssh::Error> for Error {
    fn from(e: ssh::Error) -> Self {
        Self::Ssh(e)
    }
}

impl From<PoisonError<MutexGuard<'_, HashMap<String, usize>>>> for Error {
    fn from(e: PoisonError<MutexGuard<HashMap<String, usize>>>) -> Self {
        Self::App(app::Error::from(e))
    }
}

impl From<PoisonError<MutexGuard<'_, Option<Config>>>> for Error {
    fn from(e: PoisonError<MutexGuard<Option<Config>>>) -> Self {
        Self::App(app::Error::from(e))
    }
}

impl From<PoisonError<MutexGuard<'_, Pool>>> for Error {
    fn from(e: PoisonError<MutexGuard<Pool>>) -> Self {
        Self::App(app::Error::from(e))
    }
}

impl<T> From<SendError<T>> for Error {
    fn from(e: SendError<T>) -> Self {
        Self::Send(e.to_string())
    }
}

#[derive(Default)]
pub struct Handle {
    pub id: Id,
    pub value: Option<thread::JoinHandle<()>>,
}

pub enum Kind {
    BackupOnChange,
    Backup,
}

#[derive(TS, Serialize)]
#[ts(
    export,
    export_to = "../../bindings/JobStatus.ts",
    rename = "JobStatus"
)]
pub enum Status {
    Running,
    Failed,
    Completed,
}

pub enum ThreadAction {
    Start,
    Continue,
    Terminate,
}

pub enum Message {
    New(Job),
    Terminate(WorkerId),
}

pub struct Pool {
    workers: Vec<Worker>,
    sender: Sender<Message>,
    receiver: Arc<Mutex<Receiver<Message>>>,
}

pub trait FnBox {
    fn call_box(self: Box<Self>, arguments: Arguments);
}

impl<F: FnOnce(Arguments)> FnBox for F {
    fn call_box(self: Box<F>, arguments: Arguments) {
        (*self)(arguments);
    }
}

#[derive(Clone)]
pub struct Arguments {
    pub id: WorkerId,
    pub sender: Arc<Mutex<Sender<ThreadAction>>>,
    pub receiver: Arc<Mutex<mpsc::Receiver<ThreadAction>>>,
}

type Job = Box<dyn FnBox + Send + 'static>;

impl Pool {
    pub fn new(size: Option<usize>) -> Self {
        let mut workers = size.map_or_else(Vec::new, Vec::with_capacity);
        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        if let Some(size) = size {
            for id in 0..size {
                workers.push(Worker::new(id, Arc::clone(&receiver)));
            }
        }

        Self {
            workers,
            sender,
            receiver,
        }
    }

    pub fn execute<F>(&mut self, f: F) -> Result<(), Error>
    where
        F: FnOnce(Arguments) + Send + 'static,
    {
        let job = Box::new(f);

        if !self.has_available_worker() {
            self.create_workers(1);
        }

        Ok(self.sender.send(Message::New(job))?)
    }

    #[must_use]
    pub fn available_workers(&self) -> usize {
        self.workers.iter().fold(0, |acc, w| {
            if *w.is_available.lock().expect(IS_AVAILABLE_SHOULD_LOCK) {
                acc + 1
            } else {
                acc
            }
        })
    }

    fn has_available_worker(&mut self) -> bool {
        if self.workers.is_empty() {
            return false;
        };

        for worker in &mut self.workers {
            if *worker.is_available.lock().expect(IS_AVAILABLE_SHOULD_LOCK) {
                if worker.thread.is_none() {
                    *worker.is_available.lock().expect(IS_AVAILABLE_SHOULD_LOCK) = false;
                    worker.start();
                }

                return true;
            }
        }

        false
    }

    pub fn create_workers(&mut self, size: usize) {
        if size == 0 {
            warn!("size must be greater than 0");
            return;
        }

        for _ in 0..size {
            let id = self.workers.len() + 1;
            let mut worker = Worker::new(id, Arc::clone(&self.receiver));
            worker.start();
            info!("Adding new worker with id {id}");
            self.workers.push(worker);
        }
    }

    pub fn start_all_stopped_workers(&mut self) {
        for worker in &mut self.workers {
            if *worker.is_available.lock().expect(IS_AVAILABLE_SHOULD_LOCK)
                && worker.thread.is_none()
            {
                worker.start();
            }
        }
    }

    pub fn stop_all_workers(&mut self) {
        info!("Sending terminate message to all workers.");

        for worker in &mut self.workers {
            if let Err(e) = self.sender.send(Message::Terminate(worker.id)) {
                info!("Error sending terminate message: {e:?}");
            }
        }

        for worker in &mut self.workers {
            match worker.thread.take() {
                Some(thread) => match thread.join() {
                    Ok(_) => {
                        info!("Worker {} terminated.", worker.id);
                    }
                    Err(e) => {
                        info!("Error shutting down worker {}: {e:?}", worker.id);
                    }
                },
                None => {
                    info!("Worker {} already terminated.", worker.id);
                }
            }
        }
    }

    pub fn terminate_job(&mut self, id: WorkerId, event_trigger: impl Fn()) -> Result<(), String> {
        let worker = match self.workers.iter_mut().find(|w| w.id == id) {
            Some(worker) => worker,
            None => return Err("Could not find worker".to_string()),
        };

        match worker.local_sender.lock() {
            Ok(sender) => {
                if let Err(error) = sender.send(ThreadAction::Terminate) {
                    return Err(error.to_string());
                }
            }
            Err(e) => return Err(format!("Could not send terminate message: \n{e:?}")),
        }

        // INFO: trigger event on the running job so that it can be closed
        event_trigger();

        Ok(())
    }
}

pub struct Worker {
    pub id: WorkerId,
    is_available: Arc<Mutex<bool>>,
    pub thread: Option<thread::JoinHandle<()>>,
    local_sender: Arc<Mutex<Sender<ThreadAction>>>,
    local_receiver: Arc<Mutex<Receiver<ThreadAction>>>,
    receiver: Arc<Mutex<mpsc::Receiver<Message>>>,
}

impl Worker {
    pub fn new(id: WorkerId, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Self {
        let (local_sender, local_receiver): Channel = mpsc::channel();
        let local_sender = Arc::new(Mutex::new(local_sender));
        let local_receiver = Arc::new(Mutex::new(local_receiver));
        let is_available = Arc::new(Mutex::new(true));

        Self {
            id,
            is_available,
            thread: None,
            local_sender,
            local_receiver,
            receiver,
        }
    }

    pub fn start(&mut self) {
        info!("Starting worker {}", self.id);
        let local_sender = Arc::clone(&self.local_sender);
        let local_receiver = Arc::clone(&self.local_receiver);

        let arguments = Arguments {
            id: self.id,
            receiver: local_receiver,
            sender: local_sender,
        };
        let is_available = Arc::clone(&self.is_available);

        arguments
            .sender
            .lock()
            .expect("could not lock sender")
            .send(ThreadAction::Start)
            .expect("could not send start message");

        let id = self.id;
        let receiver = Arc::clone(&self.receiver);

        let thread = thread::spawn(move || loop {
            let message = receiver
                .lock()
                .expect("could not lock receiver")
                .recv()
                .expect("could not receive message");

            match message {
                Message::New(job) => {
                    *is_available.lock().expect(IS_AVAILABLE_SHOULD_LOCK) = false;
                    info!("Worker {id} got a job; executing.");

                    job.call_box(arguments.clone());
                    info!("Worker {id} finished.");
                    *is_available.lock().expect(IS_AVAILABLE_SHOULD_LOCK) = true;
                }
                Message::Terminate(termination_id) => {
                    if termination_id == id {
                        info!("Worker {id} was told to terminate.");
                        *is_available.lock().expect(IS_AVAILABLE_SHOULD_LOCK) = true;
                        break;
                    }
                }
            }
        });

        self.thread = Some(thread);
    }
}

#[must_use]
pub fn id_from_backup(backup: &Backup, kind: &Kind) -> String {
    match kind {
        Kind::Backup => {
            format!(
                "{}_{}_backup",
                backup.client_location.path, backup.server_location.path
            )
        }
        Kind::BackupOnChange => {
            format!(
                "{}_{}_backup_on_change",
                backup.client_location.path, backup.server_location.path
            )
        }
    }
}

pub fn check_status(
    id: &String,
    running_jobs: &Arc<Mutex<Active>>,
    failed_jobs: &Arc<Mutex<Failed>>,
) -> Result<Status, Error> {
    if failed_jobs.lock()?.contains_key(id) {
        error!("{id}: failed");
        return Ok(Status::Failed);
    } else if running_jobs.lock()?.contains_key(id) {
        return Ok(Status::Running);
    }

    info!("{id}: completed");
    Ok(Status::Completed)
}
