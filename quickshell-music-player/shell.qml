//@ pragma UseQApplication

import QtQuick
import Quickshell
import Quickshell.Services.Mpris
import Quickshell.Wayland

import "components"
import "presets"
import "services"
import "common"

ShellRoot {
    id: root

    // Select the center monitor by output name, not pointer location.
    // Do not create the widgets if that output is disconnected.
    readonly property var desktopScreen: {
        for (let i = 0; i < Quickshell.screens.length; ++i) {
            const candidate = Quickshell.screens[i];
            if (candidate.name === "HDMI-A-1")
                return candidate;
        }
        return null;
    }

    readonly property MprisPlayer activePlayer: MprisController.activePlayer
    // Keep the last playing player through transient Firefox/MPRIS state gaps.
    property MprisPlayer recentPlayer: null
    property bool visibleGrace: false
    Timer {
        id: visibilityGraceTimer
        interval: 2200
        onTriggered: root.visibleGrace = false
    }
    Timer {
        interval: 400
        repeat: true
        running: true
        triggeredOnStart: true
        onTriggered: {
            if (root.activePlayer?.isPlaying) {
                root.recentPlayer = root.activePlayer;
                return;
            }
            // Controller filtering can temporarily lose the playing Firefox entry.
            for (const p of Mpris.players.values) {
                if (p?.isPlaying && p?.trackTitle) {
                    root.recentPlayer = p;
                    return;
                }
            }
        }
    }
    // Keep the exact player that was paused using the desktop widget.
    property MprisPlayer heldPlayer: null
    property bool pausedByWidget: false
    readonly property MprisPlayer displayedPlayer:
        pausedByWidget && heldPlayer !== null && !heldPlayer.isPlaying
            ? heldPlayer
            : activePlayer?.isPlaying ? activePlayer
            : recentPlayer?.isPlaying ? recentPlayer
            : (activePlayer ?? recentPlayer)
    readonly property bool musicPlaying: displayedPlayer !== null && displayedPlayer.isPlaying
    onMusicPlayingChanged: {
        if (musicPlaying) {
            visibleGrace = true;
            visibilityGraceTimer.stop();
            if (pausedByWidget) onResumeFromWidget();
        } else if (!pausedByWidget && visibleGrace) {
            visibilityGraceTimer.restart();
        }
    }

    function onPauseFromWidget() {
        heldPlayer = displayedPlayer;
        pausedByWidget = heldPlayer !== null;
        visibleGrace = false;
        visibilityGraceTimer.stop();
    }
    function onResumeFromWidget() {
        pausedByWidget = false;
        heldPlayer = null;
    }
    onActivePlayerChanged: {
        // A new playing player takes precedence over the manually paused one.
        if (pausedByWidget && activePlayer !== null && activePlayer !== heldPlayer && activePlayer.isPlaying)
            onResumeFromWidget();
    }
    Connections {
        target: root.heldPlayer
        function onTrackTitleChanged() {
            if (root.pausedByWidget) root.onResumeFromWidget();
        }
    }
    readonly property bool lyricsAvailable: LyricsService.status === "ok"
        && LyricsService.lyricsLines.length > 0

    // Request lyrics even when the lyrics window is hidden.
    Component.onCompleted: LyricsService.subscribe()
    Component.onDestruction: LyricsService.unsubscribe()

    PanelWindow {
        id: playerWindow
        screen: root.desktopScreen
        visible: root.desktopScreen !== null && root.displayedPlayer !== null
            && (root.musicPlaying || root.pausedByWidget || root.visibleGrace)
        WlrLayershell.layer: WlrLayer.Bottom
        WlrLayershell.keyboardFocus: WlrKeyboardFocus.None
        exclusionMode: ExclusionMode.Ignore

        anchors {
            bottom: true
            left: true
        }
        margins {
            bottom: 28
            left: 150
        }
        implicitWidth: 430
        implicitHeight: 150
        color: "transparent"

        DesktopMusicCard {
            anchors.fill: parent
            player: root.displayedPlayer
            onPauseRequested: root.onPauseFromWidget()
            onResumeRequested: root.onResumeFromWidget()
        }
    }

    PanelWindow {
        id: lyricsWindow
        screen: root.desktopScreen
        visible: root.desktopScreen !== null
            && (root.musicPlaying || (!root.pausedByWidget && root.visibleGrace))
            && root.lyricsAvailable
        WlrLayershell.layer: WlrLayer.Bottom
        WlrLayershell.keyboardFocus: WlrKeyboardFocus.None
        exclusionMode: ExclusionMode.Ignore

        // On a 1920x1080 central screen: vertically centered, near right edge.
        anchors {
            top: true
            right: true
        }
        margins {
            top: root.desktopScreen !== null
                ? Math.max(0, Math.round((root.desktopScreen.height - implicitHeight) / 2))
                : 0
            right: 50
        }
        implicitWidth: 740
        implicitHeight: 510
        color: "transparent"

        PlayerLyrics {
            anchors.fill: parent
            anchors.margins: 16
            showPlaceholder: false
            textAlignment: Text.AlignHCenter
            baseSize: 28
            activeScale: 1.15
            lineSpacing: 14
            activeColor: "#ffffff"
            textColor: "#b5b5bf"
            indicatorColor: "#ffffff"
        }
    }
}
