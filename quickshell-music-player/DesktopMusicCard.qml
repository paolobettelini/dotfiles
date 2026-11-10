import QtQuick
import Quickshell.Services.Mpris
import "components"

Item {
    id: root
    property MprisPlayer player: null
    // Alpha affects the card background only, not the artwork or controls.
    property real backgroundOpacity: 0.58

    PlayerBase {
        id: base
        player: root.player
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
        spacing: 28

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
                onClicked: base.togglePlaying()
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
    }

    // Reuse the upstream animated and seekable timeline, not a basic rectangle.
    PlayerProgress {
        id: progress
        x: 146
        y: 122
        width: parent.width - x - 20
        height: 14
        position: base.effectivePosition
        length: base.effectiveLength
        canSeek: base.effectiveCanSeek
        isPlaying: base.effectiveIsPlaying
        highlightColor: "#ffffff"
        trackColor: "#66ffffff"
        enableWavy: true
        onSeekRequested: seconds => base.seek(seconds)
    }
}
