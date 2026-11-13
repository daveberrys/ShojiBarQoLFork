//! Screen recording using slurp and wf-recorder / wf-screenrec.

use crate::settings::settings;
use crate::ui::*;
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use anti_webui_md3e::ColorScheme;
use anti_webui_md3e::icons;
use std::cell::Cell;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

static RECORDER_PID: AtomicU32 = AtomicU32::new(0);
static SLURP_PID: AtomicU32 = AtomicU32::new(0);

thread_local! {
    static RECORDING: Cell<Option<Signal<bool>>> = const { Cell::new(None) };
    static SENDER: Cell<Option<UiSender<RecordEvent>>> = const { Cell::new(None) };
}

/// Whether a screen recording is currently taking place.
/// Created in a root that lives outside any panel so it is never disposed.
pub fn is_recording() -> Signal<bool> {
    RECORDING.with(|r| match r.get() {
        Some(s) => s,
        None => {
            let (s, _) = create_root(|| signal(false));
            r.set(Some(s));
            s
        }
    })
}

#[derive(Clone, Copy, Debug)]
enum RecordEvent {
    Started,
    Stopped,
}

/// UI channel for recording events, kept alive across panel open/close cycles.
fn event_sender() -> UiSender<RecordEvent> {
    SENDER.with(|c| {
        let s = c.take().unwrap_or_else(|| {
            let (sender, _) = create_root(|| {
                ui_channel(move |event: RecordEvent| {
                    let rec = is_recording();
                    match event {
                        RecordEvent::Started => rec.set(true),
                        RecordEvent::Stopped => rec.set(false),
                    }
                })
            });
            sender
        });
        c.set(Some(s.clone()));
        s
    })
}

/// Toggles screen recording: starts if idle, stops if active.
pub fn toggle_recording() {
    let recording = is_recording();
    if recording.get() {
        stop_recording();
        recording.set(false);
    } else {
        start_recording();
    }
}

/// Stops any ongoing recording or selection.
pub fn stop_recording() {
    let slurp_pid = SLURP_PID.swap(0, Ordering::SeqCst);
    if slurp_pid > 0 {
        let _ = Command::new("kill").args(["-TERM", &slurp_pid.to_string()]).status();
    }
    let rec_pid = RECORDER_PID.swap(0, Ordering::SeqCst);
    if rec_pid > 0 {
        let _ = Command::new("kill").args(["-INT", &rec_pid.to_string()]).status();
    }
}

fn recordings_dir() -> PathBuf {
    std::env::var_os("XDG_VIDEOS_DIR")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Videos")))
        .unwrap_or_else(std::env::temp_dir)
}

fn is_executable(cmd: &str) -> bool {
    if let Some(path_var) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path_var) {
            let candidate = dir.join(cmd);
            if candidate.is_file() {
                return true;
            }
        }
    }
    false
}

fn recorder_command() -> &'static str {
    if is_executable("wf-screenrec") {
        "wf-screenrec"
    } else {
        "wf-recorder"
    }
}

/// Ensures the width and height of the geometry are even numbers,
/// preventing codec errors with YUV420p video encoding.
pub fn sanitize_geometry(geom: &str) -> String {
    let geom = geom.trim();
    if let Some((pos, size)) = geom.split_once(' ') {
        if let Some((w_str, h_str)) = size.split_once('x') {
            if let (Ok(mut w), Ok(mut h)) = (w_str.parse::<u32>(), h_str.parse::<u32>()) {
                if w % 2 != 0 {
                    w = w.saturating_sub(1);
                }
                if h % 2 != 0 {
                    h = h.saturating_sub(1);
                }
                return format!("{pos} {w}x{h}");
            }
        }
    }
    geom.to_string()
}

fn start_recording() {
    let recording = is_recording();
    recording.set(true);

    crate::panels::close();

    let ratio = settings().with(|s| s.screenrec_aspect_ratio.clone());
    let audio = settings().with(|s| s.screenrec_audio);
    let timestamp = local_time(Every::Second).format("%Y-%m-%d_%H-%M-%S");
    let dir = recordings_dir();
    let output_path = dir.join(format!("{timestamp}.mp4"));

    let to_ui = event_sender();

    let _ = std::thread::Builder::new()
        .name("shoji-bar-4 screenrec".into())
        .spawn(move || {
            // Give panels a brief moment to close and unmap.
            std::thread::sleep(Duration::from_millis(200));

            let mut slurp_cmd = Command::new("slurp");
            let ratio_clean = ratio.trim();
            if !ratio_clean.is_empty()
                && !ratio_clean.eq_ignore_ascii_case("free")
                && !ratio_clean.eq_ignore_ascii_case("any")
                && !ratio_clean.eq_ignore_ascii_case("anything")
                && !ratio_clean.eq_ignore_ascii_case("none")
            {
                slurp_cmd.args(["-a", ratio_clean]);
            }

            slurp_cmd.stdout(Stdio::piped()).stderr(Stdio::null());

            let Ok(slurp_child) = slurp_cmd.spawn() else {
                let _ = Command::new("notify-send")
                    .args(["-a", "shoji-bar-4", "Screen record failed", "Could not run slurp"])
                    .spawn();
                to_ui.send(RecordEvent::Stopped);
                return;
            };

            SLURP_PID.store(slurp_child.id(), Ordering::SeqCst);
            let Ok(slurp_out) = slurp_child.wait_with_output() else {
                SLURP_PID.store(0, Ordering::SeqCst);
                to_ui.send(RecordEvent::Stopped);
                return;
            };
            SLURP_PID.store(0, Ordering::SeqCst);

            if !slurp_out.status.success() {
                to_ui.send(RecordEvent::Stopped);
                return;
            }

            let raw_geom = String::from_utf8_lossy(&slurp_out.stdout);
            let geom = sanitize_geometry(&raw_geom);
            if geom.is_empty() {
                to_ui.send(RecordEvent::Stopped);
                return;
            }

            let _ = std::fs::create_dir_all(&dir);

            let bin = recorder_command();
            let mut rec_cmd = Command::new(bin);
            rec_cmd.args(["-g", &geom, "-f", &output_path.to_string_lossy()]);
            if audio {
                rec_cmd.arg("-a");
            }

            let Ok(mut rec_child) = rec_cmd.spawn() else {
                let _ = Command::new("notify-send")
                    .args([
                        "-a",
                        "shoji-bar-4",
                        "Screen record failed",
                        &format!("Could not run {bin}"),
                    ])
                    .spawn();
                to_ui.send(RecordEvent::Stopped);
                return;
            };

            RECORDER_PID.store(rec_child.id(), Ordering::SeqCst);
            to_ui.send(RecordEvent::Started);

            let file_name = output_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy();
            let _ = Command::new("notify-send")
                .args([
                    "-a",
                    "shoji-bar-4",
                    "-i",
                    "media-record",
                    "Recording started",
                    &format!("Saving to {file_name}"),
                ])
                .spawn();

            let status = rec_child.wait();
            RECORDER_PID.store(0, Ordering::SeqCst);
            to_ui.send(RecordEvent::Stopped);

            let success = status.is_ok_and(|s| s.success());
            if success && output_path.exists() {
                let _ = Command::new("notify-send")
                    .args([
                        "-a",
                        "shoji-bar-4",
                        "-i",
                        "video-x-generic",
                        "Recording saved",
                        &output_path.to_string_lossy(),
                    ])
                    .spawn();
            }
        });
}

/// The button in quick settings to record or stop recording.
pub fn screenrec_button() -> Div {
    let recording = is_recording();
    pressable(
        div()
            .size(34.0_f32)
            .flex()
            .items_center()
            .justify_center()
            .flex_none()
            .rounded(17.0_f32)
            .child(dyn_sym(
                move || {
                    if recording.get() {
                        icons::STOP_CIRCLE
                    } else {
                        icons::SCREEN_RECORD
                    }
                },
                19.0_f32,
                move |c: &ColorScheme| {
                    if recording.get() {
                        c.error
                    } else {
                        c.on_surface_variant
                    }
                },
            )),
        move |c: &ColorScheme| {
            if recording.get() {
                c.error
            } else {
                c.on_surface_variant
            }
        },
    )
    .cursor(CursorIcon::Pointer)
    .on_click(|_| {
        toggle_recording();
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_geometry_odd_dimensions() {
        assert_eq!(sanitize_geometry("100,200 1281x721"), "100,200 1280x720");
        assert_eq!(sanitize_geometry("0,0 1920x1080\n"), "0,0 1920x1080");
        assert_eq!(sanitize_geometry("50,60 1281x720"), "50,60 1280x720");
        assert_eq!(sanitize_geometry("10,20 1280x721"), "10,20 1280x720");
    }

    #[test]
    fn test_recorder_command() {
        let cmd = recorder_command();
        assert!(cmd == "wf-screenrec" || cmd == "wf-recorder");
    }
}
