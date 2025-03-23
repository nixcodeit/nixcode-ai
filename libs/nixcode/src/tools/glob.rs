use std::path::PathBuf;
use glob::glob;
use schemars::{json_schema, schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::json;
use nixcode_macros::{struct_tool, tool};

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GlobToolParams {
    #[schemars(description = "Glob pattern")]
    pattern: String,
}

#[tool]
pub fn search_glob_files(params: GlobToolParams) -> serde_json::Value {
    let result = glob(params.pattern.as_str());

    let mut result_str = String::new();
    let tool_result = match result {
        Ok(paths) => {
            result_str.push_str("Glob results:\n");
            for path in paths {
                match path {
                    Ok(path) => {
                        result_str.push_str(&format!("{:?}\n", path.display()));
                    }
                    _ => (),
                }
            }

            serde_json::to_value(result_str)
        }
        Err(e) => serde_json::to_value(e.to_string())
    };

    tool_result.unwrap_or_else(|e| json!(e.to_string()))
}
