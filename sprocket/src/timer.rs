use std::time::{Duration, Instant};

/// A general purpose timer for measuring execution time or checking if a certain time has passed
pub struct Timer {
    start: Instant,
    pub target: Option<Duration>,
    // The instant of when and if stopped
    stopped: Option<Duration>,
}

impl Timer {
    /// Starts a new timer
    pub fn start() -> Self {
        Timer {
            start: Instant::now(),
            target: None,
            stopped: None,
        }
    }

    /// Creates a new timer which is going to be signaled after specified duration
    pub fn with_target(duration: Duration) -> Self {
        Timer {
            start: Instant::now(),
            target: Some(duration),
            stopped: None,
        }
    }

    /// Returns the current duration of the timer
    /// If called on a stopped timer, it return that when it was stopped
    pub fn duration(&self) -> Duration {
        match self.stopped {
            None => Instant::now().saturating_duration_since(self.start),
            Some(duration) => duration,
        }
    }

    /// Stops the timer and returns the duration
    pub fn stop(&mut self) -> Duration {
        self.stopped = Some(Instant::now().saturating_duration_since(self.start));
        self.stopped.unwrap()
    }

    /// Restarts a stopped timer
    /// Will be signaled again after passing target duration
    /// Does nothing on a running timer
    pub fn restart(&mut self) {
        self.start = Instant::now();
        self.stopped = None
    }

    pub fn running(&self) -> bool {
        self.stopped.is_none()
    }

    pub fn stopped(&self) -> bool {
        self.stopped.is_some()
    }

    /// Returns true if the timer has passed the target if set
    pub fn signaled(&self) -> bool {
        match self.target {
            Some(target) => self.duration() >= target,
            None => false,
        }
    }

    /// Returns the time to the target if any
    /// Returns None if target is None, target has passed, or timer is stopped
    pub fn remaining(&self) -> Option<Duration> {
        if self.stopped.is_some() {
            return None;
        }

        match self.target {
            Some(target) => {
                target.checked_sub(Instant::now().saturating_duration_since(self.start))
            }
            None => None,
        }
    }

    /// Returns the time in whole milliseconds to the target if any
    /// Returns None if target is None, target has passed, or timer is stopped
    pub fn remaining_ms(&self) -> Option<usize> {
        self.remaining().map(|d| d.as_millis() as usize)
    }

    /// Returns the time in whole microseconds to the target if any
    /// Returns None if target is None, target has passed, or timer is stopped
    pub fn remaining_us(&self) -> Option<usize> {
        self.remaining().map(|d| d.as_micros() as usize)
    }

    /// Returns the time in fractional seconds to the target if any
    /// Returns None if target is None, target has passed, or timer is stopped
    pub fn remaining_f32(&self) -> Option<f32> {
        self.remaining().map(|d| d.as_secs_f32())
    }
}
