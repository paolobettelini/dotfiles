import QtQuick
import Quickshell
import Quickshell.Io

// Read-only layout config. Coordinates are relative to the selected monitor's top-left.
Scope {
    id: root

    // Wait before applying a different wallpaper's coordinates, so the WallFX
    // transition can cover the repositioning. No delay on initial startup.
    property int switchDelayMs: 1000
    property bool initialized: false
    property bool layoutSwitching: false
    property string layoutWallpaperName: ""

    FileView {
        id: positionsFile
        path: Qt.resolvedUrl("desktop-widget-positions.json")
        blockLoading: true
        watchChanges: true
        onFileChanged: reload()
    }

    FileView {
        id: wallpaperState
        path: Quickshell.env("HOME") + "/.local/state/pocket-wallfx/current"
        blockLoading: true
        printErrors: false
        watchChanges: true
        onFileChanged: reload()
    }

    readonly property var config: {
        const contents = positionsFile.text().trim();
        if (!contents) return ({});
        try {
            const parsed = JSON.parse(contents);
            return parsed && typeof parsed === "object" && !Array.isArray(parsed)
                ? parsed : ({});
        } catch (error) {
            console.warn("[WallpaperPositions] Invalid JSON:", error);
            return ({});
        }
    }

    readonly property string wallpaperName: {
        const path = wallpaperState.text().trim();
        return path ? path.substring(path.lastIndexOf("/") + 1) : "";
    }

    onWallpaperNameChanged: {
        if (!initialized) {
            layoutWallpaperName = wallpaperName;
        } else if (wallpaperName !== layoutWallpaperName) {
            layoutSwitching = true;
            layoutDelay.restart();
        } else {
            layoutDelay.stop();
            layoutSwitching = false;
        }
    }

    Component.onCompleted: {
        layoutWallpaperName = wallpaperName;
        initialized = true;
    }

    Timer {
        id: layoutDelay
        interval: root.switchDelayMs
        repeat: false
        onTriggered: {
            root.layoutWallpaperName = root.wallpaperName;
            root.layoutSwitching = false;
        }
    }

    readonly property var wallpaperOverride: {
        const entries = config.wallpapers;
        const name = root.layoutWallpaperName;
        return entries && name && Object.prototype.hasOwnProperty.call(entries, name)
            ? entries[name] : null;
    }

    readonly property string monitor: {
        const overrideName = wallpaperOverride && wallpaperOverride.monitor;
        if (typeof overrideName === "string" && overrideName.trim())
            return overrideName.trim();
        const defaultName = config.monitor;
        return typeof defaultName === "string" && defaultName.trim()
            ? defaultName.trim() : "DP-2";
    }

    function coordinate(values, key, fallback) {
        const value = values && values[key];
        return typeof value === "number" && isFinite(value)
            ? Math.round(value) : fallback;
    }

    readonly property int playerX: coordinate(wallpaperOverride, "playerX", coordinate(config, "playerX", 48))
    readonly property int playerY: coordinate(wallpaperOverride, "playerY", coordinate(config, "playerY", 48))
    readonly property int lyricsX: coordinate(wallpaperOverride, "lyricsX", coordinate(config, "lyricsX", 560))
    readonly property int lyricsY: coordinate(wallpaperOverride, "lyricsY", coordinate(config, "lyricsY", 80))
}
