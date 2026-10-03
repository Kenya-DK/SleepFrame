//! Tails Warframe's `EE.log` while the game is running. Ported from
//! `EELogProcessor.cs`.
//!
//! The original project shipped the dispatcher but registered no events by
//! default, so [`dispatch`] is intentionally a no-op hook kept for parity and
//! future extensions.

use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::PathBuf;
use std::time::Duration;

use crate::win;

pub fn start() {
    std::thread::spawn(|| {
        let mut offset: u64 = 0;
        loop {
            if win::find_warframe_pid().is_some() {
                if let Some(path) = ee_log_path() {
                    tail(&path, &mut offset);
                }
            } else {
                // Game closed: reset so the next session is read from the start.
                offset = 0;
            }
            std::thread::sleep(Duration::from_secs(2));
        }
    });
}

fn ee_log_path() -> Option<PathBuf> {
    let local = std::env::var_os("LOCALAPPDATA")?;
    Some(PathBuf::from(local).join("Warframe").join("EE.log"))
}

fn tail(path: &PathBuf, offset: &mut u64) {
    let Ok(mut file) = File::open(path) else {
        return;
    };

    let length = file.metadata().map(|meta| meta.len()).unwrap_or(0);
    if length < *offset {
        // Log rotated / game restarted.
        *offset = 0;
    }
    if file.seek(SeekFrom::Start(*offset)).is_err() {
        return;
    }

    let mut reader = BufReader::new(file);
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(bytes) => {
                *offset += bytes as u64;
                dispatch(line.trim_end());
            }
            Err(_) => break,
        }
    }
}

/// Dispatches a log line to registered regex handlers. No handlers are
/// registered by default (matching the original behaviour).
fn dispatch(_line: &str) {}
