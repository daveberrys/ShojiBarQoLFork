//! Actions available from the system-controls island on the extra top bar.

use crate::quick_settings::section;
use crate::theme::{TextStyle, colors};
use crate::ui::*;
use anti_webui::prelude::*;
use anti_webui_md3e::components::sized_icon;
use anti_webui_md3e::icons::{self, Symbol};
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
            static_txt(TextStyle::BODY_MEDIUM, "Tools for the extra top bar")
                .px(8.0_f32)
                .dyn_style(|s| s.text_color(colors().on_surface_variant)),
        ))
        .child(settle_in(
            2,
            section().child(extra_function(
                icons::SCREEN_RECORD,
                "Screen recording",
                "Select an area to record. Reopen this page and select it again to stop.",
                || {
                    crate::panels::close();
                    crate::screenrec::toggle_recording();
                },
            )),
        ))
        .child(settle_in(
            3,
            section().child(extra_function(
                icons::COLORIZE,
                "Hyprpicker",
                "After a one-second delay, pick a color; its output is copied to your clipboard.",
                run_hyprpicker,
            )),
        ))
}

fn extra_function(
    icon: &'static Symbol,
    name: &'static str,
    description: &'static str,
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
                    .dyn_style(|s| {
                        s.bg(colors().secondary_container)
                            .text_color(colors().on_secondary_container)
                    })
                    .child(sized_icon(icon, 20.0)),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w(0.0)
                    .gap(3.0_f32)
                    .child(static_txt(TextStyle::TITLE_SMALL, name))
                    .child(
                        TextStyle::BODY_SMALL
                            .apply(div().child(description))
                            .dyn_style(|s| s.text_color(colors().on_surface_variant)),
                    ),
            ),
        |c| c.on_surface,
    )
    .cursor(CursorIcon::Pointer)
    .on_click(move |_| run())
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
