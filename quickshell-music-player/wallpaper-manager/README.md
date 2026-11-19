# Wallpaper Manager (Rust 2024)

Small event-driven helper for `pocket-wallpaper` and the Quickshell music widget.
This replaces the former `scripts/wallpaper-manager.py` without changing the
CLI or the JSON state format.

## Build

Requires Rust **1.85+** (edition 2024):

```bash
cargo test
cargo build --release
install -Dm755 target/release/wallpaper-manager \
    ~/.local/src/quickshell-music-player/scripts/wallpaper-manager
```

Build from the `quickshell-music-player/wallpaper-manager` directory.
The installed `WallpaperManager.qml` invokes this executable directly, not Python.

## Commands

```bash
~/.local/src/quickshell-music-player/scripts/wallpaper-manager test "Nightcall" "Kavinsky"
~/.local/src/quickshell-music-player/scripts/wallpaper-manager status
~/.local/src/quickshell-music-player/scripts/wallpaper-manager sync "Nightcall" "Kavinsky"
~/.local/src/quickshell-music-player/scripts/wallpaper-manager observe ~/Pictures/wallpapers/9.jpg
```

`test` only checks matching and never changes the wallpaper. The other commands
are normally invoked by Quickshell automatically.

## Policy

- The first matching `wallpapers` rule in `desktop-widget-positions.json` wins.
  Matching is case-insensitive and substring-based, with `authors` and `songs`
  treated as OR conditions. A `title` in the form `Artist - Song` may also
  match an `authors` rule via its artist prefix.
- A matching song temporarily selects its wallpaper through
  `~/.local/bin/pocket-wallpaper set FILE` (or the command in `$PATH`).
- When the song ends or no rule matches, the original manually selected image
  returns. A manual selection during a matched song overrides the rule for the
  remainder of that song.
- Only images directly inside `~/Pictures/wallpapers/` are allowed. Supported
  extensions: `.jpg`, `.jpeg`, `.png`, `.webp`.
- The layout JSON is **read-only**. Persistent state is stored atomically in
  `~/.local/state/pocket-wallfx/music-manager.json`, protected by the existing
  `music-manager.lock` file using Linux `flock`.
- Existing Python-generated `music-manager.json` files are compatible.
- The helper runs only for events. No recurring timer or daemon is created.

## Quickshell integration

`WallpaperManager.qml` executes:

```qml
Quickshell.execDetached([helper, "sync", root.title, root.artist]);
Quickshell.execDetached([helper, "observe", path]);
```

where `helper` resolves to `scripts/wallpaper-manager` (no `.py`).

`WallpaperPositions.qml` applies coordinates after 1000 ms when wallpaper
changes, hiding the card and lyrics until the coordinates are updated.
The selected monitor remains configurable via JSON.
