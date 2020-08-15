use std::time::{Duration, Instant};

/// Contains time information of a certain part of the program
/// Tracks elapsed time, delta time, frame count
pub struct Time {
    /// The time on the last update
    cur: Instant,
    /// The time of the previous update
    prev: Instant,
    init: Instant,
    framecount: usize,
    delta: Duration,
    elapsed: Duration,
}

impl Time {
    /// Creates and initializes a new time struct
    /// Few time instances should be need for a program, usually one per thread
    pub fn new() -> Self {
        Time {
            cur: Instant::now(),
            prev: Instant::now(),
            init: Instant::now(),
            framecount: 0,
            delta: Duration::from_secs(0),
            elapsed: Duration::from_secs(0),
        }
    }

    /// Updates and advances the time to the next frame and caluclates deltatime
    /// This function will not panic even if current time is less than previous time
    /// If current time is less than previous time, deltatime will be 0 for that frame
    pub fn update(&mut self) {
        self.prev = self.cur;
        self.cur = Instant::now();

        self.delta = self.cur.saturating_duration_since(self.prev);

        self.elapsed = self.cur.saturating_duration_since(self.init);
        self.framecount += 1;
    }

    /// Returns the duration between the last frame and start of current frame in seconds
    pub fn delta_f32(&self) -> f32 {
        self.delta.as_secs_f32()
    }

    /// Returns the raw duration between the last frame and start of current frame
    pub fn delta_raw(&self) -> Duration {
        self.delta
    }

    /// Returns the duration between the last frame and start of current frame in whole milliseconds
    /// Can be used for precise timing and benchmarking
    /// A whole smaller time unit does not lose precision to rounding errors like floats
    pub fn delta_ms(&self) -> usize {
        self.delta.as_millis() as usize
    }

    /// Returns the duration between the last frame and start of current frame in whole microseconds
    /// Can be used for precise timing and benchmarking
    /// A whole smaller time unit does not lose precision to rounding errors like floats
    pub fn delta_us(&self) -> usize {
        self.delta.as_micros() as usize
    }

    /// Returns the elapsed time since creation of self and the start of the current frame in seconds
    pub fn elapsed_f32(&self) -> f32 {
        self.elapsed.as_secs_f32()
    }

    /// Returns the elapsed time since creation of self and the start of the current frame
    pub fn elapsed_raw(&self) -> Duration {
        self.elapsed
    }

    /// Returns the number of frames advanced with update
    /// Note, when running on different threads, if reflect the thread local frame, not graphical frame
    /// For graphical frame, use the time object of the renderer TODO
    pub fn framecount(&self) -> usize {
        self.framecount
    }

    /// Returns the framerate between this and the previous frame
    pub fn framerate(&self) -> f32 {
        1.0 / self.delta_f32()
    }
}
