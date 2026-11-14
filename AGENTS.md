# AGENTS.md

This repository is `shoji-bar-4`, a Rust desktop shell for ShojiWM built on AntiWebUI with a Material 3 Expressive visual layer (`anti-webui-md3e`). It is not a browser app, not a normal web UI, and not a generic GTK/Qt app. It is a Wayland shell surface system that draws the bar, panels, lock screen, notifications, wallpaper and system controls as compositor-owned layers.

## Project summary

- Language: Rust
- Shell/UI stack: `anti-webui` + `anti-webui-md3e`
- Target environment: ShojiWM / Wayland desktop session
- Main runtime entry: `src/main.rs`
- Persisted user config: `$XDG_STATE_HOME/shoji-bar-4/settings.json`
- Generated artifacts: `target/` and docs in `target/doc/` should be ignored

This fork is a QoL fork of the original Shoji Bar 4, with extra support for screen recording, privacy controls, monitor brightness, notifications, layered shell behavior, and Material 3 Expressive styling.

## High-level architecture

The app bootstraps a shell root in `main.rs`, then starts the IPC daemon, theme installation, task synchronization, wallpaper, brightness monitor, OSD, lock handling, and per-output shell layers. The design is split into a few key layers:

- `src/main.rs`: app entrypoint, command dispatch (`msg` / `preview`), shell initialization
- `src/shell_layer.rs`: generic lifecycle for bar/panel/notification surfaces; production Wayland adapter and test adapter share one interface
- `src/placement.rs`: geometry and bar placement state; determines bar edge, thickness, panel offsets
- `src/panels.rs`: open/close state for transient panels (`quick-settings`, `calendar`, etc.)
- `src/settings.rs`: persisted settings JSON; theme mode, palette, wallpaper, bar position, privacy toggles, etc.
- `src/theme.rs`: MD3 scheme generation, dynamic colors, wallpaper-based seed extraction, animated transitions
- `src/bar.rs`: primary bar UI and islands for workspaces, title, media, status, tray, clock
- `src/quick_settings.rs` and `src/qs_settings.rs`: quick settings panel and settings page
- `src/lock.rs`, `src/notify.rs`, `src/osd.rs`, `src/dock.rs`, `src/launcher.rs`, `src/clipboard.rs`, `src/media_panel.rs`, `src/calendar.rs`, `src/power_dialog.rs`: concrete shell surfaces and behavior
- `src/wm.rs`: ShojiWM IPC/theme syncing
- `src/ipc.rs`: socket-based control commands
- `src/brightness.rs`, `src/screenrec.rs`, `src/user.rs`, `src/wallpaper.rs`, `src/appinfo.rs`: system integrations and desktop utilities

## Runtime model

The shell creates layer-shell surfaces for each output and each transient panel. Bar placement is treated as a single placement snapshot: `placement::memoized()` updates geometry without forcing unrelated state to refresh. Panel state is tracked centrally in `panels.rs`, and `shell_layer.rs` keeps the visible layer mounted while it opens, remains open, or closes.

Important runtime principles:

- One bar and one panel invocation per output are common patterns.
- The shell uses `anti_webui` reactive signals/memos/effects to update UI state in response to settings and system changes.
- Most of the code is declarative Rust UI building, not imperative DOM code.
- Theme changes and placement updates are animated via the reactive system and Material motion.

## Settings and persistence

User configuration is stored at `$XDG_STATE_HOME/shoji-bar-4/settings.json` and loaded by `src/settings.rs`.

Key persisted settings include:

- `mode`: light / dark / auto
- `seed`: base Material color
- `seed_from_wallpaper`: derive seed from wallpaper
- `palette`: tonal-spot / vibrant / expressive / fidelity / neutral / monochrome
- `wallpaper`: path
- `pinned`: pinned dock apps
- `bar_position`: top / left / right
- `clock_24h`, `clock_seconds`
- `private_wifi`
- `hide_notification_details`
- `avatar`
- `screenrec_aspect_ratio`, `screenrec_audio`

When editing settings, keep JSON keys consistent with `Settings::to_json()` and `Settings::from_json()`. Missing or invalid keys should fall back to defaults rather than crashing.

## Theme system

`src/theme.rs` owns the Material 3 color generation:

- `scheme_for(seed, palette, dark)` creates a `ColorScheme`
- `ColorScheme::from_seed_with(...)` is the main Material You palette builder
- wallpaper-based seeds are extracted from a small decoded image using `seed_colors(...)`
- animated scheme transitions use `set_color_scheme_animated(...)` with `default_effects()`
- `target_scheme()` provides the final color target for non-reactive elements

The project follows Material 3 Expressive semantics, not a generic CSS theme. The `anti-webui-md3e` layer is responsible for the component styling, icons, shapes, motion and colors.

## Documentation and repo intent

The docs in `docs/` explain the repo's intended behavior:

- `README.md`: overview, build instructions, fork purpose, dependency list
- `docs/features.md`: detailed feature descriptions for bar, dock, quick settings, calendar, launcher, player, clipboard, lock, notifications, OSD, themes, wallpaper
- `docs/usage.md`: how to build/run, ShojiWM integration, IPC socket commands, settings
- `docs/development.md`: AntiWebUI local checkout instructions, preview tooling, layer lifecycle tests, CPU-report flow
- `docs/glossary.md`: shared terminology for bar/placement/panels/shell surfaces

Before changing behavior, read the relevant docs and prefer the existing patterns over improvising new ones.

## AntiWebUI: what it is and how to think about it

`anti-webui` is the desktop shell/UI framework this project is built on. It is not a browser engine, not web DOM code, and not HTML/CSS. It is a Rust-native reactive UI and shell toolkit that handles:

- signal/memo/effect-driven state
- surface/layer creation for compositor integration
- Wayland shell concepts and shell adapters
- geometry, motion, interaction and rendering primitives
- app/system shell features such as tray, battery, media, notifications, session lock, power, apps, network, etc.

In this repo, `anti_webui::prelude::*` and `anti_webui::shell::prelude::*` are the base toolkits for rendering and shell integration. The shell uses layer surfaces, output binding, input regions, and a reactive render tree rather than direct browser DOM programming.

When you edit code here, think in terms of:

- `Signal`, `Memo`, `effect`, `memo`, `task`, `div()`, `dyn_style()`, `on_click()`, `request_frame()`
- shell surfaces and layer lifecycles rather than page routes
- output-aware placement and per-surface state rather than generic page UI

## anti-webui-md3e: what it is and why it matters

`anti-webui-md3e` is the Material 3 Expressive UI layer built on top of AntiWebUI. It provides the design system this shell depends on:

- Material 3 color schemes and dynamic color (`ColorScheme`, `SchemeVariant`, `seed_colors`)
- Material shapes, state layers, ripples and ink behavior
- components such as buttons, sliders, menus, shapes, icons, pickers, and progress widgets
- iconography and motion primitives (`icons`, `motion::fast_spatial`, `default_effects`, etc.)
- Material 3 Expressive visual styling: rounded surfaces, shape morphing, elevated state transitions, and expressive motion

This project clearly depends on `anti_webui_md3e` for almost all visual polish and interaction details. Examples from the source:

- `use anti_webui_md3e::components::{...}`
- `use anti_webui_md3e::icons::{...}`
- `use anti_webui_md3e::motion::{...}`
- `use anti_webui_md3e::{ColorScheme, set_color_scheme_animated}`

This is not “web UI in Rust” and not “a webpage disguised as a shell.” It is a native shell UI built with a reactive desktop toolkit and a Material 3 Expressive component package.

## Key modules to understand before editing

Start here for any change:

1. `src/main.rs`
2. `src/settings.rs`
3. `src/theme.rs`
4. `src/shell_layer.rs`
5. `src/placement.rs`
6. `src/panels.rs`
7. `src/bar.rs`
8. `src/quick_settings.rs`
9. `src/ui.rs`

`src/ui.rs` contains reusable helpers for text, icons, island styling, transitions, and animation that many modules depend on.

## Common editing conventions

- Follow existing module boundaries; do not add a blanket “global” abstraction unless it clearly matches the current structure.
- Preserve the reactive pattern: if a value depends on settings or system state, prefer `memo()`/`effect()` and existing signal flows over ad hoc global state.
- Keep UI concerns and shell concerns separated: shell-layer decisions live in `shell_layer.rs`, placement geometry in `placement.rs`, panel state in `panels.rs`, theme logic in `theme.rs`, and concrete surfaces in their module files.
- Keep JSON-facing settings stable and backward-compatible where possible.
- Prefer existing Material 3/shape/icon/motion helpers from `anti_webui_md3e` instead of inventing custom styles.
- For any local AntiWebUI development, follow `docs/development.md`; use a `.cargo/config.toml` patch if working against a local checkout.

## Build, test and run

Standard build and launch:

```bash
cargo build --release
./target/release/shoji-bar-4
```

Useful dev commands from docs:

```bash
cargo build
cargo test --locked --bin shoji-bar-4 shell_layer::tests
```

Preview tooling (important for UI-only iteration):

```bash
shoji-bar-4 preview dock out.png --pointer 470,110
shoji-bar-4 preview quick-settings out.png --width 460 --height 900 --scale 1 --light
```

See `docs/development.md` for the full preview and CPU-report flow.

## Safety and scope notes for future agents

- Do not treat this as a web project. There is no browser-based UI, no React/Vite setup, and no generic DOM/CSS architecture.
- Ignore `target/` and other generated artifacts when editing or searching.
- Keep `docs/`, `README.md`, and `AGENTS.md` aligned with behavior changes.
- Prefer narrowly scoped edits that preserve the existing shell lifecycle and theme model.
- If you modify the shell surfaces or configuration behavior, verify the update is still compatible with ShojiWM and Wayland layer semantics.
- Avoid compiling, and let the user compile to test the changes.

## Quick file map

- `src/main.rs`: startup and command dispatch
- `src/ipc.rs`: socket command server
- `src/settings.rs`: persisted settings model
- `src/theme.rs`: Material color scheme generation
- `src/wm.rs`: theme sync to ShojiWM
- `src/shell_layer.rs`: layer lifecycle management
- `src/placement.rs`: bar placement math
- `src/panels.rs`: panel open/close state
- `src/bar.rs`: primary bar and islands
- `src/dock.rs`: dock behavior and context menu
- `src/quick_settings.rs`: quick settings panel and controls
- `src/qs_settings.rs`: settings subpage UI
- `src/launcher.rs`: launcher panel
- `src/clipboard.rs`: clipboard history
- `src/media_panel.rs`: player panel
- `src/calendar.rs`: calendar panel and phone-sharing features
- `src/lock.rs`: lock screen and PAM authentication flow
- `src/notify.rs`: notification daemon and popups
- `src/osd.rs`: volume/brightness OSD
- `src/power_dialog.rs`: power actions confirmation
- `src/screenrec.rs`: screen recording integration
- `src/brightness.rs`: DDC brightness control
- `src/wallpaper.rs`: wallpaper handling
- `src/user.rs`: user avatar and desktop image utilities
- `src/appinfo.rs`: app lookup and icon resolution
- `src/ui.rs`: shared UI helpers and material-inspired widgets

## Bottom line

This repo is a Rust-native Wayland shell UI built with AntiWebUI, using `anti-webui-md3e` as its Material 3 Expressive component and styling layer. The most important mental model for future agents is: this is a desktop shell, not a website; use the reactive shell toolkit and Material styling system correctly, and preserve the placement/layer lifecycle and theme logic whenever making changes.
