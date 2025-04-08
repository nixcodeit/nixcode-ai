# Command Tools

The command tools provide operations for executing external commands, particularly Cargo commands for Rust projects.

## Overview

Command tools allow the LLM to execute external commands, enabling it to build, test, format, and fix Rust code. These tools are implemented in the `tools/commands` directory.

## Available Tools

### CargoBuildTool

Builds a Rust project using Cargo.

```rust
#[derive(JsonSchema, Serialize, Deserialize)]
pub struct CargoBuildProps {
    #[serde(default)]
    pub package: Option<String>,
    #[serde(default)]
    pub release: Option<bool>,
}

#[tool("Build the Rust project using cargo")]
pub async fn cargo_build(props: CargoBuildProps, project: Arc<Project>) -> serde_json::Value {
    // Implementation
}
```

Example usage:
```json
{
  "name": "cargo_build",
  "parameters": {
    "release": true
  }
}
```

### CargoTestTool

Runs tests for a Rust project using Cargo.

```rust
#[derive(JsonSchema, Serialize, Deserialize)]
pub struct CargoTestProps {
    #[serde(default)]
    pub package: Option<String>,
    #[serde(default)]
    pub test: Option<String>,
    #[serde(default)]
    pub lib: Option<bool>,
    #[serde(default)]
    pub verbose: Option<bool>,
    #[serde(default)]
    pub quiet: Option<bool>,
    #[serde(default)]
    pub nocapture: Option<bool>,
}

#[tool("Run tests for a Rust project using cargo test")]
pub async fn cargo_test(props: CargoTestProps, project: Arc<Project>) -> serde_json::Value {
    // Implementation
}
```

Example usage:
```json
{
  "name": "cargo_test",
  "parameters": {
    "nocapture": true
  }
}
```

### CargoFmtTool

Formats Rust code using Cargo.

```rust
#[derive(JsonSchema, Serialize, Deserialize)]
pub struct CargoFmtProps {
    #[serde(default)]
    pub all: Option<bool>,
    #[serde(default)]
    pub check: Option<bool>,
    #[serde(default)]
    pub path: Option<String>,
}

#[tool("Format Rust code using cargo fmt")]
pub async fn cargo_fmt(props: CargoFmtProps, project: Arc<Project>) -> serde_json::Value {
    // Implementation
}
```

Example usage:
```json
{
  "name": "cargo_fmt",
  "parameters": {
    "all": true
  }
}
```

### CargoFixTool

Automatically fixes Rust code using Cargo.

```rust
#[derive(JsonSchema, Serialize, Deserialize)]
pub struct CargoFixProps {
    #[serde(default)]
    pub package: Option<String>,
    #[serde(default)]
    pub broken_code: Option<bool>,
    #[serde(default)]
    pub edition: Option<bool>,
}

#[tool("Automatically fix Rust code with cargo fix")]
pub async fn cargo_fix(props: CargoFixProps, project: Arc<Project>) -> serde_json::Value {
    // Implementation
}
```

Example usage:
```json
{
  "name": "cargo_fix",
  "parameters": {
    "edition": true
  }
}
```

## Implementation Details

### Command Execution

Command tools use Tokio's `Command` to execute external commands asynchronously:

```rust
let mut cmd = Command::new("cargo");
cmd.current_dir(current_dir).arg("build");

if let Some(package) = props.package {
    cmd.arg("--package").arg(package);
}

if release {
    cmd.arg("--release");
}

run_command(cmd).await
```

### Command Output Handling

Command output is captured and formatted using a common utility function:

```rust
pub async fn run_command(mut cmd: Command) -> serde_json::Value {
    let output = cmd.output().await;

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            let mut result = String::new();

            if !stdout.is_empty() {
                result.push_str(&format!("STDOUT:\n{}\n", stdout));
            }

            if !stderr.is_empty() {
                result.push_str(&format!("STDERR:\n{}\n", stderr));
            }

            if output.status.success() {
                result.push_str("Command executed successfully");
            } else {
                result.push_str(&format!(
                    "Command failed with exit code: {}",
                    output.status.code().unwrap_or(-1)
                ));
            }

            json!(result)
        }
        Err(e) => json!(format!("Failed to execute command: {}", e)),
    }
}
```

### Project Directory Resolution

Command tools use the project's repository path or current working directory:

```rust
let current_dir = project.get_repo_path().unwrap_or(project.get_cwd());
```

## Usage in Nixcode

Command tools are registered in the `Nixcode::new` method:

```rust
// Add cargo tools
tools.add_tool(Arc::new(CargoBuildTool {}));
tools.add_tool(Arc::new(CargoFmtTool {}));
tools.add_tool(Arc::new(CargoFixTool {}));
tools.add_tool(Arc::new(CargoTestTool {}));
```