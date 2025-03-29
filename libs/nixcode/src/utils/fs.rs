use std::ffi::OsStr;
use std::path::PathBuf;

pub fn join_path(base: impl Into<PathBuf>, path: impl Into<PathBuf>) -> anyhow::Result<PathBuf> {
    let path = path.into();
    let mut base = base.into();

    if path.is_absolute() {
        return Err(anyhow::anyhow!("Path must be relative"));
    }

    for part in path.iter() {
        if part == OsStr::new("..") {
            if !base.pop() {
                return Err(anyhow::anyhow!("Path exceeds base directory"));
            }
        } else if part != OsStr::new(".") && part != OsStr::new("/") {
            base.push(part);
        }
    }

    Ok(base)
}
