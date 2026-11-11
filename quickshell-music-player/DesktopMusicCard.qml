import QtQuick
import Quickshell
import Quickshell.Services.Mpris
import "components"
import "services"

Item {
    id: root
    property MprisPlayer player: null
    property real backgroundOpacity: 0.58
    signal pauseRequested()
    signal resumeRequested()

    PlayerBase {
        id: base
        player: root.player
    }

    // A YouTube/Firefox MPRIS player can temporarily report length == position.
    // Never use such a value as the denominator for a seek operation.
    readonly property string trackKey: (root.player?.metadata?.["xesam:url"] ?? "")
        + "|" + (base.effectiveTitle ?? "")
    property real stableDuration: 0
    property real previousDurationSample: 0
    property bool seekCoolingDown: false

    onTrackKeyChanged: {
        stableDuration = 0;
        previousDurationSample = 0;
        seekCoolingDown = false;
    }

    Timer {
        interval: 400
        repeat: true
        running: root.player !== null
        triggeredOnStart: true
        onTriggered: {
            if (!root.player) return;
            // MprisController discards Firefox's bogus position-as-length value.
            // If yt-dlp is installed, it can also obtain a cached YouTube duration.
            const candidate = Number(MprisController.trackLength(root.player));
            const position = Number(base.effectivePosition);
            if (!Number.isFinite(candidate) || candidate <= 15
                    || !Number.isFinite(position) || position < 0
                    || position > candidate + 1) {
                previousDurationSample = 0;
                return;
            }
            // Ignore late bogus samples that match the playback position.
            if (Math.abs(candidate - position) <= 1.5 && root.stableDuration === 0) {
                previousDurationSample = 0;
                return;
            }
            if (previousDurationSample > 0 && Math.abs(candidate - previousDurationSample) < 0.5) {
                if (root.stableDuration === 0 || Math.abs(candidate - root.stableDuration) < 2)
                    root.stableDuration = candidate;
            }
            previousDurationSample = candidate;
        }
    }

    Timer {
        id: seekCooldown
        interval: 1200
        onTriggered: root.seekCoolingDown = false
    }

    Rectangle {
        anchors.fill: parent
        radius: 20
        color: Qt.rgba(0.055, 0.065, 0.09, root.backgroundOpacity)
        border.width: 1
        border.color: "#50ffffff"
    }

    Image {
        x: 15
        y: 15
        width: 115
        height: 115
        source: base.displayedArtFilePath
        fillMode: Image.PreserveAspectCrop
        smooth: true
        cache: true
    }

    Column {
        x: 146
        y: 21
        width: parent.width - x - 16
        spacing: 5
        Text {
            width: parent.width - 8
            text: base.effectiveTitle || "Now playing"
            color: "#ffffff"
            font.pixelSize: 16
            font.bold: true
            elide: Text.ElideRight
        }
        Text {
            width: parent.width - 8
            text: base.effectiveArtist || ""
            color: "#c9c9d0"
            font.pixelSize: 13
            elide: Text.ElideRight
        }
    }

    Row {
        x: 154
        y: 75
        spacing: 14
        Text {
            text: "⏮"
            color: "white"
            font.pixelSize: 24
            MouseArea {
                anchors.fill: parent
                cursorShape: Qt.PointingHandCursor
                onClicked: base.previous()
            }
        }
        Text {
            text: base.effectiveIsPlaying ? "⏸" : "▶"
            color: "white"
            font.pixelSize: 24
            MouseArea {
                anchors.fill: parent
                cursorShape: Qt.PointingHandCursor
                onClicked: {
                    if (base.effectiveIsPlaying) root.pauseRequested();
                    else root.resumeRequested();
                    base.togglePlaying();
                }
            }
        }
        Text {
            text: "⏭"
            color: "white"
            font.pixelSize: 24
            MouseArea {
                anchors.fill: parent
                cursorShape: Qt.PointingHandCursor
                onClicked: base.next()
            }
        }
        // Firefox often does not expose MPRIS volume. Control the PipeWire
        // default output volume instead, in 5% steps (clamped to 100%).
        Text {
            text: "−"
            color: "#c9c9d0"
            font.pixelSize: 24
            MouseArea {
                anchors.fill: parent
                cursorShape: Qt.PointingHandCursor
                onClicked: Quickshell.execDetached([
                    "wpctl", "set-volume", "-l", "1.0", "@DEFAULT_AUDIO_SINK@", "5%-"
                ])
            }
        }
        Text {
            text: "＋"
            color: "#c9c9d0"
            font.pixelSize: 23
            MouseArea {
                anchors.fill: parent
                cursorShape: Qt.PointingHandCursor
                onClicked: Quickshell.execDetached([
                    "wpctl", "set-volume", "-l", "1.0", "@DEFAULT_AUDIO_SINK@", "5%+"
                ])
            }
        }
    }

    // Keep the ORIGINAL wavy slider rendering. The overlay consumes mouse
    // interactions to prevent unreliable Firefox MPRIS duration-based seeks.
    PlayerProgress {
        id: progress
        x: 146
        y: 122
        width: parent.width - x - 20
        height: 14
        position: base.effectivePosition
        length: root.stableDuration
        canSeek: true
        isPlaying: base.effectiveIsPlaying
        highlightColor: "#ffffff"
        trackColor: "#66ffffff"
        enableWavy: true
        scrollable: false
        // Deliberately do not connect onSeekRequested: only the overlay seeks.
    }

    // Larger hitbox (30 px), without changing the 14 px wavy visual.
    MouseArea {
        x: progress.x
        y: progress.y + (progress.height - height) / 2
        width: progress.width
        height: 30
        acceptedButtons: Qt.LeftButton
        cursorShape: root.stableDuration > 0 && base.effectiveCanSeek
            ? Qt.PointingHandCursor : Qt.ArrowCursor
        onClicked: mouse => {
            const total = root.stableDuration;
            if (root.seekCoolingDown || !base.effectiveCanSeek
                    || !Number.isFinite(total) || total <= 15) return;
            const target = Math.max(1, Math.min(total - 3, mouse.x / width * total));
            if (!Number.isFinite(target)) return;
            root.seekCoolingDown = true;
            seekCooldown.restart();
            base.seek(target);
        }
    }
}
