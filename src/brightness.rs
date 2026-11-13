//! The brightness of the monitors, over DDC/CI with `ddcutil`. One slider sets every monitor that
//! `ddcutil` finds, and what it is told waits until the slider has stopped moving for a second, so
//! that a drag is one command per monitor rather than a hundred.

use anti_webui::prelude::*;
use std::process::{Command, Stdio};
use std::sync::OnceLock;
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::Duration;

/// How long the monitors wait after the last change before they are told.
const SETTLE: Duration = Duration::from_secs(1);

/// The brightness the panel and the OSD show, from 0 to 1, or `None` when no monitor of the
/// computer can be set this way. It is shared because the commands run on a thread of their own.
struct State {
    level: SharedSignal<Option<f64>>,
    requests: mpsc::Sender<f64>,
}

static STATE: OnceLock<State> = OnceLock::new();

/// The brightness shown, from 0 to 1, if there is a monitor to set.
pub fn level() -> Option<f64> {
    state().level.get()
}

/// Makes the monitors this bright, from 0 to 1, once the slider has been still for a second.
pub fn set(level: f64) {
    let level = level.clamp(0.0, 1.0);
    let state = state();
    state.level.set(Some(level));
    // If the thread is gone there is nothing to tell, but the slider still follows.
    let _ = state.requests.send(level);
}

/// Finds the monitors and reads their brightness now, rather than when the panel first opens.
pub fn install() {
    let _ = state();
}

fn state() -> &'static State {
    STATE.get_or_init(|| {
        let level = shared_signal(None::<f64>);
        let (requests, asked) = mpsc::channel();
        let worker_level = level.clone();
        let _ = std::thread::Builder::new()
            .name("shoji-bar-4 brightness".into())
            .spawn(move || work(asked, worker_level));
        State { level, requests }
    })
}

/// Reads the monitors once, then sets them all whenever the slider comes to rest.
fn work(asked: mpsc::Receiver<f64>, level: SharedSignal<Option<f64>>) {
    let monitors = monitors();
    if monitors.is_empty() {
        level.set(None);
        return;
    }
    level.set(Some(read(&monitors).unwrap_or(1.0)));
    loop {
        let Ok(mut wanted) = asked.recv() else {
            return;
        };
        // Take whatever else comes while the second passes: the last is what stays.
        loop {
            match asked.recv_timeout(SETTLE) {
                Ok(next) => wanted = next,
                Err(RecvTimeoutError::Timeout) => break,
                Err(RecvTimeoutError::Disconnected) => return,
            }
        }
        apply(&monitors, wanted);
    }
}

/// The bus numbers of the monitors `ddcutil` can talk to, from its own idea of them.
fn monitors() -> Vec<String> {
    let Ok(output) = Command::new("ddcutil")
        .args(["detect", "--brief"])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
    else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }
    buses(&String::from_utf8_lossy(&output.stdout))
}

/// The bus numbers of the displays in what `ddcutil detect --brief` says. Its `I2C bus:` is a
/// path like `/dev/i2c-2`, but `--bus` takes the number alone.
fn buses(output: &str) -> Vec<String> {
    output
        .lines()
        .filter_map(|line| line.trim().strip_prefix("I2C bus:"))
        .filter_map(|bus| bus.trim().rsplit('/').next())
        .map(|name| name.strip_prefix("i2c-").unwrap_or(name))
        .filter(|number| !number.is_empty() && number.bytes().all(|b| b.is_ascii_digit()))
        .map(str::to_owned)
        .collect()
}

/// The average of what the monitors say they are at, for the slider to start at.
fn read(monitors: &[String]) -> Option<f64> {
    let mut total = 0.0;
    let mut found = 0u32;
    for bus in monitors {
        let Ok(output) = Command::new("ddcutil")
            .arg("--bus")
            .arg(bus)
            .args(["getvcp", "10", "--brief"])
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .output()
        else {
            continue;
        };
        if !output.status.success() {
            continue;
        }
        if let Some(level) = parse_brightness(&String::from_utf8_lossy(&output.stdout)) {
            total += level;
            found += 1;
        }
    }
    (found > 0).then(|| total / f64::from(found))
}

/// `VCP 10 C 70 100`: the current value and the highest it goes.
fn parse_brightness(output: &str) -> Option<f64> {
    let line = output
        .lines()
        .find(|line| line.trim_start().starts_with("VCP 10 "))?;
    let mut numbers = line.split_whitespace().rev();
    let max: f64 = numbers.next()?.parse().ok()?;
    let current: f64 = numbers.next()?.parse().ok()?;
    (max > 0.0).then(|| (current / max).clamp(0.0, 1.0))
}

/// Tells every monitor, one after another, so that one that is off or slow does not keep the rest
/// from changing.
fn apply(monitors: &[String], level: f64) {
    let value = (level.clamp(0.0, 1.0) * 100.0).round().to_string();
    for bus in monitors {
        let _ = Command::new("ddcutil")
            .arg("--bus")
            .arg(bus)
            .args(["setvcp", "10"])
            .arg(&value)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
}

#[cfg(test)]
mod tests {
    use super::{buses, parse_brightness};

    #[test]
    fn finds_every_bus_ddcutil_reports() {
        let output = "Display 1\n\
                      \x20  I2C bus:          /dev/i2c-2\n\
                      \x20  DRM connector:    card1-HDMI-A-1\n\
                      \n\
                      Display 2\n\
                      \x20  I2C bus:          /dev/i2c-3\n\
                      \x20  DRM connector:    card1-DP-1\n";
        assert_eq!(buses(output), ["2", "3"]);
        assert_eq!(buses(""), Vec::<String>::new());
    }

    #[test]
    fn reads_the_value_and_the_highest_from_getvcp() {
        assert_eq!(parse_brightness("VCP 10 C 70 100\n"), Some(0.7));
        assert_eq!(parse_brightness("VCP 10 C 0 100\n"), Some(0.0));
        assert_eq!(parse_brightness("VCP 10 C 100 100\n"), Some(1.0));
    }

    #[test]
    fn has_nothing_for_a_monitor_that_does_not_say() {
        assert_eq!(parse_brightness(""), None);
        assert_eq!(parse_brightness("VCP 10 C x 100\n"), None);
        assert_eq!(parse_brightness("VCP 10 C 0 0\n"), None);
    }
}
