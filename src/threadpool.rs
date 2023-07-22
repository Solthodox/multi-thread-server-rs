use std::{
    sync::{
        mpsc::{self, Receiver},
        Arc, Mutex,
    },
    thread::{self},
};

/// Error types that may occur while creating or executing tasks in the thread pool.
#[derive(Debug, PartialEq, Eq)]
pub enum ThreadPoolError {
    /// Error indicating a problem occurred during the creation of the thread pool.
    CreationError(String),
    /// Error indicating a problem occurred during the execution of a task in the thread pool.
    ExecutionError(String),
}

/// A simple thread pool implementation.
#[derive(Debug)]
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

/// A worker thread in the thread pool.
#[derive(Debug)]
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

/// Type alias for a closure-based job that can be executed in the thread pool.
type Job = Box<dyn FnOnce() + Send + 'static>;

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in &mut self.workers {
            println!("Worker {} desconnected; shutting down.", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

impl ThreadPool {
    /// Constructs a new thread pool with the given number of worker threads and stack size in bytes.
    ///
    /// # Arguments
    ///
    /// * `size`: The number of worker threads in the thread pool.
    /// * `stack_size_bytes`: The size of the stack for each worker thread in bytes.
    ///
    /// # Returns
    ///
    /// A `Result` containing the created thread pool if successful, or an error of type `ThreadPoolError`.
    ///
    /// # Panics
    ///
    /// This function may panic if the `thread::Builder` fails to create a new thread.
    pub fn build(size: usize, stack_size_bytes: usize) -> Result<ThreadPool, ThreadPoolError> {
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        if size > 0 && size <= 10000 {
            Ok(ThreadPool {
                workers: Self::spawn_workers(size, Arc::clone(&receiver), stack_size_bytes),
                sender: Some(sender),
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

    /// Executes the given closure-based job in one of the worker threads of the thread pool.
    ///
    /// # Arguments
    ///
    /// * `f`: The closure-based job to be executed in the thread pool.
    ///
    /// # Returns
    ///
    /// A `Result` indicating whether the job was successfully executed or an error of type `ThreadPoolError`.
    pub fn execute<F>(&self, f: F) -> Result<(), ThreadPoolError>
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        match self.sender.as_ref().unwrap().send(job) {
            Ok(_) => return Ok(()),
            Err(_) => {
                return Err(ThreadPoolError::ExecutionError(String::from(
                    "Failed to execute task",
                )))
            }
        }
    }

    // Private method to spawn worker threads in the thread pool.
    fn spawn_workers(
        size: usize,
        receiver: Arc<Mutex<Receiver<Job>>>,
        stack_size_bytes: usize,
    ) -> Vec<Worker> {
        let mut workers: Vec<Worker> = Vec::with_capacity(size);
        for id in 0..size {
            let builder = thread::Builder::new();
            let builder = builder.stack_size(stack_size_bytes);
            let receiver_clone = Arc::clone(&receiver);
            let thread = builder
                .spawn(move || loop {
                    match receiver_clone.lock().unwrap().recv() {
                        Ok(job) => {
                            println!("Worker {id} got a job; executing.");
                            job();
                        }
                        Err(_) => {
                            println!("Worker {id} shutting down");
                            break;
                        }
                    }
                })
                .unwrap();

            workers.push(Worker {
                id,
                thread: Some(thread),
            });
        }
        workers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    #[test]
    fn test_thread_pool_creation_error_zero_size() {
        // Trying to create a thread pool with zero worker threads should return an error.
        let result = ThreadPool::build(0, 1024 * 1024);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ThreadPoolError::CreationError(String::from("Thread pool size is zero"))
        );
    }

    #[test]
    fn test_thread_pool_creation_error_large_size() {
        // Trying to create a thread pool with more than 10,000 worker threads should return an error.
        let result = ThreadPool::build(20000, 1024 * 1024);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ThreadPoolError::CreationError(String::from("Thread pool size is greater than 10,000"))
        );
    }
}
