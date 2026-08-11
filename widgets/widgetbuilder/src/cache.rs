use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone)]
pub struct FsValue {
    path: PathBuf,
}

impl FsValue {
    pub fn cache(namespace: &str, name: &str) -> io::Result<Self> {
        Self::under(xdg_cache_home()?, namespace, name)
    }

    pub fn state(namespace: &str, name: &str) -> io::Result<Self> {
        Self::under(xdg_state_home()?, namespace, name)
    }

    pub fn from_path(path: impl Into<PathBuf>) -> io::Result<Self> {
        let path = path.into();
        ensure_parent(&path)?;
        Ok(Self { path })
    }

    fn under(mut base: PathBuf, namespace: &str, name: &str) -> io::Result<Self> {
        base.push(namespace);
        fs::create_dir_all(&base)?;
        base.push(name);
        Ok(Self { path: base })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn exists(&self) -> bool {
        self.path.is_file()
    }

    pub fn read_string(&self) -> io::Result<String> {
        fs::read_to_string(&self.path)
    }

    pub fn read_optional(&self) -> io::Result<Option<String>> {
        match self.read_string() {
            Ok(value) => Ok(Some(value)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error),
        }
    }

    pub fn write_atomic(&self, content: &str) -> io::Result<()> {
        ensure_parent(&self.path)?;

        let file_name = self
            .path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("value");
        let temp_name = format!(".{file_name}.{}.tmp", std::process::id());
        let temp_path = self.path.with_file_name(temp_name);

        fs::write(&temp_path, content)?;
        if let Err(error) = fs::rename(&temp_path, &self.path) {
            let _ = fs::remove_file(&temp_path);
            return Err(error);
        }

        Ok(())
    }

    pub fn age(&self) -> io::Result<Duration> {
        let modified = fs::metadata(&self.path)?.modified()?;
        SystemTime::now()
            .duration_since(modified)
            .map_err(|error| io::Error::new(io::ErrorKind::Other, error))
    }
}

pub fn xdg_cache_home() -> io::Result<PathBuf> {
    if let Some(path) = non_empty_env_path("XDG_CACHE_HOME") {
        return Ok(path);
    }

    Ok(home_dir()?.join(".cache"))
}

pub fn xdg_state_home() -> io::Result<PathBuf> {
    if let Some(path) = non_empty_env_path("XDG_STATE_HOME") {
        return Ok(path);
    }

    Ok(home_dir()?.join(".local/state"))
}

pub fn legacy_widgets_cache(name: &str) -> io::Result<PathBuf> {
    Ok(home_dir()?.join(".cache/widgets").join(name))
}

fn home_dir() -> io::Result<PathBuf> {
    non_empty_env_path("HOME").ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "neither HOME nor an XDG directory is configured",
        )
    })
}

fn non_empty_env_path(name: &str) -> Option<PathBuf> {
    let value = env::var_os(name)?;
    if value.is_empty() {
        None
    } else {
        Some(PathBuf::from(value))
    }
}

fn ensure_parent(path: &Path) -> io::Result<()> {
    if let Some(parent) = path.parent().filter(|parent| !parent.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}
