#!/usr/bin/env python3
"""Adapt pocket-dots' original coverflow picker to standalone Quickshell."""
from pathlib import Path
import sys

path = Path(sys.argv[1])
s = path.read_text()
replacements = {
    'root.home + "/.config/hypr/set-wallpaper.sh"': 'root.home + "/.local/bin/pocket-wallpaper", "set"',
    'root.home + "/.config/quickshell/scripts/wall-thumbs.sh"': 'root.home + "/.config/quickshell/pocket-wallfx/wall-thumbs.sh"',
    'screen: ShellState.focusedScreen': 'screen: Quickshell.screens.find(s => s.name === Hyprland.focusedMonitor?.name) || Quickshell.screens[0]',
    'Appearance.animMed': '210',
    'Appearance.mOut': '110',
    'Appearance.fsL': '24',
    'Colors.bg': '"#171923"',
    'Colors.hairline': '"#667788"',
    'Colors.label': '"#ffffff"',
    'StyledText {': 'Text {',
    'I18n.tr("No hay wallpapers en {0}", "~/Pictures/wallpapers")': '"No wallpapers in ~/Pictures/wallpapers"',
    'GlobalShortcut { name: "wallpaper"; description: "Wallpaper picker"; onPressed: root.open = !root.open }': 'IpcHandler { target: "wallpicker"; function toggle(): string { root.open = !root.open; return root.open ? "open" : "closed" } }',
}
for old,new in replacements.items():
    count=s.count(old)
    if count!=1:
        raise SystemExit(f'Unexpected upstream picker source ({old!r}: {count}). Review upstream changes.')
    s=s.replace(old,new)
path.write_text(s)
