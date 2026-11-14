> [!WARNING]
> Everything here is vibecoded, and that should be already obvious because Shoji bar is vibecoded. No I am not coding this by hand. Yes I do test them and they do work.

> [!NOTE]
> This is a fork of Shoji bar 4. Support the original curator - I highly suggest checking out the original Shoji bar 4 repository linked here: [https://github.com/bea4dev/shoji-bar-4](https://github.com/bea4dev/shoji-bar-4).
>
> This is a QOL fork of Shoji bar 4 because I really like Material Design 3 Expressive and wanted to bring the features I like from Noctalia over to Shoji bar.

# shoji-bar-4 (dvs' QoL fork.)

A fork of the original [shoji-bar-4](https://github.com/bea4dev/shoji-bar-4) baked with QoL features such as listed below.

[Video preview](https://x.com/TheDevDoggo/status/2121363757893063070)

## What this fork adds

This fork introduces quality-of-life features, Noctalia-inspired additions, and fixes for various gripes that were not in the original Shoji bar:

- **Screen recording**: Built-in support for `wf-recorder` and `wl-screenrec`.
- **Privacy features**: Hidden Wi-FI, and hidden notifications.
- **External monitor brightness**: Brightness control via `ddcutil`.
- **QoL improvements**: Various personal fixes and polish throughout the shell.

## Getting started

You will need any desktop environment that supports Shoji, such as Niri, Hyprland, Shoji, or Umbriel, a Rust toolchain, and also the prerequisites: NetworkManager, BlueZ, PipeWire, UPower, power-profiles-daemon, and logind. `cliphist` and `wl-clipboard` give the clipboard history, `zenity` lets you pick a user picture, `ddcutil` sets the brightness of every monitor, and `wf-recorder` or `wl-screenrec` handles screen recording.

### Extra Dependencies
```bash
# Ubuntu
sudo apt install pkg-config libfontconfig1-dev libwayland-dev libpam0g-dev libxcb1-dev libxkbcommon-dev libxkbcommon-x11-dev

# Arch Linux
sudo pacman -S pkgconf fontconfig wayland pam libxcb libxkbcommon libxkbcommon-x11

# Fedora
sudo dnf install pkgconf-pkg-config fontconfig-devel wayland-devel pam-devel libxcb-devel libxkbcommon-devel libxkbcommon-x11-devel
```

### Compiling
```sh
git clone https://github.com/daveberrys/shoji-bar-4 ~/.config/shoji-bar-4
cd ~/.config/shoji-bar-4
cargo build --release
```

## Getting nightly.link
TBA

## Documentation

- [Features](docs/features.md): everything the shell does, part by part.
- [Usage](docs/usage.md): running it, ShojiWM integration, commands and settings.
- [Development](docs/development.md): building against a local AntiWebUI, offscreen previews and CPU reports.
- [Glossary](docs/glossary.md): the bar, islands, panels, placements and shell surfaces' shared language.