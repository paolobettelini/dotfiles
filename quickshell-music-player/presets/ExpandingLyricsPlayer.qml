pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Layouts
import Qt5Compat.GraphicalEffects as GE
import Quickshell.Services.Mpris
import "../common"
import "../common"
import "../common"
import "../common"
import "../services"
import "../components"
import "../components/lyricsProfiles.js" as LyricsProfiles

Item {
    id: root
    property MprisPlayer player: null
    property list<real> visualizerPoints: []
    property real radius: Appearance.zzzEverywhere ? Appearance.zzz.panelRadius : Appearance.angelEverywhere ? Appearance.angel.roundingNormal : Appearance.rounding.verylarge
    property real screenX: 0
    property real screenY: 0

    readonly property bool isBrowserOrVideo: MprisController._isBrowserPlayer(root.player) || MprisController._isBrowserYoutubePlayer(root.player)
    readonly property bool canHaveLyrics: true
    readonly property bool lyricsExpanded: root.canHaveLyrics && Config.getNestedValue("background.widgets.mediaControls.lyricsExpanded", false)
    readonly property real headerHeight: root.lyricsExpanded
        ? 90
        : card.height
    readonly property real buttonSize: 32
    readonly property real buttonIconSize: 18

    readonly property string vizType: Config.getNestedValue("background.widgets.mediaControls.visualizerType", "wave")
    readonly property string vizPosition: Config.getNestedValue("background.widgets.mediaControls.visualizerPosition", "bottom")

    PlayerBase {
        id: playerBase
        player: root.player
    }

    property color themeSourceColor: playerBase.artDominantColor
    property QtObject blendedColors: AdaptedMaterialScheme {
        color: root.themeSourceColor
    }

    readonly property color surfaceColor: Appearance.zzzEverywhere ? Appearance.zzz.paper
        : Appearance.inirEverywhere ? playerBase.inirLayer1
        : Appearance.auroraEverywhere ? "transparent"
        : (root.blendedColors?.colLayer0 ?? Appearance.colors.colLayer0)
    readonly property color ink: Appearance.zzzEverywhere ? Appearance.zzz.ink
        : Appearance.inirEverywhere ? playerBase.inirText
        : (root.blendedColors?.colOnLayer0 ?? Appearance.colors.colOnLayer0)
    readonly property color accent: Appearance.zzzEverywhere ? Appearance.zzz.accent
        : Appearance.inirEverywhere ? playerBase.inirPrimary
        : (root.blendedColors?.colPrimary ?? Appearance.colors.colPrimary)
    readonly property color onAccent: Appearance.zzzEverywhere ? Appearance.zzz.onSticker
        : Appearance.inirEverywhere ? playerBase.inirOnPrimary
        : (root.blendedColors?.colOnPrimary ?? Appearance.colors.colOnPrimary)

    StyledRectangularShadow {
        target: card
        visible: !Appearance.zzzEverywhere && (Appearance.angelEverywhere || (!Appearance.inirEverywhere && !Appearance.auroraEverywhere))
    }

    Rectangle {
        id: card
        anchors.centerIn: parent
        width: parent.width - Appearance.sizes.elevationMargin
        height: parent.height - Appearance.sizes.elevationMargin
        radius: Appearance.zzzEverywhere ? Appearance.zzz.panelRadius : Appearance.inirEverywhere ? Appearance.inir.roundingNormal : root.radius
        color: "transparent"
        border.width: Appearance.zzzEverywhere ? Appearance.zzz.borderThick : 1
        border.color: Appearance.zzzEverywhere ? Appearance.zzz.hairlineStrong
            : Appearance.inirEverywhere ? Appearance.inir.colBorder
            : Qt.rgba(root.ink.r, root.ink.g, root.ink.b, 0.14)
        clip: true

        Behavior on color {
            enabled: Appearance.animationsEnabled
            ColorAnimation {
                duration: Appearance.animation.elementMoveFast.duration
                easing.type: Appearance.animation.elementMoveFast.type
                easing.bezierCurve: Appearance.animation.elementMoveFast.bezierCurve
            }
        }

        layer.enabled: false
        layer.effect: GE.OpacityMask {
            maskSource: Rectangle {
                width: card.width
                height: card.height
                radius: card.radius
            }
        }

        // Background cover art wash
        Item {
            visible: false
            anchors.fill: parent

            StyledImage {
                anchors.fill: parent
                source: playerBase.displayedArtFilePath
                fillMode: Image.PreserveAspectCrop
                cache: false
                antialiasing: true
                smooth: true
                sourceSize.width: 72
                sourceSize.height: 72
                opacity: 0.20
            }

            Rectangle {
                anchors.fill: parent
                color: card.color
                opacity: 0.60
            }
        }

        Rectangle {
            anchors.fill: parent
            visible: false
            gradient: Gradient {
                orientation: Gradient.Horizontal
                GradientStop { position: 0.0; color: ColorUtils.transparentize(root.surfaceColor, 0.65) }
                GradientStop { position: 0.4; color: ColorUtils.transparentize(root.surfaceColor, 0.45) }
                GradientStop { position: 1.0; color: ColorUtils.transparentize(root.surfaceColor, 0.25) }
            }
        }

        // Visualizer overlay at the bottom of the card
        WaveVisualizer {
            visible: root.vizType === "wave" && root.vizPosition !== "none"
            anchors { left: parent.left; right: parent.right }
            y: parent.height - height
            height: 14
            live: playerBase.effectiveIsPlaying
            points: root.visualizerPoints
            maxVisualizerValue: 1000
            smoothing: 3
            opacity: 0.25
            color: ColorUtils.transparentize(root.accent, 0.6)
        }
        CavaVisualizer {
            visible: root.vizType === "bars" && root.vizPosition !== "none"
            anchors { left: parent.left; right: parent.right }
            y: parent.height - height
            height: 38
            live: playerBase.effectiveIsPlaying
            points: root.visualizerPoints
            maxVisualizerValue: 1000
            smoothing: 2
            barCount: 32
            barSpacing: 2
            barRadius: 2
            barMinHeight: 1
            colorLow: ColorUtils.transparentize(root.accent, 0.45)
            colorMed: ColorUtils.transparentize(root.accent, 0.25)
            colorHigh: root.accent
        }

        ZzzGraphicPlate {
            visible: false
            anchors.fill: parent
            accentColor: root.accent
        }

        Column {
            anchors.fill: parent
            spacing: 0

            Item {
                id: headerItem
                width: parent.width
                height: root.headerHeight

                Behavior on height {
                    enabled: Appearance.animationsEnabled
                    NumberAnimation {
                        duration: Appearance.animation.elementResize.duration
                        easing.type: Appearance.animation.elementResize.type
                        easing.bezierCurve: Appearance.animation.elementResize.bezierCurve
                    }
                }

                Item {
                    id: artRect
                    anchors {
                        left: parent.left
                        top: parent.top
                        bottom: parent.bottom
                        leftMargin: root.lyricsExpanded ? 10 : 12
                        topMargin: root.lyricsExpanded ? 10 : 12
                        bottomMargin: root.lyricsExpanded ? 10 : 12
                    }
                    width: height

                    Behavior on anchors.leftMargin {
                        enabled: Appearance.animationsEnabled
                        NumberAnimation {
                            duration: Appearance.animation.elementResize.duration
                            easing.type: Appearance.animation.elementResize.type
                            easing.bezierCurve: Appearance.animation.elementResize.bezierCurve
                        }
                    }
                    Behavior on anchors.topMargin {
                        enabled: Appearance.animationsEnabled
                        NumberAnimation {
                            duration: Appearance.animation.elementResize.duration
                            easing.type: Appearance.animation.elementResize.type
                            easing.bezierCurve: Appearance.animation.elementResize.bezierCurve
                        }
                    }
                    Behavior on anchors.bottomMargin {
                        enabled: Appearance.animationsEnabled
                        NumberAnimation {
                            duration: Appearance.animation.elementResize.duration
                            easing.type: Appearance.animation.elementResize.type
                            easing.bezierCurve: Appearance.animation.elementResize.bezierCurve
                        }
                    }

                    // Interactive micro-animations on hover, click, and view change
                    scale: artMouseArea.pressed ? 0.94 : (artMouseArea.containsMouse ? 1.04 : 1.0)
                    transformOrigin: Item.Center

                    Behavior on scale {
                        enabled: Appearance.animationsEnabled
                        NumberAnimation {
                            duration: 220
                            easing.type: Easing.OutBack
                            easing.overshoot: 1.2
                        }
                    }

                    readonly property real currentRadius: root.lyricsExpanded
                        ? Appearance.rounding.small
                        : (Appearance.rounding.small + 2)

                    Rectangle {
                        anchors.fill: parent
                        radius: artRect.currentRadius
                        color: Appearance.zzzEverywhere ? Appearance.zzz.paperAlt : Appearance.inirEverywhere ? playerBase.inirLayer2 : (root.blendedColors?.colLayer1 ?? Appearance.colors.colLayer1)

                        Behavior on radius {
                            enabled: Appearance.animationsEnabled
                            NumberAnimation { duration: 250; easing.type: Easing.OutCubic }
                        }

                        MaterialSymbol {
                            anchors.centerIn: parent
                            text: "music_note"
                            fill: 1
                            iconSize: Math.round(artRect.height / 3)
                            color: ColorUtils.applyAlpha(root.ink, 0.6)
                            visible: playerBase.displayedArtFilePath === ""
                        }
                    }

                    Rectangle {
                        id: artMask
                        anchors.fill: parent
                        radius: artRect.currentRadius
                        visible: false
                        Behavior on radius {
                            enabled: Appearance.animationsEnabled
                            NumberAnimation { duration: 250; easing.type: Easing.OutCubic }
                        }
                    }

                    StyledImage {
                        anchors.fill: parent
                        source: playerBase.displayedArtFilePath
                        fillMode: Image.PreserveAspectCrop
                        cache: true
                        antialiasing: true
                        smooth: true
                        mipmap: true
                        sourceSize.width: 300
                        sourceSize.height: 300
                        visible: playerBase.displayedArtFilePath !== ""
                        layer.enabled: true
                        layer.effect: GE.OpacityMask {
                            maskSource: artMask
                        }
                    }

                    MouseArea {
                        id: artMouseArea
                        anchors.fill: parent
                        hoverEnabled: root.canHaveLyrics
                        enabled: root.canHaveLyrics
                        cursorShape: root.canHaveLyrics ? Qt.PointingHandCursor : Qt.ArrowCursor
                        onClicked: {
                            if (root.canHaveLyrics) {
                                Config.setNestedValue("background.widgets.mediaControls.lyricsExpanded", !root.lyricsExpanded);
                            }
                        }
                    }
                }

                // Header content right of artRect
                Item {
                    anchors {
                        left: artRect.right
                        right: parent.right
                        top: parent.top
                        bottom: parent.bottom
                        leftMargin: root.lyricsExpanded ? 10 : 14
                        rightMargin: root.lyricsExpanded ? 10 : 14
                        topMargin: root.lyricsExpanded ? 6 : 10
                        bottomMargin: root.lyricsExpanded ? 6 : 10
                    }

                    ColumnLayout {
                        anchors.fill: parent
                        spacing: root.lyricsExpanded ? 3 : 6

                        RowLayout {
                            Layout.fillWidth: true
                            Layout.fillHeight: true
                            spacing: 8

                            ColumnLayout {
                                Layout.fillWidth: true
                                Layout.alignment: Qt.AlignVCenter
                                spacing: 2

                                StyledText {
                                    Layout.fillWidth: true
                                    text: StringUtils.cleanMusicTitle(playerBase.effectiveTitle) || Translation.tr("Something")
                                    font.pixelSize: Appearance.font.pixelSize.normal
                                    font.weight: Font.DemiBold
                                    color: root.ink
                                    elide: Text.ElideRight
                                }

                                StyledText {
                                    Layout.fillWidth: true
                                    text: playerBase.effectiveArtist || Translation.tr("Play")
                                    font.pixelSize: Appearance.font.pixelSize.small
                                    color: ColorUtils.applyAlpha(root.ink, 0.65)
                                    elide: Text.ElideRight
                                }
                            }

                            // Controls Pill (Aligned to top-right)
                            Rectangle {
                                id: controlsPill
                                Layout.alignment: Qt.AlignVCenter | Qt.AlignRight
                                implicitWidth: controlsRow.implicitWidth + 10
                                implicitHeight: 40
                                radius: Appearance.rounding.full
                                color: ColorUtils.transparentize(root.surfaceColor, 0.4)
                                border.width: 1
                                border.color: ColorUtils.transparentize(root.ink, 0.85)

                                RowLayout {
                                    id: controlsRow
                                    anchors.centerIn: parent
                                    spacing: 4

                                    RippleButton {
                                        id: lyricsBtn
                                        visible: root.canHaveLyrics
                                        implicitWidth: 32
                                        implicitHeight: 32
                                        buttonRadius: Appearance.rounding.full
                                        colBackground: root.lyricsExpanded ? root.accent : "transparent"
                                        colBackgroundHover: ColorUtils.transparentize(root.ink, 0.85)
                                        colRipple: ColorUtils.transparentize(root.accent, 0.5)
                                        // One click opens or closes the lyrics; a quick second click
                                        // instead moves to the next lyrics theme.
                                        onClicked: {
                                            if (doubleClickWindow.running) {
                                                doubleClickWindow.stop();
                                                const order = LyricsProfiles.order;
                                                const current = order.indexOf(Config.options?.media?.lyricsStyle ?? "default");
                                                Config.setNestedValue("media.lyricsStyle", order[(current + 1) % order.length]);
                                            } else {
                                                doubleClickWindow.start();
                                            }
                                        }
                                        Timer {
                                            id: doubleClickWindow
                                            interval: 260
                                            onTriggered: Config.setNestedValue("background.widgets.mediaControls.lyricsExpanded", !root.lyricsExpanded)
                                        }

                                        scale: lyricsBtn.pressed ? 0.88 : (lyricsBtn.hovered ? 1.08 : 1.0)
                                        Behavior on scale {
                                            enabled: Appearance.animationsEnabled
                                            NumberAnimation { duration: 180; easing.type: Easing.OutBack; easing.overshoot: 1.2 }
                                        }

                                        contentItem: MaterialSymbol {
                                            anchors.centerIn: parent
                                            text: "lyrics"
                                            iconSize: 18
                                            fill: root.lyricsExpanded ? 1 : 0
                                            color: root.lyricsExpanded ? root.onAccent : root.ink
                                        }
                                    }

                                    RippleButton {
                                        id: prevBtn
                                        implicitWidth: 32
                                        implicitHeight: 32
                                        buttonRadius: Appearance.rounding.full
                                        colBackground: "transparent"
                                        colBackgroundHover: ColorUtils.transparentize(root.ink, 0.85)
                                        colRipple: ColorUtils.transparentize(root.accent, 0.5)
                                        onClicked: playerBase.previous()

                                        scale: prevBtn.pressed ? 0.88 : (prevBtn.hovered ? 1.08 : 1.0)
                                        Behavior on scale {
                                            enabled: Appearance.animationsEnabled
                                            NumberAnimation { duration: 180; easing.type: Easing.OutBack; easing.overshoot: 1.2 }
                                        }

                                        contentItem: MaterialSymbol {
                                            anchors.centerIn: parent
                                            text: "skip_previous"
                                            iconSize: 19
                                            fill: 1
                                            color: root.ink
                                        }
                                    }

                                    MaterialShapeWrappedMaterialSymbol {
                                        id: playShape
                                        shape: MaterialShape.Shape.Circle
                                        color: root.accent
                                        colSymbol: root.onAccent
                                        text: playerBase.effectiveIsPlaying ? "pause" : "play_arrow"
                                        iconSize: 19
                                        fill: 1
                                        padding: 8

                                        scale: playArea.pressed ? 0.88 : (playArea.containsMouse ? 1.08 : 1.0)
                                        Behavior on scale {
                                            enabled: Appearance.animationsEnabled
                                            NumberAnimation { duration: 200; easing.type: Easing.OutBack; easing.overshoot: 1.25 }
                                        }

                                        MouseArea {
                                            id: playArea
                                            anchors.fill: parent
                                            hoverEnabled: true
                                            cursorShape: Qt.PointingHandCursor
                                            onClicked: playerBase.togglePlaying()
                                        }
                                    }

                                    RippleButton {
                                        id: nextBtn
                                        implicitWidth: 32
                                        implicitHeight: 32
                                        buttonRadius: Appearance.rounding.full
                                        colBackground: "transparent"
                                        colBackgroundHover: ColorUtils.transparentize(root.ink, 0.85)
                                        colRipple: ColorUtils.transparentize(root.accent, 0.5)
                                        onClicked: playerBase.next()

                                        scale: nextBtn.pressed ? 0.88 : (nextBtn.hovered ? 1.08 : 1.0)
                                        Behavior on scale {
                                            enabled: Appearance.animationsEnabled
                                            NumberAnimation { duration: 180; easing.type: Easing.OutBack; easing.overshoot: 1.2 }
                                        }

                                        contentItem: MaterialSymbol {
                                            anchors.centerIn: parent
                                            text: "skip_next"
                                            iconSize: 19
                                            fill: 1
                                            color: root.ink
                                        }
                                    }
                                }
                            }
                        }

                        // Progress bar row
                        RowLayout {
                            Layout.fillWidth: true
                            spacing: 6

                            PlayerProgress {
                                id: headerProgress
                                Layout.fillWidth: true
                                implicitHeight: 10
                                position: playerBase.effectivePosition
                                length: playerBase.effectiveLength
                                canSeek: playerBase.effectiveCanSeek
                                isPlaying: playerBase.effectiveIsPlaying
                                highlightColor: root.accent
                                trackColor: ColorUtils.transparentize(root.ink, 0.8)
                                onSeekRequested: seconds => playerBase.seek(seconds)
                            }

                            StyledText {
                                text: StringUtils.friendlyTimeForSeconds(headerProgress.previewPosition)
                                font.pixelSize: Appearance.font.pixelSize.smallest
                                font.family: Appearance.font.family.numbers
                                color: ColorUtils.applyAlpha(root.ink, 0.7)
                                visible: playerBase.effectiveLength > 0
                            }
                        }
                    }
                }
            }

            Item {
                width: parent.width
                height: root.lyricsExpanded ? 2 : 0
                visible: root.lyricsExpanded

                Rectangle {
                    anchors.centerIn: parent
                    width: parent.width - 48
                    height: 1
                    opacity: 0.15
                    gradient: Gradient {
                        orientation: Gradient.Horizontal
                        GradientStop { position: 0.0; color: "transparent" }
                        GradientStop { position: 0.2; color: root.ink }
                        GradientStop { position: 0.8; color: root.ink }
                        GradientStop { position: 1.0; color: "transparent" }
                    }
                }
            }

            Item {
                width: parent.width
                height: root.lyricsExpanded ? Math.max(0, card.height - root.headerHeight - 2) : 0
                visible: root.lyricsExpanded

                PlayerLyrics {
                    id: lyricSheet
                    anchors.fill: parent
                    // The lyric-edit view uses the whole lower card; the card's
                    // rounded mask clips it, so it needs no inset of its own.
                    anchors.leftMargin: lyricSheet.kinetic ? 0 : 18
                    anchors.rightMargin: lyricSheet.kinetic ? 0 : 18
                    anchors.topMargin: lyricSheet.kinetic ? 0 : 10
                    anchors.bottomMargin: lyricSheet.kinetic ? 0 : 14
                    showPlaceholder: true
                    textAlignment: Text.AlignHCenter
                    baseSize: Appearance.font.pixelSize.normal
                    activeScale: 1.18
                    lineSpacing: 10
                    activeColor: root.ink
                    textColor: ColorUtils.applyAlpha(root.ink, 0.6)
                    indicatorColor: root.accent
                }
            }
        }
    }
}
