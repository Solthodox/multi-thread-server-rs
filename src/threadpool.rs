use std::{
    sync::{
        mpsc::{self, Receiver},
        Arc, Mutex,
    },
    thread,
};

#[derive(Debug)]
pub enum ThreadPoolError {
    CreationError(String),
    ExecutionError(String),
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    pub fn build(size: usize, stack_size_bytes: usize) -> Result<ThreadPool, ThreadPoolError> {
        let spawn_workers = |size: usize, receiver: Arc<Mutex<Receiver<Job>>>| {
            // usaremos el builder de thread ya que nos devuelve un error si falla al crear un thread
            let mut workers: Vec<Worker> = Vec::with_capacity(size);
            for id in 0..size {
                let builder = thread::Builder::new();
                let builder = builder.stack_size(stack_size_bytes);
                let receiver_clone = Arc::clone(&receiver); // Clone the Arc to use inside the closure
                let thread = builder
                    .spawn(move || loop {
                        let job = receiver_clone.lock().unwrap().recv().unwrap();
                        job();
                    })
                    .unwrap();

                workers.push(Worker { id, thread });
            }
            workers
        };

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        if size > 0 && size <= 10000 {
            Ok(ThreadPool {
                workers: spawn_workers(size, Arc::clone(&receiver)),
                sender,
            })
        } else if size > 10000 {
            Err(ThreadPoolError::CreationError(String::from(
                "Thread pool size is greater than 10,000",
            )))
        } else {
            Err(ThreadPoolError::CreationError(String::from(
                "Thread pool size is zero",
            )))
        }
    }

    pub fn execute<F>(&self, f: F) -> Result<(), ThreadPoolError>
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        match self.sender.send(job) {
            Ok(_) => return Ok(()),
            Err(_) => {
                return Err(ThreadPoolError::ExecutionError(String::from(
                    "Failed to execute task",
                )))
            }
        }
    }
}
