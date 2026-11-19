//! Event-driven wallpaper policies for the Quickshell music widget.
//!
//! Protocol compatible with the previous wallpaper-manager.py:
//!   sync TITLE ARTIST | observe PATH | test TITLE ARTIST | status
//!
//! The layout config is read-only. Only the separate music-manager.json is
//! persisted. A process-wide flock serializes simultaneous Quickshell events.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::env;
use std::ffi::OsStr;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::os::fd::AsRawFd;
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const WALLPAPER_TIMEOUT: Duration = Duration::from_secs(90);
const IMAGE_EXTENSIONS: [&str; 4] = ["jpg", "jpeg", "png", "webp"];

#[derive(Debug)]
struct Paths {
    home: PathBuf,
    walls: PathBuf,
    current: PathBuf,
    state: PathBuf,
    lock: PathBuf,
    config: PathBuf,
}

impl Paths {
    fn from_home(home: PathBuf) -> Self {
        let root = home.join(".local/state/pocket-wallfx");
        // The installed binary lives in .../scripts/wallpaper-manager.
        // Resolve the config relative to that executable, NOT the current cwd.
        let config = home.join(".local/src/quickshell-music-player/desktop-widget-positions.json");
        Self {
            walls: home.join("Pictures/wallpapers"),
            current: root.join("current"),
            state: root.join("music-manager.json"),
            lock: root.join("music-manager.lock"),
            config,
            home,
        }
    }

    fn runtime() -> Result<Self, String> {
        let home = env::var_os("HOME").ok_or("HOME is not set")?;
        let mut paths = Self::from_home(PathBuf::from(home));
        // Allow the manager to be installed elsewhere, as long as it sits in
        // <quickshell-root>/scripts/ and the layout JSON is next to scripts/.
        if let Ok(executable) = env::current_exe() {
            if let Some(project_dir) = executable.parent().and_then(Path::parent) {
                let candidate = project_dir.join("desktop-widget-positions.json");
                if candidate.is_file() {
                    paths.config = candidate;
                }
            }
        }
        Ok(paths)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct State {
    base: String,
    seen: String,
    #[serde(rename = "lastTrackKey")]
    last_track_key: String,
    #[serde(rename = "suppressedTrackKey")]
    suppressed_track_key: String,
    target: String,
    previous: String,
    pending: bool,
}

impl State {
    fn fresh(paths: &Paths) -> Self {
        let here = read_current(paths);
        Self {
            base: here.clone(),
            seen: here.clone(),
            last_track_key: String::new(),
            suppressed_track_key: String::new(),
            target: here.clone(),
            previous: here,
            pending: false,
        }
    }

    fn load(paths: &Paths) -> Self {
        let mut state = Self::fresh(paths);
        let data = read_json(&paths.state);
        // Accept partial legacy Python state files without resetting the base.
        for (key, slot) in [
            ("base", &mut state.base),
            ("seen", &mut state.seen),
            ("lastTrackKey", &mut state.last_track_key),
            ("suppressedTrackKey", &mut state.suppressed_track_key),
            ("target", &mut state.target),
            ("previous", &mut state.previous),
        ] {
            if let Some(value) = data.get(key).and_then(Value::as_str) {
                *slot = value.to_owned();
            }
        }
        if let Some(value) = data.get("pending").and_then(Value::as_bool) {
            state.pending = value;
        }
        state
    }

    fn manual_change(&mut self, path: &str) {
        self.base = path.to_owned();
        self.seen = path.to_owned();
        self.target = path.to_owned();
        self.previous = path.to_owned();
        self.pending = false;
        // The user's choice wins until the track changes.
        self.suppressed_track_key.clone_from(&self.last_track_key);
    }

    fn observe(&mut self, path: &str) {
        if path.is_empty() || !Path::new(path).is_file() {
            return;
        }
        if self.pending {
            if path == self.target {
                self.seen = path.to_owned();
                self.pending = false;
                return;
            }
            if path == self.previous {
                return; // The automatic transition has not completed yet.
            }
        }
        if path != self.seen {
            self.manual_change(path);
        }
    }

    fn reconcile(&mut self, paths: &Paths, title: &str, artist: &str) -> Option<String> {
        let key = if title.is_empty() {
            String::new()
        } else {
            music_key(title, artist)
        };
        let current = canonical(&read_current(paths), &paths.home);
        // Also notice manual changes when FileView notifications are coalesced.
        self.observe(&current);

        if self.last_track_key != key {
            self.suppressed_track_key.clear();
        }
        self.last_track_key = key.clone();

        let mut override_path = if key.is_empty() {
            None
        } else {
            find_music_override(&read_json(&paths.config), title, artist, paths)
        };
        if !key.is_empty() && self.suppressed_track_key == key {
            override_path = None;
        }

        let target = override_path.unwrap_or_else(|| self.base.clone());
        if target.is_empty() || !Path::new(&target).is_file() {
            return None;
        }
        let target = canonical(&target, &paths.home);
        if current == target {
            self.target = target;
            self.seen = current;
            self.pending = false;
            return None;
        }
        self.previous = current;
        self.target.clone_from(&target);
        self.pending = true;
        Some(target)
    }
}

fn read_json(path: &Path) -> Value {
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .filter(Value::is_object)
        .unwrap_or_else(|| json!({}))
}

fn read_current(paths: &Paths) -> String {
    fs::read_to_string(&paths.current)
        .unwrap_or_default()
        .trim()
        .to_owned()
}

fn canonical(input: &str, home: &Path) -> String {
    if input.is_empty() {
        return String::new();
    }
    let path = if input == "~" {
        home.to_path_buf()
    } else if let Some(suffix) = input.strip_prefix("~/") {
        home.join(suffix)
    } else {
        PathBuf::from(input)
    };
    let absolute = if path.is_absolute() {
        path
    } else {
        env::current_dir().unwrap_or_else(|_| home.to_path_buf()).join(path)
    };
    fs::canonicalize(&absolute)
        .unwrap_or(absolute)
        .to_string_lossy()
        .into_owned()
}

fn fold(text: &str) -> String {
    text.trim().to_lowercase()
}

fn music_key(title: &str, artist: &str) -> String {
    // Match json.dumps([title.casefold(), artist.casefold()]) spacing from the
    // Python manager, so pending/suppression state survives migration.
    let left = serde_json::to_string(&fold(title)).expect("JSON string");
    let right = serde_json::to_string(&fold(artist)).expect("JSON string");
    format!("[{left}, {right}]")
}

fn contains_any(needles: &Value, haystack: &str) -> bool {
    needles.as_array().is_some_and(|array| {
        array.iter().filter_map(Value::as_str).any(|needle| {
            let needle = fold(needle);
            !needle.is_empty() && haystack.contains(&needle)
        })
    })
}

fn safe_image_name(filename: &str) -> bool {
    let path = Path::new(filename);
    path.components().count() == 1
        && path.file_name() == Some(OsStr::new(filename))
        && path.extension().and_then(OsStr::to_str).is_some_and(|ext| {
            IMAGE_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str())
        })
}

fn find_music_override(config: &Value, title: &str, artist: &str, paths: &Paths) -> Option<String> {
    let entries = config.get("wallpapers")?.as_object()?;
    let title_cf = fold(title);
    let artist_cf = fold(artist);
    // Firefox/YouTube often reports a channel as artist and "Artist - Song"
    // as title. Match the artist prefix as the previous manager did.
    let title_artist = title
        .split_once(" - ")
        .map(|(prefix, _)| fold(prefix))
        .unwrap_or_default();

    // serde_json's preserve_order feature is intentional: the first rule wins.
    for (filename, rules) in entries {
        if !rules.is_object() {
            continue;
        }
        let author_match = contains_any(&rules["authors"], &artist_cf)
            || (!title_artist.is_empty() && contains_any(&rules["authors"], &title_artist));
        let title_match = contains_any(&rules["songs"], &title_cf);
        if !(author_match || title_match) || !safe_image_name(filename) {
            continue;
        }
        let candidate = paths.walls.join(filename);
        if candidate.is_file() {
            return Some(canonical(&candidate.to_string_lossy(), &paths.home));
        }
        eprintln!("[wallpaper-manager] matching file missing: {}", candidate.display());
    }
    None
}

fn write_state(paths: &Paths, state: &State) -> Result<(), String> {
    let parent = paths.state.parent().ok_or("invalid state location")?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let tmp_path = parent.join(format!("music-manager-{}-{nanos}.tmp", process::id()));
    let result: io::Result<()> = (|| {
        let mut file = OpenOptions::new().write(true).create_new(true).open(&tmp_path)?;
        serde_json::to_writer_pretty(&mut file, state).map_err(io::Error::other)?;
        file.write_all(b"\n")?;
        file.sync_all()?;
        fs::rename(&tmp_path, &paths.state)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&tmp_path);
    }
    result.map_err(|error| error.to_string())
}

fn acquire_lock(paths: &Paths) -> Result<File, String> {
    fs::create_dir_all(paths.lock.parent().ok_or("invalid lock location")?)
        .map_err(|error| error.to_string())?;
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&paths.lock)
        .map_err(|error| error.to_string())?;
    // SAFETY: `file` is owned and open for the lifetime of this lock, and its
    // descriptor is valid. flock releases automatically when `file` is dropped.
    if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX) } != 0 {
        return Err(io::Error::last_os_error().to_string());
    }
    Ok(file)
}

fn apply_wallpaper(paths: &Paths, wallpaper: &str) -> Result<(), String> {
    let local = paths.home.join(".local/bin/pocket-wallpaper");
    let executable = if local.is_file() {
        local.as_os_str()
    } else {
        OsStr::new("pocket-wallpaper") // Command resolves this via PATH.
    };
    let mut child = Command::new(executable)
        .arg("set")
        .arg(wallpaper)
        .spawn()
        .map_err(|error| error.to_string())?;
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) if status.success() => return Ok(()),
            Ok(Some(status)) => return Err(format!("pocket-wallpaper exited with {status}")),
            Ok(None) if start.elapsed() >= WALLPAPER_TIMEOUT => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("pocket-wallpaper timed out after 90 seconds".to_owned());
            }
            Ok(None) => thread::sleep(Duration::from_millis(80)),
            Err(error) => return Err(error.to_string()),
        }
    }
}

fn execute_with<F>(
    paths: &Paths,
    command: &str,
    args: &[String],
    apply: &mut F,
) -> Result<Option<String>, String>
where
    F: FnMut(&str) -> Result<(), String>,
{
    let _lock = acquire_lock(paths)?;
    let mut state = State::load(paths);
    match command {
        "observe" => {
            state.observe(&canonical(&args[0], &paths.home));
        }
        "sync" => {
            let title = args[0].trim();
            let artist = args[1].trim();
            let target = state.reconcile(paths, title, artist);
            // Persist pending BEFORE launching the wallpaper transition.
            write_state(paths, &state)?;
            if let Some(target) = target {
                if let Err(error) = apply(&target) {
                    state.pending = false;
                    state.target = read_current(paths);
                    write_state(paths, &state)?;
                    return Err(format!("wallpaper switch failed: {error}"));
                }
                let now = canonical(&read_current(paths), &paths.home);
                if now == target {
                    state.pending = false;
                    state.seen = now;
                }
            }
        }
        "status" => {
            let text = serde_json::to_string_pretty(&state).map_err(|error| error.to_string())?;
            return Ok(Some(text));
        }
        _ => return Err("invalid command".to_owned()),
    }
    write_state(paths, &state)?;
    Ok(None)
}

fn usage() {
    eprintln!("Usage: wallpaper-manager [sync TITLE ARTIST | observe PATH | test TITLE ARTIST | status]");
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let paths = match Paths::runtime() {
        Ok(paths) => paths,
        Err(error) => {
            eprintln!("[wallpaper-manager] {error}");
            process::exit(1);
        }
    };
    let result = match args.as_slice() {
        [command, title, artist] if command == "test" => {
            let match_path = find_music_override(&read_json(&paths.config), title, artist, &paths);
            println!("{}", match_path.as_deref().unwrap_or("(no match)"));
            return;
        }
        [command, title, artist] if command == "sync" => execute_with(
            &paths,
            "sync",
            &[title.clone(), artist.clone()],
            &mut |target| apply_wallpaper(&paths, target),
        ),
        [command, path] if command == "observe" => execute_with(
            &paths,
            "observe",
            std::slice::from_ref(path),
            &mut |_target| Ok(()),
        ),
        [command] if command == "status" => execute_with(
            &paths,
            "status",
            &[],
            &mut |_target| Ok(()),
        ),
        _ => {
            usage();
            process::exit(2);
        }
    };
    match result {
        Ok(Some(text)) => println!("{text}"),
        Ok(None) => (),
        Err(error) => {
            eprintln!("[wallpaper-manager] {error}");
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_TEST_ID: AtomicU64 = AtomicU64::new(0);

    struct Fixture {
        root: PathBuf,
        paths: Paths,
        switches: RefCell<Vec<String>>,
    }

    impl Fixture {
        fn new() -> Self {
            let id = NEXT_TEST_ID.fetch_add(1, Ordering::Relaxed);
            let root = env::temp_dir().join(format!("qs-wallpaper-{}-{id}", process::id()));
            fs::create_dir_all(&root).unwrap();
            let mut paths = Paths::from_home(root.clone());
            paths.config = root.join("config.json");
            fs::create_dir_all(&paths.walls).unwrap();
            fs::create_dir_all(paths.current.parent().unwrap()).unwrap();
            for name in ["base.jpg", "0.jpg", "1.jpg", "9.jpg"] {
                File::create(paths.walls.join(name)).unwrap();
            }
            fs::write(
                &paths.current,
                format!("{}\n", paths.walls.join("base.jpg").display()),
            )
            .unwrap();
            fs::write(
                &paths.config,
                r#"{"wallpapers":{"9.jpg":{"lyricsX":300},"0.jpg":{"authors":["Macintosh"]},"1.jpg":{"songs":["Nightcall"]}}}"#,
            )
            .unwrap();
            Self { root, paths, switches: RefCell::new(Vec::new()) }
        }

        fn run(&self, command: &str, args: &[&str]) {
            execute_with(
                &self.paths,
                command,
                &args.iter().map(|arg| (*arg).to_owned()).collect::<Vec<_>>(),
                &mut |target| {
                    self.switches.borrow_mut().push(
                        Path::new(target).file_name().unwrap().to_string_lossy().into_owned(),
                    );
                    fs::write(&self.paths.current, format!("{target}\n"))
                        .map_err(|error| error.to_string())?;
                    Ok(())
                },
            )
            .unwrap();
        }

        fn path(&self, name: &str) -> String {
            self.paths.walls.join(name).to_string_lossy().into_owned()
        }

        fn names(&self) -> Vec<String> {
            self.switches.borrow().clone()
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn substring_case_insensitive() {
        let t = Fixture::new();
        let conf = read_json(&t.paths.config);
        assert_eq!(
            find_music_override(&conf, "Random", "MACINTOSH plus", &t.paths),
            Some(t.path("0.jpg")),
        );
        assert_eq!(
            find_music_override(&conf, "Kavinsky - nIgHtCaLl", "someone", &t.paths),
            Some(t.path("1.jpg")),
        );
        assert_eq!(
            find_music_override(&conf, "Macintosh - Song", "youtube-channel", &t.paths),
            Some(t.path("0.jpg")),
        );
    }

    #[test]
    fn unmatched_song_uses_no_override() {
        let t = Fixture::new();
        assert_eq!(find_music_override(&read_json(&t.paths.config), "Unknown", "None", &t.paths), None);
    }

    #[test]
    fn switch_between_matches_then_restore_base() {
        let t = Fixture::new();
        t.run("observe", &[&t.path("base.jpg")]);
        t.run("sync", &["A", "Macintosh PLUS"]);
        t.run("sync", &["Nightcall", "Kavinsky"]);
        t.run("sync", &["Unrelated", "Someone else"]);
        assert_eq!(t.names(), vec!["0.jpg".to_owned(), "1.jpg".to_owned(), "base.jpg".to_owned()]);
        let expected = t.path("base.jpg");
        assert_eq!(read_json(&t.paths.state)["base"].as_str(), Some(expected.as_str()));
    }

    #[test]
    fn manual_selection_suppresses_until_next_track() {
        let t = Fixture::new();
        t.run("sync", &["Song", "Macintosh"]);
        fs::write(&t.paths.current, format!("{}\n", t.path("9.jpg"))).unwrap();
        t.run("observe", &[&t.path("9.jpg")]);
        t.run("sync", &["Song", "Macintosh"]);
        assert_eq!(t.names(), vec!["0.jpg".to_owned()]);
        t.run("sync", &["Unmatched", "Other"]);
        assert_eq!(PathBuf::from(read_current(&t.paths)), t.paths.walls.join("9.jpg"));
        t.run("sync", &["Nightcall", "Kavinsky"]);
        t.run("sync", &["Another", "Other"]);
        assert_eq!(t.names(), vec!["0.jpg".to_owned(), "1.jpg".to_owned(), "9.jpg".to_owned()]);
    }

    #[test]
    fn old_current_while_pending_is_not_a_manual_switch() {
        let t = Fixture::new();
        let mut state = State::fresh(&t.paths);
        state.pending = true;
        state.previous = t.path("base.jpg");
        state.target = t.path("0.jpg");
        state.observe(&t.path("base.jpg"));
        assert!(state.pending);
        state.observe(&t.path("0.jpg"));
        assert!(!state.pending);
        assert_eq!(state.base, t.path("base.jpg"));
    }

    #[test]
    fn missing_image_is_skipped() {
        let t = Fixture::new();
        fs::write(&t.paths.config, r#"{"wallpapers":{"missing.jpg":{"songs":["Nightcall"]}}}"#).unwrap();
        t.run("sync", &["Nightcall", "Kavinsky"]);
        assert!(t.names().is_empty());
    }

    #[test]
    fn order_of_json_rules_is_preserved() {
        let t = Fixture::new();
        fs::write(
            &t.paths.config,
            r#"{"wallpapers":{"9.jpg":{"songs":["Nightcall"]},"1.jpg":{"songs":["Nightcall"]}}}"#,
        ).unwrap();
        assert_eq!(find_music_override(&read_json(&t.paths.config), "Nightcall", "Kavinsky", &t.paths), Some(t.path("9.jpg")));
    }

    #[test]
    fn failed_switch_clears_pending_without_losing_base() {
        let t = Fixture::new();
        let error = execute_with(
            &t.paths,
            "sync",
            &["Nightcall".to_owned(), "Kavinsky".to_owned()],
            &mut |_target| Err("simulated helper failure".to_owned()),
        );
        assert!(error.is_err());
        let state = State::load(&t.paths);
        assert!(!state.pending);
        assert_eq!(state.base, t.path("base.jpg"));
        assert_eq!(state.target, t.path("base.jpg"));
    }

    #[test]
    fn python_state_format_is_compatible() {
        let t = Fixture::new();
        let old = json!({
            "base": t.path("9.jpg"),
            "seen": t.path("0.jpg"),
            "lastTrackKey": "[\"song\", \"macintosh\"]",
            "suppressedTrackKey": "[\"song\", \"macintosh\"]",
            "pending": false
        });
        fs::write(&t.paths.state, old.to_string()).unwrap();
        let state = State::load(&t.paths);
        assert_eq!(state.base, t.path("9.jpg"));
        assert_eq!(state.suppressed_track_key, music_key("Song", "Macintosh"));
    }
}
