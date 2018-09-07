use std::{
    num::NonZeroUsize,
    sync::{Arc, mpsc, Mutex},
    thread
};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    pub fn new(size: NonZeroUsize) -> ThreadPool {
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let size = size.get();
        let mut workers = Vec::with_capacity(size);
        for id in 1..=size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    /// Delegate some task to the ThreadPool in the form of a closure.
    pub fn execute<F>(&self, f: F)
        where F: FnOnce() + Send + 'static {
        let job = Box::new(f);
        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers...");

        // Tell the workers to shut down, i.e. stop looping.
        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        // Close down each thread, but allow them to finish executing first.
        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                println!("Stopping worker {}...", worker.id);
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                let message = receiver.lock().unwrap().recv().unwrap();
                match message {
                    Message::NewJob(job) => job.call_box(),
                    Message::Terminate => break,
                }
            }
        });

        Worker { id, thread: Some(thread) }
    }
}

enum Message {
    NewJob(Job),
    Terminate,
}

trait FnBox {
    fn call_box(self: Box<Self>);
}

impl<F> FnBox for F
    where F: FnOnce() {
    fn call_box(self: Box<F>) {
        (*self)()
    }
}

type Job = Box<dyn FnBox + Send + 'static>;
