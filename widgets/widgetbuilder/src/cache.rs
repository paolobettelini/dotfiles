use std::path::PathBuf;
use std::time::SystemTime;

pub struct FSCachedValue {
    path: PathBuf,
}

impl FSCachedValue {
    pub fn new(name: &str) -> Self {
        let mut path = Self::get_cache_dir();
        Self::init_cache_path(&path);

        // Add file to path
        path.push(name);

        Self { path }
    }

    pub fn read_content(&self) -> Option<String> {
        let content = std::fs::read_to_string(&self.path);

        Some(content.ok()?)
    }

    pub fn write_content(&self, content: &str) -> std::io::Result<()> {
        std::fs::write(&self.path, content)?;

        Ok(())
    }

    pub fn last_update(&self) -> Option<u64> {
        let metadata = std::fs::metadata(&self.path).ok()?;

        let modified_time = metadata.modified().ok()?;

        // Get the current time
        let current_time = SystemTime::now();

        // Calculate the time difference in seconds
        let time_difference = current_time.duration_since(modified_time).ok()?.as_secs();

        Some(time_difference)
    }

    pub fn get_cache_dir() -> PathBuf {
        let home = std::env::var("HOME").expect("Could not read $HOME variable");
        let dir = format!("{home}/.cache/widgets");

        PathBuf::from(dir)
    }

    pub fn init_cache_path(path: &PathBuf) {
        // Create all folders recursively
        let res = std::fs::create_dir_all(&path);

        if let Err(err) = res {
            panic!("Error creating cache path: {}", err);
        }
    }
}
