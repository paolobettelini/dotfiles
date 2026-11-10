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
    readonly property bool musicPlaying: activePlayer !== null && activePlayer.isPlaying
    readonly property bool lyricsAvailable: LyricsService.status === "ok"
        && LyricsService.lyricsLines.length > 0

    // Request lyrics even when the lyrics window is hidden.
    Component.onCompleted: LyricsService.subscribe()
    Component.onDestruction: LyricsService.unsubscribe()

    PanelWindow {
        id: playerWindow
        screen: root.desktopScreen
        visible: root.desktopScreen !== null && root.musicPlaying
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
            player: root.activePlayer
        }
    }

    PanelWindow {
        id: lyricsWindow
        screen: root.desktopScreen
        visible: root.desktopScreen !== null && root.musicPlaying && root.lyricsAvailable
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
