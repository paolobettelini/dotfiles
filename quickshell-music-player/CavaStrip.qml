import QtQuick
import Quickshell.Io

Item {
    id: root
    required property bool playing
    property var bars: []
    property int barCount: 36

    // CAVA captures the default PipeWire output and streams semicolon-delimited
    // spectrum samples; this process runs only during playback.
    Process {
        id: cavaProcess
        command: ["sh", "-c", "exec cava -p \"$HOME/.config/quickshell/music-cava.conf\""]
        running: root.playing

        stdout: SplitParser {
            splitMarker: "\n"
            onRead: line => {
                const data = String(line).trim().split(";");
                if (data.length < root.barCount) return;
                const next = [];
                for (let i = 0; i < root.barCount; ++i) {
                    const value = Number(data[i]);
                    next.push(Number.isFinite(value)
                        ? Math.max(0, Math.min(1, value / 100)) : 0);
                }
                root.bars = next;
            }
        }
    }

    onPlayingChanged: {
        if (!playing) bars = [];
    }
    onBarsChanged: graph.requestPaint()

    Canvas {
        id: graph
        anchors.fill: parent
        antialiasing: true
        onPaint: {
            const ctx = getContext("2d");
            ctx.clearRect(0, 0, width, height);
            if (!root.playing || root.bars.length !== root.barCount) return;

            const spacing = 3;
            const barWidth = (width - spacing * (root.barCount - 1)) / root.barCount;
            for (let i = 0; i < root.barCount; ++i) {
                const level = root.bars[i];
                const h = Math.max(1, level * height);
                // Blue -> cyan -> magenta across the spectrum.
                const hue = 210 + 100 * i / Math.max(1, root.barCount - 1);
                ctx.fillStyle = `hsla(${hue}, 95%, 70%, 0.95)`;
                ctx.fillRect(i * (barWidth + spacing), height - h, Math.max(1, barWidth), h);
            }
        }
    }
}
