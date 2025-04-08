# Utilities

The utilities module provides common utility functions used throughout the nixcode library.

## Overview

The utilities module contains helper functions and utilities that are used by multiple components of the library. These utilities are organized into categories based on their functionality.

## Categories

- [File System Utilities](./fs.md): Utilities for working with files and directories

## Usage

Utilities are typically imported and used by other modules in the library:

```rust
use crate::utils::fs;

let path = fs::join_path(cwd.clone(), file_path);
```