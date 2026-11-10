pragma Singleton
pragma ComponentBehavior: Bound

import QtQuick
import Quickshell
import Quickshell.Io
import Quickshell.Services.Mpris
import "../common"
import "../services"

Singleton {
    id: root

    readonly property MprisPlayer activePlayer: MprisController.activePlayer

    property int subscribers: 0
    readonly property bool active: root.subscribers > 0

    property var lyricsLines: []
    property int activeIndex: -1
    property real currentPosition: 0
    property string status: "idle"

    readonly property int before: 3
    readonly property int after: 3
    readonly property int total: 7
    property var slots: ["", "", "", "", "", "", ""]

    property int _requestSerial: 0
    property string _latestRequestId: ""
    property string _latestTrackKey: ""
    property string _publishedTrackKey: ""
    property var _pendingRequest: null
    property string _runningRequestId: ""
    property string _runningTrackKey: ""

    function subscribe(): void {
        root.subscribers++;
    }

    function unsubscribe(): void {
        root.subscribers = Math.max(0, root.subscribers - 1);
    }

    property real _ignoreReportedUntil: 0

    function seekToTime(seconds: real): void {
        if (typeof seconds !== "number" || isNaN(seconds))
            return;
        const target = Math.max(0, seconds);
        root.currentPosition = target;
        root._reanchor(target);
        root._lastReported = target;
        // Ignore stale D-Bus reports for 800ms after seeking so player doesn't bounce back
        root._ignoreReportedUntil = Date.now() + 800;

        let idx = -1;
        for (let i = 0; i < root.lyricsLines.length; i++) {
            if (root.lyricsLines[i].time <= target)
                idx = i;
            else
                break;
        }
        root.activeIndex = idx;
        root.slots = root.buildSlots(idx);

        const isYt = (YtMusic.mpvPlayer && root.activePlayer === YtMusic.mpvPlayer) || MprisController._isYtMusicMpv(root.activePlayer);
        if (isYt) {
            YtMusic.seek(target);
        } else if (root.activePlayer) {
            const dbusName = root.activePlayer.dbusName || "";
            const playerName = dbusName.replace("org.mpris.MediaPlayer2.", "");
            if (playerName !== "") {
                Quickshell.execDetached(["playerctl", "-p", playerName, "position", String(target)]);
            } else {
                Quickshell.execDetached(["playerctl", "position", String(target)]);
            }
        }
    }

    function buildSlots(idx: int): var {
        const result = [];
        for (let i = 0; i < root.total; i++) {
            const lineIdx = idx - root.before + i;
            if (lineIdx >= 0 && lineIdx < root.lyricsLines.length)
                result.push(root.lyricsLines[lineIdx].text || "♪");
            else
                result.push("");
        }
        return result;
    }

    function _clearPublished(): void {
        root.lyricsLines = [];
        root.activeIndex = -1;
        root.slots = ["", "", "", "", "", "", ""];
        root._publishedTrackKey = "";
        root._lastReported = -1;
        root._reanchor(0);
    }

    function _snapshot(): var {
        const player = root.activePlayer;
        let title = player?.trackTitle ?? "";
        let artist = player?.trackArtist ?? "";
        const album = player?.trackAlbum ?? "";

        // Normalize common YouTube music uploads before searching LRCLIB/NetEase.
        // Example: "Paul Simon - Still Crazy After All These Years (Official Audio)"
        //          + "PaulSimonVEVO" -> "Still Crazy After All These Years" / "Paul Simon".
        const browserPlayer = MprisController._isBrowserPlayer(player)
            || MprisController._isBrowserYoutubePlayer(player);
        if (browserPlayer) {
            title = title.replace(/\s*[([{]\s*(official\s+(music\s+)?video|official\s+audio|lyrics?|visuali[sz]er|audio)\s*[)\]}]\s*/ig, " ").trim();
            title = title.replace(/\s*\|\s*(official\s+video|official\s+audio|lyrics?).*$/i, "").trim();
            if (title.includes(" - ")) {
                const parts = title.split(" - ");
                if (parts.length === 2 && parts[0].trim() && parts[1].trim()) {
                    const prefix = parts[0].trim();
                    // A channel name ending in VEVO is often not a clean artist tag.
                    const channelArtist = artist.replace(/VEVO$/i, "").replace(/\s+/g, "").toLowerCase();
                    const prefixArtist = prefix.replace(/\s+/g, "").toLowerCase();
                    if (!artist || channelArtist === prefixArtist || artist.toLowerCase().endsWith("vevo")) {
                        artist = prefix;
                        title = parts[1].trim();
                    }
                }
            }
        }
        const duration = player?.length ?? 0;
        // AMLL indexes word-level lyrics by Spotify track id, and MPRIS already
        // carries it as /com/spotify/track/<id> — no searching required.
        const trackId = String(player?.metadata?.["mpris:trackid"] ?? "");
        const spotifyId = trackId.includes("/spotify/track/") ? trackId.split("/").pop() : "";
        const key = JSON.stringify([
            player?.dbusName ?? "",
            player?.uniqueId ?? 0,
            title,
            artist,
            album,
            Math.floor(duration)
        ]);
        return {
            key: key,
            spotifyId: spotifyId,
            title: title,
            artist: artist,
            album: album,
            duration: duration
        };
    }

    function _invalidateRequests(): void {
        root._requestSerial++;
        root._latestRequestId = String(root._requestSerial);
        root._pendingRequest = null;
        if (lyricsProc.running)
            lyricsProc.running = false;
    }

    function scheduleRefresh(): void {
        if (!root.active) {
            refreshTimer.stop();
            root._invalidateRequests();
            return;
        }
        refreshTimer.restart();
    }

    function _queueCurrentTrack(): void {
        if (!root.active)
            return;

        const snapshot = root._snapshot();
        const settled = root.status === "ok" || root.status === "not_found"
            || root.status === "no_info" || root.status === "error";
        const inFlight = lyricsProc.running || root._pendingRequest !== null;
        if (snapshot.key === root._latestTrackKey && (settled || inFlight))
            return;

        if (snapshot.key !== root._publishedTrackKey)
            root._clearPublished();

        root._requestSerial++;
        const requestId = String(root._requestSerial);
        root._latestRequestId = requestId;
        root._latestTrackKey = snapshot.key;
        const player = root.activePlayer;
        if (!snapshot.title || !snapshot.artist) {
            root._pendingRequest = null;
            if (lyricsProc.running)
                lyricsProc.running = false;
            root._publishFailure(requestId, "no_info");
            return;
        }

        root._pendingRequest = {
            requestId: requestId,
            trackKey: snapshot.key,
            spotifyId: snapshot.spotifyId,
            title: snapshot.title,
            artist: snapshot.artist,
            album: snapshot.album,
            duration: snapshot.duration
        };
        root.status = "loading";

        if (lyricsProc.running)
            lyricsProc.running = false;
        else
            root._startPendingRequest();
    }

    function _startPendingRequest(): void {
        if (!root.active || root._pendingRequest === null || lyricsProc.running)
            return;

        const request = root._pendingRequest;
        if (request.requestId !== root._latestRequestId) {
            root._pendingRequest = null;
            return;
        }

        root._pendingRequest = null;
        root._runningRequestId = request.requestId;
        root._runningTrackKey = request.trackKey;
        lyricsProc.command = [
            "python3",
            Qt.resolvedUrl("../scripts/lyrics/lyrics.py").toString().replace("file://", ""),
            request.title,
            request.artist,
            request.album,
            String(Math.floor(request.duration)),
            request.requestId,
            request.spotifyId ?? ""
        ];
        lyricsProc.running = true;
    }

    function _publishFailure(requestId: string, nextStatus: string): void {
        Qt.callLater(() => {
            if (!root.active || requestId !== root._latestRequestId)
                return;
            root.status = nextStatus;
            root._clearPublished();
            root._scheduleRetry(nextStatus);
        });
    }

    // Self-healing. "error" means a provider could not be reached (a stalled
    // connection, a rate limit), not that the song has no lyrics — and an error
    // counted as settled, so one bad moment used to stick until the track
    // changed. Retry it a few times with growing gaps. "not_found" is a real
    // answer from every provider and is left alone.
    readonly property var _retryDelays: [4000, 15000, 45000]
    property int _retryCount: 0
    property string _retryTrackKey: ""

    function _scheduleRetry(status: string): void {
        if (status !== "error") {
            root._retryCount = 0;
            return;
        }
        const key = root._latestTrackKey;
        if (key !== root._retryTrackKey) {
            root._retryTrackKey = key;
            root._retryCount = 0;
        }
        if (root._retryCount >= root._retryDelays.length)
            return;
        retryTimer.interval = root._retryDelays[root._retryCount];
        root._retryCount++;
        retryTimer.restart();
    }

    Timer {
        id: retryTimer
        repeat: false
        onTriggered: {
            // Only retry the track that failed; a new track brings its own fetch.
            if (!root.active || root.status !== "error"
                    || root._snapshot().key !== root._retryTrackKey)
                return;
            root._latestTrackKey = "";
            root._queueCurrentTrack();
        }
    }

    function _publishSuccess(requestId: string, lines: var): void {
        Qt.callLater(() => {
            if (!root.active || requestId !== root._latestRequestId)
                return;

            root._retryCount = 0;
            root.lyricsLines = lines;
            root.activeIndex = -1;
            root.slots = root.buildSlots(-1);
            root._lastReported = -1;
            const reported = root.activePlayer?.position ?? 0;
            root._reanchor(reported);
            root._publishedTrackKey = root._latestTrackKey;
            root.status = "ok";
        });
    }

    onActiveChanged: root.scheduleRefresh()
    onActivePlayerChanged: root.scheduleRefresh()

    Timer {
        id: refreshTimer
        interval: 350
        repeat: false
        onTriggered: root._queueCurrentTrack()
    }

    property real _anchorPos: 0
    property real _anchorMs: 0
    property real _lastReported: -1

    readonly property bool _playing: root.activePlayer?.isPlaying ?? false

    function _reanchor(pos: real): void {
        root._anchorPos = pos;
        root._anchorMs = Date.now();
    }

    function _estimatedPosition(): real {
        if (!root._playing)
            return root._anchorPos;
        return root._anchorPos + (Date.now() - root._anchorMs) / 1000;
    }

    on_PlayingChanged: root._reanchor(root._estimatedPosition())

    Timer {
        id: syncTimer
        interval: 50
        repeat: true
        running: root.active && root.status === "ok" && root.lyricsLines.length > 0
        onTriggered: {
            if (Date.now() < root._ignoreReportedUntil) {
                const pos = root._estimatedPosition();
                root.currentPosition = pos;
                let idx = -1;
                for (let i = 0; i < root.lyricsLines.length; i++) {
                    if (root.lyricsLines[i].time <= pos)
                        idx = i;
                    else
                        break;
                }
                if (idx !== root.activeIndex) {
                    root.activeIndex = idx;
                    root.slots = root.buildSlots(idx);
                }
                return;
            }

            const reported = root.activePlayer?.position ?? 0;
            if (reported !== root._lastReported) {
                root._lastReported = reported;
                if (reported > 0 || root._anchorPos === 0)
                    root._reanchor(reported);
            }

            const pos = root._estimatedPosition();
            root.currentPosition = pos;
            let idx = -1;
            for (let i = 0; i < root.lyricsLines.length; i++) {
                if (root.lyricsLines[i].time <= pos)
                    idx = i;
                else
                    break;
            }
            if (idx !== root.activeIndex) {
                root.activeIndex = idx;
                root.slots = root.buildSlots(idx);
            }
        }
    }

    Process {
        id: lyricsProc
        running: false

        stdout: StdioCollector {
            id: lyricsCollector
            onStreamFinished: {
                const raw = lyricsCollector.text.trim();
                if (raw.length === 0)
                    return;

                let payload;
                try {
                    payload = JSON.parse(raw);
                } catch (e) {
                    if (root._runningRequestId !== ""
                            && root._runningRequestId === root._latestRequestId)
                        console.log("LyricsService: unparseable output from lyrics.py:", raw.slice(0, 200));
                    return;
                }

                const requestId = String(payload.requestId ?? "");
                if (requestId === "" || requestId !== root._runningRequestId
                        || requestId !== root._latestRequestId)
                    return;

                if (payload.status !== "ok") {
                    if (payload.status === "error")
                        console.log("LyricsService:", payload.message ?? "unknown error");
                    root._publishFailure(requestId,
                        payload.status === "no_info" ? "no_info"
                            : payload.status === "error" ? "error" : "not_found");
                    return;
                }

                const lines = (payload.lines ?? [])
                    .filter(line => typeof line.t === "number")
                    .map(line => ({
                        time: line.t,
                        text: line.text ?? "",
                        // Present only for word-level providers; the renderer
                        // falls back to its own estimate when absent.
                        words: Array.isArray(line.words) ? line.words : null
                    }));

                if (lines.length === 0) {
                    root._publishFailure(requestId, "not_found");
                    return;
                }

                root._publishSuccess(requestId, lines);
            }
        }

        onExited: {
            root._runningRequestId = "";
            root._runningTrackKey = "";
            Qt.callLater(root._startPendingRequest);
        }
    }

    Connections {
        target: root.activePlayer

        function onPostTrackChanged(): void {
            root.scheduleRefresh();
        }

        function onTrackTitleChanged(): void {
            root.scheduleRefresh();
        }

        function onTrackArtistChanged(): void {
            root.scheduleRefresh();
        }

        function onTrackAlbumChanged(): void {
            root.scheduleRefresh();
        }

        function onLengthChanged(): void {
            root.scheduleRefresh();
        }
    }
}
