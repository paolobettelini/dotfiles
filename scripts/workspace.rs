// Usage: workspace <command> <workspace>
// E.g. workspace workspace 1
//      workspace movetoworkspace 3
//      workspace fix

// How it works:
// Depending on the monitors IDs (ID-0, ID-1, ID-2),
// when you want to move to a (split) workspace W,
// the actual workspace is (W-1) * monitors + ID + 1
// However, the default workspace must match the monitor IDs
// E.g
//      ID-0 -> 1
//      ID-1 -> 2
//      ID-2 -> 3
// This should be the default. If you changed them and want them like that
// edit the following variable with your
// monitor-default workspace displacements
const DISPLACEMENTS: [usize; 3] = [1, 2, 3];
// If you don't care you can call
// the command `workspace fix`, which moves
// the workspaces to their correct monitors.
// This is also useful if the workspaces-monitors get messed up.

// TODO ora si può usare "hyprctl activeworkspace" per ottenere il monitor corrente

use std::{
    env::args,
    process::Command,
    str::{from_utf8, FromStr},
};

#[derive(Debug)]
struct Monitor {
    id: usize,
    width: usize,
    height: usize,
    pos_x: i32,
    pos_y: i32,
}

enum Parameters {
    FixWorkspaces,
    DispatchWorkspaceCmd(DispatchParameters),
}

struct DispatchParameters {
    command: String,
    workspace: usize,
}

fn main() {
    let args = Parameters::parse();

    let monitors = get_monitors();

    match args {
        Parameters::FixWorkspaces => handle_fix_cmd(&monitors),
        Parameters::DispatchWorkspaceCmd(cmd) => handle_workspace_cmd(&monitors, &cmd),
    };
}

impl Parameters {
    fn parse() -> Parameters {
        let args: Vec<String> = args().collect();

        let arg1 = args.get(1).unwrap();

        if arg1 == "fix" {
            return Parameters::FixWorkspaces;
        }

        // Read values
        let command = (&arg1).to_string();
        let workspace: usize = args.get(2).unwrap().parse::<usize>().unwrap();

        Parameters::DispatchWorkspaceCmd(DispatchParameters { command, workspace })
    }
}

fn handle_workspace_cmd(monitors: &Vec<Monitor>, cmd: &DispatchParameters) {
    let cursor_pos = get_cursor_position();
    let current_monitor = get_current_monitor(monitors, cursor_pos).unwrap();
    let actual_workspace = (cmd.workspace - 1) * monitors.len() + DISPLACEMENTS[current_monitor.id];

    dispatch_workspace_cmd(&cmd.command, actual_workspace);
}

fn handle_fix_cmd(monitors: &Vec<Monitor>) {
    let workspace_ids = get_workspaces_ids();
    let monitors = monitors.len();

    for workspace in workspace_ids {
        // Compute correct monitor.
        let monitor = ((workspace as usize % monitors) + monitors - 1) % monitors;
        let monitor = DISPLACEMENTS[monitor] - 1;
        dispatch_moveworkspacetomonitor_cmd(&workspace.to_string(), &monitor.to_string())
    }
}

// Dispatches "hyprctl dispatch <cmd> <workspace>"
fn dispatch_workspace_cmd(cmd: &str, workspace: usize) {
    // Dispatch command
    Command::new("hyprctl")
        .arg("dispatch")
        .arg(cmd)
        .arg(&workspace.to_string())
        .output()
        .unwrap();
}

// Dispatches "hyprctl dispatch <cmd> <workspace>"
fn dispatch_moveworkspacetomonitor_cmd(workspace: &str, monitor: &str) {
    // Dispatch command
    Command::new("hyprctl")
        .arg("dispatch")
        .arg("moveworkspacetomonitor")
        .arg(workspace)
        .arg(monitor)
        .output()
        .unwrap();
}

fn get_current_monitor(monitors: &[Monitor], cursor_pos: (i32, i32)) -> Option<&Monitor> {
    monitors.iter().find(|&monitor| {
        cursor_pos.0 >= monitor.pos_x
            && cursor_pos.1 >= monitor.pos_y
            && cursor_pos.0 < monitor.pos_x + monitor.width as i32
            && cursor_pos.1 < monitor.pos_y + monitor.height as i32
    })
}

fn get_monitors() -> Vec<Monitor> {
    // Call "hyprctl monitors"
    let raw = Command::new("hyprctl")
        .arg("monitors")
        .output()
        .unwrap()
        .stdout;

    let input = from_utf8(&raw).unwrap();

    // Extract values from output
    let mut monitors = vec![];
    let mut current_id: i32 = -1;
    for line in input.lines() {
        if line.starts_with("Monitor") {
            current_id += 1;
            continue;
        }

        if line.contains('@') {
            let split: Vec<&str> = line.trim().split('@').collect();
            let resolution = extract_tuple(split[0], "x").unwrap();
            let split: Vec<&str> = split[1].split(' ').collect();
            let position = extract_tuple(split[2], "x").unwrap();

            monitors.push(Monitor {
                id: current_id as usize,
                width: resolution.0,
                height: resolution.1,
                pos_x: position.0,
                pos_y: position.1,
            })
        }
    }

    monitors
}

// "1920, 1080" -> (1920, 1080)
fn get_cursor_position() -> (i32, i32) {
    let raw = Command::new("hyprctl")
        .arg("cursorpos")
        .output()
        .unwrap()
        .stdout;

    let input = from_utf8(&raw).unwrap().trim_end();
    extract_tuple(input, ", ").unwrap()
}

fn get_workspaces_ids() -> Vec<i32> {
    let raw = Command::new("hyprctl")
        .arg("workspaces")
        .output()
        .unwrap()
        .stdout;

    let input = from_utf8(&raw).unwrap().trim();
    let mut workspace_ids: Vec<i32> = Vec::new();

    // Split the input string by lines
    let lines = input.lines();

    // Iterate over each line
    for line in lines {
        // Split the line by whitespace
        if line.starts_with("workspace ID") {
            let words: Vec<&str> = line.split_whitespace().collect();
            if let Some(id) = words.get(2) {
                if let Ok(id_num) = id.parse::<i32>() {
                    workspace_ids.push(id_num);
                }
            }
        }
    }

    workspace_ids
}

// "1920x1080" -> (1820, 1080)
fn extract_tuple<F: FromStr>(input: &str, separator: &str) -> Result<(F, F), <F as FromStr>::Err> {
    let split: Vec<&str> = input.split(separator).collect();
    let v1: F = split[0].parse()?;
    let v2: F = split[1].parse()?;
    Ok((v1, v2))
}
