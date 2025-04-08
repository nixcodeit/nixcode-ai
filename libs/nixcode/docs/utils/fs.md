# File System Utilities

The file system utilities provide helper functions for working with files and directories.

## Overview

The file system utilities module (`utils/fs.rs`) contains functions for working with file paths, particularly for safely joining paths and preventing path traversal attacks.

## Functions

### join_path

```rust
pub fn join_path(base: impl Into<PathBuf>, path: impl Into<PathBuf>) -> anyhow::Result<PathBuf>
```

Joins a base path with a relative path, ensuring that the resulting path is within the base directory.

#### Parameters

- `base`: The base directory path
- `path`: The relative path to join with the base

#### Returns

- `Ok(PathBuf)`: The joined path if it is within the base directory
- `Err(anyhow::Error)`: An error if the path is absolute or would escape the base directory

#### Example

```rust
use crate::utils::fs;

let base = PathBuf::from("/home/user/project");
let path = PathBuf::from("src/main.rs");

let joined = fs::join_path(base, path).unwrap();
// joined = "/home/user/project/src/main.rs"

let base = PathBuf::from("/home/user/project");
let path = PathBuf::from("../../../etc/passwd");

let joined = fs::join_path(base, path);
// joined = Err(anyhow::Error("Path exceeds base directory"))
```

## Implementation Details

### Path Safety

The `join_path` function includes safety checks to ensure that paths are:
1. Relative (not absolute)
2. Within the base directory (no path traversal)

```rust
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
```

## Usage in Tools

The file system utilities are used by various tools to ensure that file operations are performed safely:

```rust
// In read_text_file.rs
let file_path = PathBuf::from(params.path);

let cwd = project.get_cwd();
let path = fs::join_path(cwd.clone(), file_path);
if path.is_err() {
    return json!(path.unwrap_err().to_string());
}

let path = path.unwrap();
if !path.starts_with(cwd) {
    return json!("Path must be inside project directory");
}
```