//! Input simulation helpers built on top of `enigo`, plus the small random /
//! timing utilities the macros rely on (ported from `Helper.cs`).

use enigo::{Button, Coordinate, Direction, Enigo, Key, Keyboard, Mouse, Settings};
use std::cell::Cell;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

thread_local! {
    static RNG_STATE: Cell<u64> = Cell::new(seed());
}

fn seed() -> u64 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0x9E37_79B9_7F4A_7C15);
    // Avoid a zero state for xorshift.
    nanos | 1
}

fn next_u64() -> u64 {
    RNG_STATE.with(|state| {
        let mut x = state.get();
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        state.set(x);
        x
    })
}

/// Returns a random value in `[min, max)`. Mirrors `Helper.GetRandomDelay`.
pub fn random_range(min: i32, max: i32) -> i32 {
    if max <= min {
        return min;
    }
    let span = (max - min) as u64;
    min + (next_u64() % span) as i32
}

pub fn sleep_ms(ms: u64) {
    std::thread::sleep(Duration::from_millis(ms));
}

pub fn sleep_rand(min: i32, max: i32) {
    sleep_ms(random_range(min, max).max(0) as u64);
}

/// Interruptible sleep used by the macro workers. Returns `false` if a stop
/// was requested while sleeping.
pub fn sleep_until_stopped(stop: &std::sync::atomic::AtomicBool, ms: u64) -> bool {
    use std::sync::atomic::Ordering;
    let deadline = std::time::Instant::now() + Duration::from_millis(ms);
    while std::time::Instant::now() < deadline {
        if stop.load(Ordering::Relaxed) {
            return false;
        }
        let remaining = deadline.saturating_duration_since(std::time::Instant::now());
        std::thread::sleep(remaining.min(Duration::from_millis(25)));
    }
    !stop.load(Ordering::Relaxed)
}

/// Wrapper around `enigo` exposing the handful of operations the macros need.
pub struct Input {
    enigo: Enigo,
}

impl Input {
    pub fn new() -> Result<Self, String> {
        let enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
        Ok(Self { enigo })
    }

    pub fn key(&mut self, key: Key, direction: Direction) {
        let _ = self.enigo.key(key, direction);
    }

    pub fn text(&mut self, text: &str) {
        let _ = self.enigo.text(text);
    }

    /// Presses and releases a button with a short hold in between. Games often
    /// ignore a down+up that arrives as a single instant batch, so we hold it
    /// briefly to make the click register.
    pub fn click_button(&mut self, button: Button) {
        let _ = self.enigo.button(button, Direction::Press);
        sleep_ms(25);
        let _ = self.enigo.button(button, Direction::Release);
    }

    pub fn move_to(&mut self, x: i32, y: i32) {
        let _ = self.enigo.move_mouse(x, y, Coordinate::Abs);
    }
}
