# Manjaro Sway Integration

This directory contains configuration files for integrating SmellyType into [Manjaro Sway](https://github.com/manjaro-sway/manjaro-sway) as the default voice-to-text tool.

## Overview

SmellyType provides push-to-talk voice-to-text for Wayland compositors. It uses OpenAI's Whisper for local, private speech recognition with no cloud services required.

**Why SmellyType for Manjaro Sway:**
- Single Rust binary with no runtime dependencies
- Native Wayland support via wtype
- Sway binding mode integration for reliable push-to-talk
- Built-in Waybar module with Nerd Font icons
- Already packaged in AUR (`smellytype`, `smellytype-bin`)

## Required PRs

### 1. manjaro-sway/iso-profiles

Add to `community/sway/Packages-Desktop`:

```
smellytype
wtype
```

### 2. manjaro-sway/desktop-settings

#### A. Waybar config template

Add to `community/sway/usr/share/sway/templates/waybar/config.jsonc`:

In `modules-right` array (suggested position after `pulseaudio`):
```json
"custom/smellytype",
```

Add module definition:
```json
"custom/smellytype": {
    "exec": "smellytype status --follow --format json --icon-theme nerd-font",
    "return-type": "json",
    "format": "{}",
    "tooltip": true,
    "on-click": "smellytype record toggle",
    "on-click-right": "systemctl --user restart smellytype"
},
```

#### B. Waybar styles

Add to `community/sway/usr/share/sway/templates/waybar/style.css`:

```css
#custom-smellytype {
    padding: 0 8px;
}

#custom-smellytype.idle {
    color: @theme_text_color;
}

#custom-smellytype.recording {
    color: @error_color;
    animation: blink-critical 1s ease-in-out infinite;
}

#custom-smellytype.transcribing {
    color: @warning_color;
}

#custom-smellytype.stopped {
    color: alpha(@theme_text_color, 0.5);
}
```

#### C. Sway keybindings

Create `community/sway/etc/sway/config.d/97-smellytype.conf`:

```bash
# SmellyType push-to-talk voice-to-text
# Hold ScrollLock while speaking, release to transcribe

bindsym --no-repeat Scroll_Lock exec smellytype record start; mode smellytype

mode smellytype {
    bindsym --release Scroll_Lock exec smellytype record stop; mode default
    bindsym Escape exec smellytype record cancel; mode default
}
```

#### D. Default config

Create `community/sway/etc/skel/.config/smellytype/config.toml`:

```toml
model = "base.en"
device = "default"
output_method = "wtype"
state_file = "auto"

[status]
icon_theme = "nerd-font"
```

#### E. Systemd user service autostart

Add to existing sway autostart config or create new file:

```bash
exec_always --no-startup-id systemctl --user start smellytype.service
```

### 3. manjaro-sway/packages (optional)

If they want to host the package in their repo instead of pulling from AUR:

Create a workflow to build from the existing PKGBUILD at:
https://aur.archlinux.org/packages/smellytype

## Testing

On a Manjaro Sway system:

```bash
# Install from AUR
yay -S smellytype wtype

# Copy configs
mkdir -p ~/.config/smellytype
cp skel/config.toml ~/.config/smellytype/

# Add keybindings to sway
mkdir -p ~/.config/sway/config.d
cp sway/smellytype-keybindings.conf ~/.config/sway/config.d/97-smellytype.conf

# Start service
systemctl --user enable --now smellytype

# Test
# Hold ScrollLock, speak, release
```

## File Manifest

```
contrib/manjaro-sway/
├── README.md                      # This file
├── waybar/
│   ├── smellytype-module.jsonc       # Waybar module definition
│   └── smellytype-style.css          # Waybar CSS styles
├── sway/
│   ├── smellytype-keybindings.conf   # Sway push-to-talk bindings
│   └── smellytype-autostart.conf     # Systemd service autostart
└── skel/
    └── config.toml                # Default user config
```

## Links

- SmellyType: https://voxtype.io
- GitHub: https://github.com/peteonrails/voxtype
- AUR: https://aur.archlinux.org/packages/smellytype
- Manjaro Sway: https://github.com/manjaro-sway/manjaro-sway
