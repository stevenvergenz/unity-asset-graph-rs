use std::{
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
};

const PRINT_INTERVAL: Duration = Duration::from_millis(500);

pub struct ProgressIndicator {
    current: Arc<Mutex<usize>>,
    done: Arc<Mutex<bool>>,
    handle: JoinHandle<()>,
}

impl ProgressIndicator {
    pub fn new(op: &str, total: Option<usize>) -> Self {
        let op = op.to_string();
        let current = Arc::new(Mutex::new(0));
        let thread_current = Arc::clone(&current);

        let done = Arc::new(Mutex::new(false));
        let thread_done = Arc::clone(&done);

        let handle = thread::spawn(move || {
            loop {
                thread::sleep(PRINT_INTERVAL);
                
                if let Ok(current) = thread_current.lock() {
                    Self::print_progress(&op, *current, total);
                }

                if let Ok(done) = thread_done.lock() && *done {
                    break;
                }
            }
        });

        Self {
            current,
            done,
            handle,
        }
    }

    pub fn advance(&mut self) {
        let mut current = match self.current.lock() {
            Ok(c) => c,
            Err(_) => return, // Handle lock poisoning gracefully
        };
        *current += 1;
    }

    pub fn finish(self, message: &str) {
        // Wait for the progress thread to finish
        let mut done = match self.done.lock() {
            Ok(d) => d,
            Err(_) => panic!("Failed to lock done mutex"), // Handle lock poisoning gracefully
        };
        *done = true;
        drop(done); // Release the lock before joining the thread

        self.handle.join().expect("Progress thread panicked");
        println!("\n{message}");
    }

    fn print_progress(op: &str, current: usize, total: Option<usize>) {
        if let Some(total) = total {
            let percent = (current as f64 / total as f64) * 100.0;
            print!("\r{}: {:.2}% ({} / {})", op, percent, current, total);
        }
        else {
            print!("\r{}: {}", op, current);
        }
    }
}