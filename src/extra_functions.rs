//! Actions available from the system-controls island on the extra top bar.

use crate::quick_settings::section;
use crate::theme::{TextStyle, colors};
use crate::ui::*;
use anti_webui::prelude::*;
use anti_webui_md3e::components::sized_icon;
use anti_webui_md3e::icons;
use std::io::Write;
use std::process::{Command, Stdio};
use std::time::Duration;

pub fn page() -> Div {
    div()
        .flex()
        .flex_col()
        .gap(12.0_f32)
        .pb(8.0_f32)
        .child(settle_in(
            0,
            crate::qs_pages::header("Extra functions", div()),
        ))
        .child(settle_in(
            1,
            static_txt(TextStyle::BODY_MEDIUM, "Tools for the extra top bar - Hint, right click on the control island to immediately open this!")
                .px(8.0_f32)
                .dyn_style(|s| s.text_color(colors().on_surface_variant)),
        ))
        .child(settle_in(
            2,
            section().child(screen_recording_action()),
        ))
        .child(settle_in(
            3,
            section().child(extra_function(
                icon_badge(
                    sized_icon(icons::COLORIZE, 20.0),
                    |c| c.secondary_container,
                    |c| c.on_secondary_container,
                ),
                "Hyprpicker",
                TextStyle::BODY_SMALL
                    .apply(div().child("After a one-second delay, pick a color; its output is copied to your clipboard."))
                    .dyn_style(|s| s.text_color(colors().on_surface_variant)),
                run_hyprpicker,
            )),
        ))
}

fn screen_recording_action() -> Div {
    let recording = crate::screenrec::is_recording();
    let icon = icon_badge(
        dyn_sym(
            move || {
                if recording.get() {
                    icons::STOP_CIRCLE
                } else {
                    icons::SCREEN_RECORD
                }
            },
            20.0,
            move |c| {
                if recording.get() {
                    c.error
                } else {
                    c.on_secondary_container
                }
            },
        ),
        move |c| {
            if recording.get() {
                c.error_container
            } else {
                c.secondary_container
            }
        },
        move |c| {
            if recording.get() {
                c.on_error_container
            } else {
                c.on_secondary_container
            }
        },
    );
    let description = TextStyle::BODY_SMALL
        .apply(div().child(text(move || {
            if recording.get() {
                "Recording · click to stop".to_owned()
            } else {
                "Select an area to record".to_owned()
            }
        })))
        .dyn_style(move |s| {
            s.text_color(if recording.get() {
                colors().error
            } else {
                colors().on_surface_variant
            })
        });
    extra_function(
        icon,
        "Screen recording",
        description,
        crate::screenrec::toggle_recording,
    )
}

fn extra_function(
    icon: Div,
    name: &'static str,
    description: Div,
    run: impl Fn() + 'static,
) -> Div {
    pressable(
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(12.0_f32)
            .min_h(64.0)
            .px(8.0_f32)
            .py(8.0_f32)
            .rounded(18.0_f32)
            .child(
                div()
                    .size(40.0)
                    .rounded(20.0_f32)
                    .flex()
                    .items_center()
                    .justify_center()
                    .flex_none()
                    .child(icon),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w(0.0)
                    .gap(3.0_f32)
                    .child(static_txt(TextStyle::TITLE_SMALL, name))
                    .child(description),
            ),
        |c| c.on_surface,
    )
    .cursor(CursorIcon::Pointer)
    .on_click(move |_| run())
}

fn icon_badge(
    icon: Div,
    background: impl crate::ui::Role,
    foreground: impl crate::ui::Role,
) -> Div {
    div()
        .size(40.0)
        .rounded(20.0_f32)
        .flex()
        .items_center()
        .justify_center()
        .flex_none()
        .dyn_style(move |s| {
            s.bg(background(&colors()))
                .text_color(foreground(&colors()))
        })
        .child(icon)
}

fn run_hyprpicker() {
    crate::panels::close();
    let result = std::thread::Builder::new()
        .name("shoji-bar-4 hyprpicker".into())
        .spawn(|| {
            let result = (|| {
                std::thread::sleep(Duration::from_secs(1));
                let picked = Command::new("hyprpicker")
                    .output()
                    .map_err(|e| format!("Could not run hyprpicker: {e}"))?;
                if !picked.status.success() {
                    return Err("Hyprpicker did not return a color".to_string());
                }
                if picked.stdout.is_empty() {
                    return Err("Hyprpicker returned no color".to_string());
                }

                let mut copier = Command::new("wl-copy")
                    .stdin(Stdio::piped())
                    .spawn()
                    .map_err(|e| format!("Could not run wl-copy: {e}"))?;
                let mut stdin = copier
                    .stdin
                    .take()
                    .ok_or_else(|| "Could not open wl-copy input".to_string())?;
                let copy_result = stdin
                    .write_all(&picked.stdout)
                    .map_err(|e| format!("Could not copy the selected color: {e}"));
                drop(stdin);
                let status = copier
                    .wait()
                    .map_err(|e| format!("Could not wait for wl-copy: {e}"))?;
                copy_result?;
                if !status.success() {
                    return Err("wl-copy could not copy the selected color".to_string());
                }
                Ok(())
            })();

            if let Err(message) = result {
                let _ = Command::new("notify-send")
                    .args(["-a", "shoji-bar-4", "Hyprpicker failed", &message])
                    .spawn();
            }
        });
    if let Err(error) = result {
        let _ = Command::new("notify-send")
            .args([
                "-a",
                "shoji-bar-4",
                "Hyprpicker failed",
                &format!("Could not start picker: {error}"),
            ])
            .spawn();
    }
}
