import QtQuick
import Quickshell
import Quickshell.Io

// Event-driven policy bridge. The helper owns the base wallpaper and serializes
// calls to pocket-wallpaper; QML never writes the layout configuration.
Scope {
    id: root

    property var player: null
    property bool trackActive: false
    property var layoutSource: null

    readonly property string title: trackActive ? String(player?.trackTitle ?? "").trim() : ""
    readonly property string artist: trackActive ? String(player?.trackArtist ?? "").trim() : ""
    readonly property string songKey: title ? JSON.stringify([title, artist]) : ""
    readonly property string configurationKey: layoutSource ? JSON.stringify(layoutSource.config) : ""

    readonly property string helper: Qt.resolvedUrl("scripts/wallpaper-manager")
        .toString().replace("file://", "")

    // When Firefox changes tracks it can publish half-updated metadata or a
    // temporary empty title. Debounce to avoid flashing back to the base image.
    function scheduleSync() {
        syncTimer.interval = songKey ? 650 : 2600;
        syncTimer.restart();
    }

    function syncTrack() {
        Quickshell.execDetached([
            helper, "sync", root.title, root.artist
        ]);
    }

    function observeWallpaper(path) {
        if (!path) return;
        Quickshell.execDetached([
            helper, "observe", path
        ]);
    }

    Timer {
        id: syncTimer
        repeat: false
        onTriggered: root.syncTrack()
    }

    FileView {
        id: currentWallpaper
        path: Quickshell.env("HOME") + "/.local/state/pocket-wallfx/current"
        blockLoading: true
        watchChanges: true
        printErrors: false
        onFileChanged: reload()
    }

    readonly property string currentPath: currentWallpaper.text().trim()
    onCurrentPathChanged: observeWallpaper(currentPath)
    onSongKeyChanged: scheduleSync()
    onConfigurationKeyChanged: scheduleSync()
    Component.onCompleted: {
        observeWallpaper(currentPath);
        scheduleSync();
    }
}
