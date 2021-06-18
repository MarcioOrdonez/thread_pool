use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;
use std::sync::Mutex;

pub struct ThreadPool {
    _handles: Vec<std::thread::JoinHandle<()>>,
    sender: Sender<Box<dyn FnMut() + Send>>,
}

impl ThreadPool {
    pub fn new(num_threads: u8) -> Self {
        // Adding the + Send makes the dyn clousere thread safe so we can send it between threads
        let (sender, receiver) = channel::<Box<dyn FnMut() + Send>>();
        // receiver needs to be an ARC so it can have multiple owners and will be destroyed after all life times
        let receiver = Arc::new(Mutex::new(receiver));
        let mut _handles = vec![];
        for _ in 0..num_threads {
            let receiver_clone = receiver.clone();
            let handle = std::thread::spawn(move || loop {
                // we have ownership of the receiver because of the `move` and it will
                // destroy the receiver when the spawn ends so in the next thread
                // receiver no longer existis, so we need to use `ARC` to solve this
                let mut work = match receiver_clone.lock().unwrap().recv() {
                    Ok(work) => work,
                    Err(_) => break,
                };
                work();
            });
            _handles.push(handle);
        }
        Self { _handles, sender }
    }
    pub fn execute<T: FnMut() + Send + 'static>(&self, work: T) {
        self.sender.send(Box::new(work)).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        use std::sync::atomic::{AtomicU32, Ordering};
        let n = AtomicU32::new(0);
        let nref = Arc::new(n);
        let pool = ThreadPool::new(10);
        let clone = nref.clone();
        let foo = move || {
            clone.fetch_add(1, Ordering::SeqCst);
        };
        pool.execute(foo.clone());
        pool.execute(foo.clone());
        std::thread::sleep(std::time::Duration::from_secs(2));
        assert_eq!(nref.load(Ordering::SeqCst), 2);
    }
}
