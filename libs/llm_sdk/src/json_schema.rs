#[macro_export]
macro_rules! json_schema {
    // Obsługa struktur/obiektów z polami
    ({ $($field:literal: $type:ty),* $(,)? }) => {{
        use serde_json::{json, Value};

        let mut properties = serde_json::Map::new();
        let mut required = Vec::new();

        $(
            properties.insert($field.to_string(), json_schema!($type));

            // Jeśli typ nie jest Option<T>, jest wymagany
            if !stringify!($type).starts_with("Option<") {
                required.push($field.to_string());
            }
        )*

        let mut schema = json!({
            "type": "object",
            "properties": properties
        });

        if !required.is_empty() {
            schema.as_object_mut().unwrap().insert("required".to_string(), json!(required));
        }

        schema
    }};

    // Obsługa typów podstawowych
    (u8) => {{ serde_json::json!({ "type": "integer", "minimum": 0, "maximum": 255 }) }};
    (u16) => {{ serde_json::json!({ "type": "integer", "minimum": 0, "maximum": 65535 }) }};
    (u32) => {{ serde_json::json!({ "type": "integer", "minimum": 0, "maximum": 4294967295 }) }};
    (u64) => {{ serde_json::json!({ "type": "integer", "minimum": 0 }) }};
    (u128) => {{ serde_json::json!({ "type": "integer", "minimum": 0 }) }};
    (usize) => {{ serde_json::json!({ "type": "integer", "minimum": 0 }) }};
    (i8) => {{ serde_json::json!({ "type": "integer", "minimum": -128, "maximum": 127 }) }};
    (i16) => {{ serde_json::json!({ "type": "integer", "minimum": -32768, "maximum": 32767 }) }};
    (i32) => {{ serde_json::json!({ "type": "integer", "minimum": -2147483648, "maximum": 2147483647 }) }};
    (i64) => {{ serde_json::json!({ "type": "integer" }) }};
    (i128) => {{ serde_json::json!({ "type": "integer" }) }};
    (isize) => {{ serde_json::json!({ "type": "integer" }) }};
    (f32) => {{ serde_json::json!({ "type": "number" }) }};
    (f64) => {{ serde_json::json!({ "type": "number" }) }};
    (bool) => {{ serde_json::json!({ "type": "boolean" }) }};
    (String) => {{ serde_json::json!({ "type": "string" }) }};
    (&str) => {{ serde_json::json!({ "type": "string" }) }};
    (char) => {{ serde_json::json!({ "type": "string", "maxLength": 1 }) }};

    // Obsługa typu Option<T>
    (Option<$type:ty>) => {{
        use serde_json::{json, Value};

        let inner_schema = json_schema!($type);

        // Utwórz nowy schemat, który pozwala na typ lub null
        let mut schema = inner_schema.as_object().unwrap().clone();

        if let Some(t) = schema.get("type") {
            if t.is_string() {
                schema.insert("type".to_string(), json!([t.as_str().unwrap(), "null"]));
            } else if t.is_array() {
                let mut types = t.as_array().unwrap().clone();
                types.push(json!("null"));
                schema.insert("type".to_string(), json!(types));
            }
        } else {
            schema.insert("type".to_string(), json!(["null", "object"]));
        }

        json!(schema)
    }};

    // Obsługa Vec<T> lub tablicy
    (Vec<$type:ty>) => {{
        use serde_json::{json, Value};

        let items = json_schema!($type);

        json!({
            "type": "array",
            "items": items
        })
    }};

    // Obsługa tablic o ustalonej długości
    ([$type:ty; $len:expr]) => {{
        use serde_json::{json, Value};

        let items = json_schema!($type);

        json!({
            "type": "array",
            "items": items,
            "minItems": $len,
            "maxItems": $len
        })
    }};

    // Obsługa map
    (std::collections::HashMap<$key:ty, $value:ty>) => {{
        use serde_json::{json, Value};

        let value_schema = json_schema!($value);

        json!({
            "type": "object",
            "additionalProperties": value_schema
        })
    }};

    // Obsługa BTreeMap
    (std::collections::BTreeMap<$key:ty, $value:ty>) => {{
        use serde_json::{json, Value};

        let value_schema = json_schema!($value);

        json!({
            "type": "object",
            "additionalProperties": value_schema
        })
    }};

    // Obsługa krotek
    (($($type:ty),+)) => {{
        use serde_json::{json, Value};

        let items = vec![$(json_schema!($type)),+];

        json!({
            "type": "array",
            "items": items,
            "minItems": items.len(),
            "maxItems": items.len()
        })
    }};

    // Obsługa typu Unit
    (()) => {{
        serde_json::json!({
            "type": "null"
        })
    }};

    // Obsługa innych typów (struktury, enumy, itp.)
    ($type:ty) => {{
        // To jest przypadek domyślny
        serde_json::json!({
            "type": "object",
            "description": stringify!($type)
        })
    }};
}
