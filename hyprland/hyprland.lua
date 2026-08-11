------------------
-- MONITORS
------------------

hl.monitor({
    output = "DP-3",
    mode = "1920x1080@60",
    position = "0x0",
    scale = 1,
})

hl.monitor({
    output = "HDMI-A-1",
    mode = "1920x1080@60",
    position = "1920x0",
    scale = 1,
})

hl.monitor({
    output = "DP-2",
    mode = "1920x1080@60",
    position = "3840x0",
    scale = 1,
})


------------------
-- ENVIRONMENT
------------------

hl.env("GTK_THEME", "Sweet-Nova:dark")
hl.env("QT_QPA_PLATFORM", "wayland")
hl.env("QT_QPA_PLATFORMTHEME", "qt5ct")

-- Weather dashboard
hl.env("WEATHER_API_KEY", "<key>")
hl.env("WEATHER_LOCATION", "London,uk")
hl.env("WEATHER_UNITS", "metric")


------------------
-- AUTOSTART
------------------

hl.on("hyprland.start", function()

    -- Clean up a stale dashboard instance from a previous session.
    hl.exec_cmd([[
        systemctl --user kill \
            --kill-whom=all \
            --signal=KILL \
            hypr-dashboard.scope \
            >/dev/null 2>&1 || true

        systemctl --user stop \
            hypr-dashboard.scope \
            >/dev/null 2>&1 || true
    ]])


    -- Wallpaper
    hl.exec_cmd("awww-daemon")
    hl.exec_cmd("awww img /usr/share/backgrounds/desk.jpg")


    -- Clipboard
    hl.exec_cmd(
        "wl-paste --type text --watch cliphist store"
    )

    hl.exec_cmd(
        "wl-paste --type image --watch cliphist store"
    )

    hl.exec_cmd(
        "cliphist wipe"
    )


    -- SSHFS
    hl.exec_cmd(
        "sshfs paolo@192.168.1.111:/home/paolo/Desktop /home/paolo/server"
    )


    -- Disk
    hl.exec_cmd(
        "mount /dev/sda2 /home/paolo/Desktop/sda2"
    )


    -- DBus / systemd environment
    hl.exec_cmd(
        "dbus-update-activation-environment --systemd WAYLAND_DISPLAY XDG_CURRENT_DESKTOP HYPRLAND_INSTANCE_SIGNATURE"
    )


    -- split-monitor-workspaces workaround:
    -- remap after all three monitors are ready.
    hl.exec_cmd(
        "sleep 2 && hyprctl reload"
    )
end)


------------------
-- INPUT
------------------

hl.config({
    input = {
        kb_layout = "ch",
        kb_variant = "fr",
        kb_model = "",
        kb_options = "",
        kb_rules = "",

        follow_mouse = 2,
        float_switch_override_focus = 0,

        sensitivity = 0,

        touchpad = {
            natural_scroll = false,
        },
    },
})


------------------
-- GENERAL
------------------

hl.config({
    general = {
        gaps_in = 5,
        gaps_out = 5,

        border_size = 0,

        col = {
            active_border = "rgba(66888888)",
            inactive_border = "rgba(595959aa)",
        },

        layout = "dwindle",
    },
})


------------------
-- DECORATION
------------------

hl.config({
    decoration = {
        rounding = 7,

        blur = {
            enabled = true,
            size = 3,
            passes = 1,
        },
    },
})


------------------
-- BINDS BEHAVIOR
------------------

hl.config({
    binds = {
        -- Applications must never be allowed to disable
        -- Hyprland global keybinds.
        disable_keybind_grabbing = true,
    },
})


------------------
-- WINDOW RULES
------------------

-- Opacity for Alacritty and VS Code
hl.window_rule({
    match = {
        class = "^(Alacritty|Code)$",
    },

    opacity = "0.9 0.9",
})


-- Float Alacritty, Latex-rec and swayimg
hl.window_rule({
    match = {
        class = "^(Alacritty|Latex-rec|swayimg.*)$",
    },

    float = true,
})


------------------
-- ANIMATIONS
------------------

hl.config({
    animations = {
        enabled = true,
    },
})


hl.animation({
    leaf = "windowsIn",
    enabled = true,
    speed = 4,
    bezier = "default",
})


hl.animation({
    leaf = "windowsMove",
    enabled = true,
    speed = 4,
    bezier = "default",
})


hl.animation({
    leaf = "windowsOut",
    enabled = true,
    speed = 4,
    bezier = "default",
    style = "popin 80%",
})


hl.animation({
    leaf = "border",
    enabled = false,
})


hl.animation({
    leaf = "borderangle",
    enabled = false,
})


hl.animation({
    leaf = "fade",
    enabled = true,
    speed = 7,
    bezier = "default",
})


hl.animation({
    leaf = "workspaces",
    enabled = false,
})


------------------
-- DWINDLE
------------------

hl.config({
    dwindle = {
        preserve_split = true,
    },
})


---------------------------
-- SPLIT MONITOR WORKSPACES
---------------------------

package.path = package.path .. ";./?.lua;./?/init.lua"

local smw = require("plugins.split-monitor-workspaces")


smw.setup({
    -- 10 workspaces for every monitor.
    workspace_count = 10,

    -- Fixed monitor ordering.
    monitor_priority = {
        "DP-3",
        "HDMI-A-1",
        "DP-2",
    },

    -- Recommended when the config is reloaded.
    keep_focused = true,

    -- Keep empty workspaces alive.
    enable_persistent_workspaces = true,

    enable_notifications = false,
})


------------------
-- VARIABLES
------------------

local mainMod = "SUPER"

local DASHBOARD_WORKSPACE = "Dashboard"
local DASHBOARD_UNIT = "hypr-dashboard.scope"


------------------
-- DASHBOARD
------------------

local function open_dashboard()

    -- First move to the dedicated empty workspace.
    hl.dispatch(
        hl.dsp.focus({
            workspace = "name:" .. DASHBOARD_WORKSPACE,
        })
    )


    -- Wait until the workspace switch has happened.
    hl.timer(function()

        -- dashboard is launched inside its own systemd scope.
        --
        -- Any process forked by dashboard remains inside this
        -- cgroup and can therefore be killed as one group.
        hl.exec_cmd([[
            systemctl --user kill \
                --kill-whom=all \
                --signal=KILL \
                hypr-dashboard.scope \
                >/dev/null 2>&1 || true

            systemctl --user stop \
                hypr-dashboard.scope \
                >/dev/null 2>&1 || true

            systemd-run \
                --user \
                --scope \
                --quiet \
                --collect \
                --unit=hypr-dashboard \
                --property=KillMode=control-group \
                /bin/bash -lc 'exec dashboard'
        ]])

    end, {
        timeout = 150,
        type = "oneshot",
    })
end


local function close_dashboard()

    -- IMPORTANT:
    --
    -- Kill the ENTIRE dashboard cgroup first.
    --
    -- dashboard creates a layer surface, so merely changing
    -- workspace is not sufficient. The layer has to die.
    --
    -- SIGKILL is intentional here: the emergency path must
    -- never wait for an application that has grabbed input.

    hl.exec_cmd([[
        systemctl --user kill \
            --kill-whom=all \
            --signal=KILL \
            hypr-dashboard.scope \
            >/dev/null 2>&1 || true

        systemctl --user stop \
            hypr-dashboard.scope \
            >/dev/null 2>&1 || true

        hyprctl dispatch \
            'hl.dsp.focus({ workspace = "previous_per_monitor" })'
    ]])
end


local function toggle_dashboard()

    local workspace = hl.get_active_workspace()

    if workspace == nil then
        return
    end


    if workspace.name == DASHBOARD_WORKSPACE then
        close_dashboard()
        return
    end


    open_dashboard()
end


------------------
-- EXIT HYPRLAND
------------------

hl.bind(
    mainMod .. " + Escape",

    hl.dsp.exec_cmd(
        "command -v hyprshutdown >/dev/null 2>&1 && hyprshutdown || hyprctl dispatch 'hl.dsp.exit()'"
    )
)


------------------
-- CLOSE WINDOW
------------------

hl.bind(
    mainMod .. " + Q",
    hl.dsp.window.close()
)


------------------
-- DASHBOARD BIND
------------------

hl.bind(
    mainMod .. " + D",

    toggle_dashboard,

    {
        -- Works through input inhibitors.
        locked = true,

        -- Ignore application shortcut-inhibition requests.
        bypass = true,

        -- Cannot be shadowed by another bind.
        transparent = true,

        -- Available from every submap.
        submap_universal = true,

        description = "Toggle dashboard",
    }
)

------------------
-- PROGRAMS
------------------

hl.bind(
    mainMod .. " + Home",

    hl.dsp.exec_cmd(
        "alacritty -o font.size=13"
    )
)


hl.bind(
    mainMod .. " + E",

    hl.dsp.exec_cmd(
        "nautilus"
    )
)


hl.bind(
    mainMod .. " + Return",

    hl.dsp.exec_cmd(
        "rofi -show drun"
    )
)


hl.bind(
    mainMod .. " + U",

    hl.dsp.exec_cmd(
        "latex-rec"
    )
)


hl.bind(
    mainMod .. " + L",

    hl.dsp.exec_cmd(
        "cliphist list | rofi -dmenu | cliphist decode | wl-copy"
    )
)


------------------
-- WINDOW CONTROL
------------------

hl.bind(
    mainMod .. " + V",

    hl.dsp.window.float({
        action = "toggle",
    })
)


hl.bind(
    mainMod .. " + F",

    hl.dsp.window.fullscreen({
        mode = "fullscreen",
        action = "toggle",
    })
)


------------------
-- SCREENSHOTS
------------------

hl.bind(
    "Print",

    hl.dsp.exec_cmd(
        "take_screenshot"
    )
)


hl.bind(
    mainMod .. " + Print",

    hl.dsp.exec_cmd(
        'swayimg "$HOME/Screenshots/latest.png"'
    )
)


------------------
-- WARFRAME
------------------

hl.bind(
    "CONTROL + B",

    hl.dsp.exec_cmd(
        "/usr/local/bin/warframe_relics"
    )
)


------------------
-- FOCUS
------------------

hl.bind(
    mainMod .. " + left",

    hl.dsp.focus({
        direction = "left",
    })
)


hl.bind(
    mainMod .. " + right",

    hl.dsp.focus({
        direction = "right",
    })
)


hl.bind(
    mainMod .. " + up",

    hl.dsp.focus({
        direction = "up",
    })
)


hl.bind(
    mainMod .. " + down",

    hl.dsp.focus({
        direction = "down",
    })
)


---------------------------
-- SPLIT WORKSPACE KEYBINDS
---------------------------

-- SUPER + 1 ... 9
-- SUPER + 0 = workspace 10
--
-- The selected workspace is relative to the currently
-- focused monitor.

for i = 1, smw.get_amount_of_workspaces() do

    local key = tostring(i)

    if i == 10 then
        key = "0"
    end


    -- Switch workspace
    hl.bind(
        mainMod .. " + " .. key,

        smw.workspace(
            tostring(i)
        )
    )


    -- Move active window silently
    hl.bind(
        mainMod .. " + SHIFT + " .. key,

        smw.move_to_workspace_silent(
            tostring(i)
        )
    )
end


------------------
-- MOUSE
------------------

-- SUPER + left mouse button:
-- move window

hl.bind(
    mainMod .. " + mouse:272",

    hl.dsp.window.drag(),

    {
        mouse = true,
    }
)


-- SUPER + right mouse button:
-- resize window

hl.bind(
    mainMod .. " + mouse:273",

    hl.dsp.window.resize(),

    {
        mouse = true,
    }
)