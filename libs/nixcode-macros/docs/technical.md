# Technical Documentation: nixcode-macros

This document provides a detailed technical explanation of the implementation of the `nixcode-macros` crate, focusing on the internal workings of the `#[tool]` procedural macro.

## Procedural Macro Overview

Procedural macros in Rust are a powerful feature that allows for code generation at compile time. They operate on the tokenized representation of Rust code, allowing for inspection, transformation, and generation of new code.

The `nixcode-macros` crate implements a procedural attribute macro named `#[tool]`, which is used to simplify the implementation of tools for LLMs in the nixcode-ai project.

## Implementation Details

### Entry Point

The entry point for the `#[tool]` macro is defined in `libs/nixcode-macros/src/lib.rs`:

```rust
#[proc_macro_attribute]
pub fn tool(args: TokenStream, input: TokenStream) -> TokenStream {
    // Macro implementation
}
```

This function takes two parameters:
- `args`: The arguments provided to the macro (in this case, an optional description string)
- `input`: The Rust code to which the macro is applied (in this case, a function definition)

The function returns a `TokenStream` that replaces the original code in the compiled output.

### Parsing Arguments

The macro first parses the optional description argument:

```rust
let description = if args.is_empty() {
    None
} else {
    let desc = parse_macro_input!(args as LitStr);
    Some(desc)
};
```

If no arguments are provided, `description` is set to `None`. Otherwise, the argument is parsed as a string literal (`LitStr`).

### Parsing the Function

Next, the macro parses the function to which it is applied:

```rust
let func = parse_macro_input!(input as ItemFn);
```

This uses the `syn` crate to parse the input tokens as a function item (`ItemFn`).

### Extracting Function Information

The macro extracts various pieces of information from the function:

```rust
let func_name = &func.sig.ident;
let tool_name = func_name.to_string();
```

It also converts the snake_case function name to PascalCase for the struct name:

```rust
let tool_name_pascal_case = tool_name
    .split('_')
    .map(|s| {
        s.chars()
            .enumerate()
            .map(|(i, c)| if i == 0 { c.to_ascii_uppercase() } else { c })
            .collect::<String>()
    })
    .collect::<String>();
```

### Validating Function Parameters

The macro validates that the function has at least one parameter:

```rust
if func.sig.inputs.len() < 1 {
    return Error::new_spanned(
        &func.sig.inputs,
        "Tool function must have minimum one parameter",
    )
    .to_compile_error()
    .into();
}
```

If the function doesn't have at least one parameter, a compile-time error is generated.

### Extracting Parameter Type

The macro extracts the type of the first parameter:

```rust
let input_arg = func.sig.inputs.first().unwrap();
let param_type = match input_arg {
    FnArg::Typed(PatType { ty, .. }) => {
        if let Type::Path(type_path) = &**ty {
            type_path
        } else {
            return Error::new_spanned(ty, "Expected a type path")
                .to_compile_error()
                .into();
        }
    }
    _ => {
        return Error::new_spanned(input_arg, "Expected a typed argument")
            .to_compile_error()
            .into()
    }
};
```

This code:
1. Gets the first parameter of the function
2. Checks that it's a typed parameter (not `self`)
3. Extracts the type path from the parameter

### Creating Identifiers

The macro creates identifiers for the generated code:

```rust
let param_ident = &param_type.path.segments.last().unwrap().ident;
let struct_ident = Ident::new(&format!("{}Tool", tool_name_pascal_case), func_name.span());
```

The `param_ident` is the identifier of the parameter type, and `struct_ident` is the identifier for the generated struct.

### Preparing the Description

The macro prepares the description for the tool:

```rust
let description_expr = if let Some(desc) = description {
    // Use the provided description
    quote! { #desc.to_string() }
} else {
    // Fallback to auto-generated description
    quote! { String::from(concat!("Use this tool for ", #tool_name, ". Auto generated.")) }
};
```

If a description was provided as an argument to the macro, it's used. Otherwise, a default description is generated based on the function name.

### Generating Code

Finally, the macro generates the code for the tool implementation:

```rust
let expanded = quote! {
    #func

    pub struct #struct_ident {}

    #[async_trait::async_trait]
    impl crate::tools::Tool for #struct_ident {
        fn get_name(&self) -> String {
            #tool_name.to_string()
        }

        fn get_schema(&self) -> nixcode_llm_sdk::tools::Tool {
            let schema = schemars::schema_for!(#param_ident);
            let mut parameters = serde_json::to_value(&schema).unwrap();
            let mut obj = parameters.as_object_mut().unwrap();
            if !obj.contains_key("properties") {
                obj.extend([
                    ("properties".into(), serde_json::json!({}))
                ]);
            }

            let tool_name = #tool_name.to_string();
            let description = #description_expr;

            nixcode_llm_sdk::tools::Tool::new(tool_name, description, parameters)
        }

        async fn execute(&self, params: serde_json::Value, project: std::sync::Arc<crate::project::Project>) -> anyhow::Result<serde_json::Value> {
            let params: #param_ident = serde_json::from_value(params)?;
            Ok(#func_name(params, project).await)
        }
    }
};
```

This generated code:

1. Includes the original function unchanged
2. Creates a new struct with the name derived from the function name
3. Implements the `Tool` trait for this struct:
   - `get_name()` returns the function name as the tool name
   - `get_schema()` generates a JSON Schema for the parameter type
   - `execute()` deserializes the parameters and calls the original function

### Returning the Generated Code

The macro returns the generated code as a `TokenStream`:

```rust
expanded.into()
```

## Code Generation Details

### Struct Generation

The macro generates a struct with a name derived from the function name. For example, if the function is named `search_glob_files`, the struct will be named `SearchGlobFilesTool`.

```rust
pub struct SearchGlobFilesTool {}
```

This struct is empty because it doesn't need to store any state. It's just a vehicle for implementing the `Tool` trait.

### Trait Implementation

The macro implements the `Tool` trait for the generated struct:

```rust
#[async_trait::async_trait]
impl crate::tools::Tool for SearchGlobFilesTool {
    // Implementation details
}
```

The `async_trait` attribute is used because the `execute` method is async.

### Name Method

The `get_name` method returns the function name as the tool name:

```rust
fn get_name(&self) -> String {
    "search_glob_files".to_string()
}
```

### Schema Method

The `get_schema` method generates a JSON Schema for the parameter type:

```rust
fn get_schema(&self) -> nixcode_llm_sdk::tools::Tool {
    let schema = schemars::schema_for!(GlobToolParams);
    let mut parameters = serde_json::to_value(&schema).unwrap();
    let mut obj = parameters.as_object_mut().unwrap();
    if !obj.contains_key("properties") {
        obj.extend([
            ("properties".into(), serde_json::json!({}))
        ]);
    }

    let tool_name = "search_glob_files".to_string();
    let description = "Search for files matching a glob pattern".to_string();

    nixcode_llm_sdk::tools::Tool::new(tool_name, description, parameters)
}
```

This method:
1. Uses `schemars::schema_for!` to generate a JSON Schema for the parameter type
2. Converts the schema to a `serde_json::Value`
3. Ensures that the schema has a `properties` field (even if empty)
4. Creates a new `Tool` with the name, description, and schema

### Execute Method

The `execute` method deserializes the parameters and calls the original function:

```rust
async fn execute(&self, params: serde_json::Value, project: std::sync::Arc<crate::project::Project>) -> anyhow::Result<serde_json::Value> {
    let params: GlobToolParams = serde_json::from_value(params)?;
    Ok(search_glob_files(params, project).await)
}
```

This method:
1. Deserializes the parameters from a `serde_json::Value` to the parameter type
2. Calls the original function with the deserialized parameters
3. Returns the result wrapped in an `Ok`

## Integration with the Tool System

The `#[tool]` macro is designed to work with the tool system in the nixcode-ai project. This system allows LLMs to invoke tools to perform actions in the real world.

The tool system requires each tool to implement the `Tool` trait, which defines:
- How to get the name of the tool
- How to get the schema of the tool's parameters
- How to execute the tool with a given set of parameters

The `#[tool]` macro automates the implementation of this trait, allowing developers to focus on implementing the tool's functionality without having to write boilerplate code.

## JSON Schema Generation

The `#[tool]` macro uses the `schemars` crate to generate a JSON Schema for the parameter type. This schema is used to validate and document the parameters that the tool accepts.

The `schemars::schema_for!` macro generates a schema for a Rust type. This schema is then converted to a `serde_json::Value` for use in the tool system.

The macro ensures that the schema has a `properties` field, even if it's empty. This is required by the tool system.

## Error Handling

The `#[tool]` macro includes compile-time error checks for:
- Functions with no parameters
- Parameters that are not properly typed

These checks help ensure that the tool is implemented correctly and can be used by the tool system.

## Conclusion

The `#[tool]` macro is a powerful tool for simplifying the implementation of tools in the nixcode-ai project. By automating the boilerplate code required to implement the `Tool` trait, it allows developers to focus on implementing the tool's functionality.

The macro uses advanced features of Rust's procedural macro system, including:
- Parsing Rust code using the `syn` crate
- Generating Rust code using the `quote` crate
- Compile-time error checking
- JSON Schema generation using the `schemars` crate

These features combine to create a powerful and flexible tool for implementing LLM tools in the nixcode-ai project.