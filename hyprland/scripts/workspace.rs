// Compile with: rustc -O workspace.rs
// usage: workspace <command> <workspace>

use std::{process::Command, str::from_utf8, env::args};

fn main() {
    let monitor_widths = vec![
        1920, 1920, 1920
    ];

    let args: Vec<String> = args().collect();

    // Read parameters
    let command = &args.get(1).unwrap();
    let workspace = &args.get(2).unwrap();
    let workspace = workspace.parse::<usize>().unwrap();

    // Compile workspace index
    let monitor_index = get_current_monitor_index(&monitor_widths);
    let workspace_index = (workspace-1) * monitor_widths.len() + monitor_index;

    Command::new("hyprctl")
        .arg("dispatch")
        .arg(&command)
        .arg(&workspace_index.to_string())
        .output()
        .unwrap();
}

fn get_current_monitor_index(monitor_widths: &Vec<usize>) -> usize {
    let mut monitor_index = None;
    let pos_x = get_cursor_position().0;

    let mut sum = 0usize;
    for (i, monitor_width) in monitor_widths.iter().enumerate() {
        sum += monitor_width;

        if pos_x <= sum {
            monitor_index = Some(i + 1);
            break;
        }
    }

    monitor_index.unwrap_or(1)
}

fn get_cursor_position() -> (usize, usize) {
    let raw = Command::new("hyprctl")
        .arg("cursorpos")
        .output()
        .unwrap()
        .stdout;

    let input = from_utf8(&raw).unwrap().trim_end();

    let split: Vec<&str> = input.split(", ").collect();
    let num1: usize = split[0].parse().unwrap();
    let num2: usize = split[1].parse().unwrap();
    (num1, num2)
}