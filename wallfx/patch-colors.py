#!/usr/bin/env python3
"""Remove pocket-dots' theme singleton dependency from standalone WallpaperFx."""
from pathlib import Path
import sys

path = Path(sys.argv[1])
source = path.read_text()
old = 'const pal = [Colors.bg, Colors.c1, Colors.c2, Colors.c3, Colors.c4, Colors.c5, Colors.dim, Colors.fg]'
new = 'const pal = [Qt.rgba(0.09, 0.10, 0.14, 1), Qt.rgba(0.88, 0.42, 0.46, 1), Qt.rgba(0.60, 0.76, 0.47, 1), Qt.rgba(0.38, 0.69, 0.94, 1), Qt.rgba(0.78, 0.47, 0.87, 1), Qt.rgba(0.90, 0.75, 0.48, 1), Qt.rgba(0.67, 0.72, 0.75, 1), Qt.rgba(1, 1, 1, 1)]'
if source.count(old) != 1:
    raise SystemExit('Upstream WallpaperFx.qml has changed; review the patch manually.')
path.write_text(source.replace(old, new))
