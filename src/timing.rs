use std::time::{Duration, Instant};
use std::fmt;

/// A simple struct to measure and report execution time
pub struct Timer {
    start: Instant,
    name: String,
}

impl Timer {
    /// Create a new timer with the given name
    pub fn new(name: &str) -> Self {
        Timer {
            start: Instant::now(),
            name: name.to_string(),
        }
    }

    /// Reset the timer
    pub fn reset(&mut self) {
        self.start = Instant::now();
    }

    /// Get the elapsed time since the timer was created or reset
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Get the elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> f64 {
        let duration = self.elapsed();
        duration.as_secs_f64() * 1000.0
    }

    /// Get the elapsed time in microseconds
    pub fn elapsed_us(&self) -> f64 {
        let duration = self.elapsed();
        duration.as_secs_f64() * 1_000_000.0
    }

    /// Get the elapsed time in nanoseconds
    pub fn elapsed_ns(&self) -> f64 {
        let duration = self.elapsed();
        duration.as_secs_f64() * 1_000_000_000.0
    }

    /// Print the elapsed time with the timer name
    pub fn report(&self) {
        println!("{}", self);
    }
}

impl fmt::Display for Timer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let elapsed = self.elapsed();
        
        if elapsed.as_secs() > 0 {
            write!(f, "{}: {:.2} s", self.name, elapsed.as_secs_f64())
        } else if elapsed.as_millis() > 0 {
            write!(f, "{}: {:.2} ms", self.name, self.elapsed_ms())
        } else if elapsed.as_micros() > 0 {
            write!(f, "{}: {:.2} µs", self.name, self.elapsed_us())
        } else {
            write!(f, "{}: {:.2} ns", self.name, self.elapsed_ns())
        }
    }
}

/// A macro to time a block of code and print the result
#[macro_export]
macro_rules! time_it {
    ($name:expr, $block:block) => {{
        let mut timer = $crate::timing::Timer::new($name);
        let result = $block;
        timer.report();
        result
    }};
}

/// A macro to time a function call and print the result
#[macro_export]
macro_rules! time_call {
    ($name:expr, $func:ident, $($arg:expr),*) => {{
        let mut timer = $crate::timing::Timer::new($name);
        let result = $func($($arg),*);
        timer.report();
        result
    }};
}
