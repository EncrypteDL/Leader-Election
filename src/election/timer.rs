use logging::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// Timer internal thread: check interval
const THREAD_CHECK_INTERVAL: Duration = Duration::from_millis(10);

#[derive(Debug)]
pub struct Timer {
    // Timer Name
    name: String,
    // Controls whether the timer is executed
    alive: Arc<AtomicBool>,
    // Timer trigger interval
    trigger_interval: Arc<Mutex<Duration>>,
    // The next time in the timer triggers
    next_trigger: Arc<Mutex<Instant>>,
    // The time when the timer was last reset
    pub last_reset_at: Option<Instant>,
    // Timer internal thread
    handle: Option<std::thread::JoinHandle<()>>,
}

impl Timer {
    pub fn new(name: &str) -> Self {
        Timer {
            name: name.to_string(),
            alive: Arc::new(AtomicBool::new(false)),
            trigger_interval: Arc::new(Mutex::new(Duration::from_secs(std::u64::MAX))),
            next_trigger: Arc::new(Mutex::new(Instant::now())),
            last_reset_at: None,
            handle: None,
        }
    }

    // Start Timer
    pub fn schedule<F>(&mut self, trigger_interval: Duration, callback: F)
    where
        F: 'static + Send + Clone + FnMut() -> (),
    {
        info!(
            "{} start schedule with trigger interval: {}ms",
            self.name,
            trigger_interval.as_millis()
        );

        (*self.trigger_interval.lock().unwrap()) = trigger_interval;
        (*self.next_trigger.lock().unwrap()) = Instant::now() + trigger_interval;
        self.alive.store(true, Ordering::SeqCst);

        let trigger_interval = self.trigger_interval.clone();
        let next_trigger = self.next_trigger.clone();
        let alive = self.alive.clone();

        self.handle = Some(std::thread::spawn(move || {
            loop {
                std::thread::sleep(THREAD_CHECK_INTERVAL);

                if !alive.load(Ordering::SeqCst) {
                    break;
                }

                if (*next_trigger.lock().unwrap()) <= Instant::now() {
                    // Execute callback function asynchronously without blocking the timer thread
                    let mut callback = callback.clone();
                    std::thread::spawn(move || {
                        callback();
                    });

                    // Recalculate the next trigger time
                    (*next_trigger.lock().unwrap()) =
                        Instant::now() + (*trigger_interval.lock().unwrap());
                }
            }
        }));
    }

    // Reset the timer firing interval
    pub fn reset(&mut self, trigger_interval: Duration) {
        info!(
            "{} reset with trigger interval: {}ms",
            self.name,
            trigger_interval.as_millis()
        );
        self.last_reset_at = Some(Instant::now());
        (*self.trigger_interval.lock().unwrap()) = trigger_interval;
        (*self.next_trigger.lock().unwrap()) = Instant::now() + trigger_interval;
    }

    // Stop Timer
    pub fn stop(&mut self) {
        info!("{} stopping", self.name);
        self.alive.store(false, Ordering::SeqCst);
        if let Some(handle) = self.handle.take() {
            handle.join().unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_timer() {
        let mut timer = super::Timer::new("test_timer");
        timer.schedule(std::time::Duration::from_secs(1), || {
            println!("hello {:?}", std::time::Instant::now());
        });
        std::thread::sleep(std::time::Duration::from_secs(10));

        timer.reset(std::time::Duration::from_secs(2));

        std::thread::sleep(std::time::Duration::from_secs(10));
    }
}
