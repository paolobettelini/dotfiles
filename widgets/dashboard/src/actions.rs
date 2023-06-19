use std::process::Command;

pub fn shutdown() {
    println!("Shutting down");

    Command::new("systemctl")
        .args(&["poweroff"])
        .spawn()
        .expect("Failed to execute shutdown command.");
}

pub fn reboot() {
    println!("Rebooting");

    Command::new("systemctl")
        .args(&["reboot"])
        .spawn()
        .expect("Failed to execute reboot command.");
}

pub fn logout() {
    println!("Logout not yet implemented");
}

pub fn suspend() {
    println!("Suspending");

    Command::new("systemctl")
        .args(&["suspend"])
        .spawn()
        .expect("Failed to execute suspend command.");
}

pub fn open_reddit() {
    Command::new("firefox")
        .args(&["--new-tab", "https://www.reddit.com/"])
        .spawn()
        .expect("Failed to execute firefox command.");
}

pub fn open_youtube() {
    Command::new("firefox")
        .args(&["--new-tab", "https://www.youtube.com/"])
        .spawn()
        .expect("Failed to execute firefox command.");
}

pub fn open_whatsapp() {
    Command::new("firefox")
        .args(&["--new-tab", "https://web.whatsapp.com/"])
        .spawn()
        .expect("Failed to execute firefox command.");
}

pub fn open_github() {
    Command::new("firefox")
        .args(&["--new-tab", "https://github.com/"])
        .spawn()
        .expect("Failed to execute firefox command.");
}