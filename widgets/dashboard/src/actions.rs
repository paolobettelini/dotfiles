use gtk::gio;
use std::env;
use std::process::Command;

pub type ActionResult = Result<(), String>;

pub fn shutdown() -> ActionResult {
    spawn_command("systemctl", &["poweroff"])
}

pub fn reboot() -> ActionResult {
    spawn_command("systemctl", &["reboot"])
}

pub fn suspend() -> ActionResult {
    spawn_command("systemctl", &["suspend"])
}

pub fn logout() -> ActionResult {
    if Command::new("hyprctl")
        .args(["dispatch", "exit"])
        .spawn()
        .is_ok()
    {
        return Ok(());
    }

    if let Ok(session) = env::var("XDG_SESSION_ID") {
        return spawn_command("loginctl", &["terminate-session", &session]);
    }

    Err("could not start hyprctl and XDG_SESSION_ID is unavailable".to_string())
}

pub fn open_url(url: &str) -> ActionResult {
    gio::AppInfo::launch_default_for_uri(url, None::<&gio::AppLaunchContext>)
        .map_err(|error| format!("failed to open {url}: {error}"))
}

fn spawn_command(program: &str, args: &[&str]) -> ActionResult {
    Command::new(program)
        .args(args)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("failed to execute {program}: {error}"))
}
