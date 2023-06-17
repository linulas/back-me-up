use crate::models::app;
use crate::models::backup::Backup;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, SendError, Sender};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread;

pub mod backup;

pub type Id = String;
pub type WorkerId = usize;
pub type Channel = (Sender<ThreadAction>, Receiver<ThreadAction>);
pub type Active = HashMap<Id, WorkerId>;

#[derive(Debug, Serialize)]
pub enum Error {
    App(app::Error),
    NotFound(String),
    Send(String),
    Terminate(String),
}

impl From<PoisonError<MutexGuard<'_, HashMap<String, usize>>>> for Error {
    fn from(e: PoisonError<MutexGuard<HashMap<String, usize>>>) -> Self {
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

        Self { workers, sender, receiver }
    }
    pub fn execute<F>(&mut self, f: F) -> Result<(), Error>
    where
        F: FnOnce(Arguments) + Send + 'static,
    {
        let job = Box::new(f);

        if !self.has_available_worker() {
           self.add_worker(); 
        }

        Ok(self.sender.send(Message::New(job))?)
    }

    fn has_available_worker(&self) -> bool {
        if self.workers.is_empty() {
            return false;
        };

        for worker in &self.workers {
            if worker.thread.is_none() {
                return true;
            }
        }

        false
    }

    fn add_worker(&mut self) {
        let id = self.workers.len() + 1;
        println!("Adding new worker width id {id}");

        self.workers.push(Worker::new(id, Arc::clone(&self.receiver)));
    }

    pub fn terminate_worker(
        &mut self,
        id: WorkerId,
        event_trigger: impl Fn(),
    ) -> Result<(), String> {
        let worker = match self.workers.iter_mut().find(|w| w.id == id) {
            Some(worker) => worker,
            None => return Err("Could not find worker".to_string()),
        };

        match worker.sender.lock() {
            Ok(sender) => {
                if let Err(error) = sender.send(ThreadAction::Terminate) {
                    return Err(error.to_string());
                }
            }
            Err(e) => return Err(format!("Could not send terminate message: \n{e:?}")),
        }

        let thread = if let Some(thread) = worker.thread.take() {
            thread
        } else {
            println!("Worker {} already terminated.", worker.id);
            return Ok(());
        };

        // INFO: trigger event on the running job so that it can be closed
        event_trigger();

        println!(
            "Shutting down worker {} (isFinised: {})",
            worker.id,
            thread.is_finished()
        );
        match thread.join() {
            Ok(_) => {
                println!("Worker {} terminated.", worker.id);
                Ok(())
            }
            Err(e) => {
                println!("Error shutting down worker {}: {e:?}", worker.id);
                Err("Error shutting down worker".to_string())
            }
        }
    }
}

impl Drop for Pool {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers.");

        for worker in &mut self.workers {
            if let Err(e) = self.sender.send(Message::Terminate(worker.id)) {
                println!("Error sending terminate message: {e:?}");
            }
        }

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            match worker.thread.take() {
                Some(thread) => match thread.join() {
                    Ok(_) => {
                        println!("Worker {} terminated.", worker.id);
                    }
                    Err(e) => {
                        println!("Error shutting down worker {}: {e:?}", worker.id);
                    }
                },
                None => {
                    println!("Worker {} already terminated.", worker.id);
                }
            }
        }
    }
}

pub struct Worker {
    pub id: WorkerId,
    pub thread: Option<thread::JoinHandle<()>>,
    sender: Arc<Mutex<Sender<ThreadAction>>>,
}

impl Worker {
    pub fn new(id: WorkerId, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Self {
        let (sender, worker_receiver): Channel = mpsc::channel();
        let worker_receiver = Arc::new(Mutex::new(worker_receiver));
        let sender = Arc::new(Mutex::new(sender));
        let receiver_for_thread = Arc::clone(&worker_receiver);
        let sender_for_thread = Arc::clone(&sender);
        let arguments = Arguments {
            id,
            sender: sender_for_thread,
            receiver: receiver_for_thread,
        };

        sender
            .lock()
            .expect("could not lock sender")
            .send(ThreadAction::Start)
            .expect("could not send start message");

        let thread = thread::spawn(move || loop {
            let message = receiver
                .lock()
                .expect("could not lock receiver")
                .recv()
                .expect("could not receive message");

            match message {
                Message::New(job) => {
                    println!("Worker {id} got a job; executing.");

                    job.call_box(arguments);
                    println!("Worker {id} finished.");
                    break;
                }
                Message::Terminate(termination_id) => {
                    if termination_id == id {
                        println!("Worker {id} was told to terminate.");
                        break;
                    }
                }
            }
        });
        Self {
            id,
            thread: Some(thread),
            sender,
        }
    }
}

pub fn id_from_backup(backup: &Backup) -> String {
    format!(
        "{}_{}",
        backup.client_folder.path, backup.server_folder.path
    )
}
