use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    thread,
};

/// Type alias for a task function
type Task = Box<dyn FnOnce() + Send + 'static>;

/// Messages sent to worker threads
enum Message {
    Task(Task),
    Terminate,
}

/// A simple thread pool for executing tasks concurrently
pub struct ThreadPool {
    threads: Vec<std::thread::JoinHandle<()>>,
    manager: mpsc::Sender<Message>,
    active_tasks: Arc<AtomicUsize>,
}

impl ThreadPool {
    /// Create a new thread pool with the given number of threads
    pub fn new(size: usize) -> Self {
        let (sender, receiver) = mpsc::channel::<Message>();
        let receiver = Arc::new(Mutex::new(receiver));
        let active_tasks = Arc::new(AtomicUsize::new(0));

        let mut threads = Vec::with_capacity(size);

        for _ in 0..size {
            let receiver = Arc::clone(&receiver);
            let active_tasks = Arc::clone(&active_tasks);

            let thread = thread::spawn(move || {
                while let Ok(message) = receiver.lock().unwrap().recv() {
                    if let Message::Task(task) = message {
                        task();
                        active_tasks.fetch_sub(1, Ordering::SeqCst);
                    } else {
                        break;
                    }
                }
            });

            threads.push(thread);
        }

        ThreadPool {
            threads,
            manager: sender,
            active_tasks,
        }
    }

    /// Execute a task in the thread pool
    #[inline]
    pub fn submit<F>(&self, task: Box<F>)
    where
        F: FnOnce() + Send + 'static,
    {
        let message = Message::Task(task);
        self.active_tasks.fetch_add(1, Ordering::SeqCst);
        self.manager.send(message).unwrap();
    }

    /// Terminate all threads and wait for them to finish
    #[inline]
    pub fn terminate(&mut self) {
        for _ in &self.threads {
            self.manager.send(Message::Terminate).unwrap();
        }

        for thread in self.threads.drain(..) {
            thread.join().unwrap();
        }
    }

    /// Wait for all active tasks to complete
    #[inline]
    pub fn wait_all(&self) {
        while self.active_tasks.load(Ordering::SeqCst) > 0 {
            thread::yield_now();
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.terminate();
    }
}
