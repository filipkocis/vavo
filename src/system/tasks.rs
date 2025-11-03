use std::{
    any::Any,
    sync::{Arc, Mutex, PoisonError, TryLockError},
    thread,
};

type WorkerResult<T> = Arc<Mutex<Option<T>>>;

type Panic = Box<dyn Any + Send + 'static>;

/// A worker that executes a task in a separate thread
struct Worker<T> {
    finished: bool,
    result: WorkerResult<T>,
    panic: Arc<Mutex<Option<Panic>>>,
    handle: Option<thread::JoinHandle<()>>,
}

impl<T> Worker<T> {
    /// Create a new worker
    #[inline]
    fn new(result: WorkerResult<T>, closure: impl FnOnce() + Send + 'static) -> Self {
        let panic = Arc::new(Mutex::new(None));
        let p = Arc::clone(&panic);

        let handle = thread::spawn(move || {
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(closure)) {
                Ok(_) => {}
                Err(e) => {
                    p.lock().unwrap().replace(e);
                }
            };
        });

        Self {
            finished: false,
            result,
            panic,
            handle: Some(handle),
        }
    }

    /// Retrieve the result of the task if its ready
    #[inline]
    fn get_result(&mut self) -> Option<Result<T, Panic>> {
        if self.finished {
            return None;
        }

        let res = self.result.try_lock();

        // Panic if the lock is poisoned
        if let Err(TryLockError::Poisoned(PoisonError { .. })) = res {
            panic!("Task result lock was poisoned");
        };

        // Panic if task panicked
        if let Ok(mut guard) = self.panic.lock()
            && let Some(panic) = guard.take()
        {
            self.handle.take().unwrap().join().unwrap();
            return Some(Err(panic));
        }

        // Early return if task is not finished
        if !self.handle.as_ref().unwrap().is_finished() {
            return None;
        }

        let mut guard = res.ok()?;

        // Return the result
        if let Some(result) = guard.take() {
            self.handle.take().unwrap().join().unwrap();
            return Some(Ok(result));
        }

        // Result not ready
        None
    }
}

/// A synchronous task that runs a function on a separate thread
pub struct Task<T>(Option<Worker<T>>);

/// An asynchronous task that runs a future on a separate thread
pub struct AsyncTask<T>(Option<Worker<T>>);

impl<T: Send + 'static> Task<T> {
    /// Restart with a new task, returns false if the previous task is still running
    pub fn restart<F>(&mut self, task: F) -> bool
    where
        F: FnOnce() -> T + Send + 'static,
    {
        if self.is_running() {
            return false;
        }

        let new_task = Self::execute(task);
        *self = new_task;
        true
    }

    /// Execute a synchronous task in a separate thread
    pub fn execute<F>(task: F) -> Self
    where
        F: FnOnce() -> T + Send + 'static,
    {
        let result = Arc::new(Mutex::new(None));
        let r = Arc::clone(&result);

        let closure = move || {
            let res = task();
            let mut lock = r.lock().unwrap();
            *lock = Some(res);
        };

        let worker = Worker::new(result, closure);

        Task(Some(worker))
    }

    /// Check if the task is finished
    #[inline]
    pub fn is_finished(&self) -> bool {
        self.0.is_none()
    }

    /// Check if the task is still running
    #[inline]
    pub fn is_running(&self) -> bool {
        self.0.is_some()
    }

    /// Check if the task has panicked
    #[inline]
    pub fn is_panic(&self) -> bool {
        if let Some(worker) = &self.0 {
            return worker.panic.lock().unwrap().is_some();
        }
        false
    }

    /// Retrieve the result of a synchronous task, it is non-blocking.
    /// If `T` does not match the actual return type, it panics.
    pub fn retrieve(&mut self) -> Option<Result<T, Panic>> {
        let worker = self.0.as_mut()?;
        let result = worker.get_result();

        if result.is_some() {
            self.0.take();
        }

        result
    }
}

impl<T: Send + 'static> AsyncTask<T> {
    /// Restart with a new task, returns false if the previous task is still running
    pub fn restart<F, Fut>(&mut self, task: F) -> bool
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = T>,
    {
        if self.is_running() {
            return false;
        }

        let new_task = Self::execute_async(task);
        *self = new_task;
        true
    }

    /// Execute an asynchronous task in a separate thread
    pub fn execute_async<F, Fut>(task: F) -> Self
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = T>,
    {
        let result = Arc::new(Mutex::new(None));
        let r = Arc::clone(&result);

        let closure = move || {
            let res = pollster::block_on(task());
            let mut lock = r.lock().unwrap();
            *lock = Some(res);
        };

        let worker = Worker::new(result, closure);

        AsyncTask(Some(worker))
    }

    /// Check if the task is finished
    #[inline]
    pub fn is_finished(&self) -> bool {
        self.0.is_none()
    }

    /// Check if the task is still running
    #[inline]
    pub fn is_running(&self) -> bool {
        self.0.is_some()
    }

    /// Check if the task has panicked
    #[inline]
    pub fn is_panic(&self) -> bool {
        if let Some(worker) = &self.0 {
            return worker.panic.lock().unwrap().is_some();
        }
        false
    }

    /// Retrieve the result of an asynchronous task, it is non-blocking.
    /// If `T` does not match the actual return type, it panics.
    pub fn retrieve(&mut self) -> Option<Result<T, Panic>> {
        let worker = self.0.as_mut()?;
        let result = worker.get_result();

        if result.is_some() {
            self.0.take();
        }

        result
    }
}
