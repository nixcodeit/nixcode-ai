use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Error, FnArg, Ident, ItemFn, LitStr, PatType, Type};

#[proc_macro_attribute]
pub fn tool(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the description argument
    let description = if args.is_empty() {
        None
    } else {
        let desc = parse_macro_input!(args as LitStr);
        Some(desc)
    };

    let func = parse_macro_input!(input as ItemFn);

    let func_name = &func.sig.ident;
    let tool_name = func_name.to_string();
    let tool_name_pascal_case = tool_name
        .split('_')
        .map(|s| {
            s.chars()
                .enumerate()
                .map(|(i, c)| if i == 0 { c.to_ascii_uppercase() } else { c })
                .collect::<String>()
        })
        .collect::<String>();

    // Extract function parameters
    if func.sig.inputs.len() < 1 {
        return Error::new_spanned(
            &func.sig.inputs,
            "Tool function must have minimum one parameter",
        )
        .to_compile_error()
        .into();
    }

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

    let param_ident = &param_type.path.segments.last().unwrap().ident;
    let struct_ident = Ident::new(&format!("{}Tool", tool_name_pascal_case), func_name.span());

    // Prepare the description
    let description_expr = if let Some(desc) = description {
        // Use the provided description
        quote! { #desc.to_string() }
    } else {
        // Fallback to auto-generated description
        quote! { String::from(concat!("Use this tool for ", #tool_name, ". Auto generated.")) }
    };

    let expanded = quote! {
        #func

        pub struct #struct_ident {}

        #[async_trait::async_trait]
        impl crate::tools::Tool for #struct_ident {
            fn get_name(&self) -> String {
                #tool_name.to_string()
            }

            fn get_schema(&self) -> nixcode_llm_sdk::tools::Tool {
                use schemars::transform::{Transform, RecursiveTransform};
                use schemars::generate::SchemaSettings;
                use schemars::Schema;

                let settings = SchemaSettings::default().with(|s| {
                    s.option_nullable = false;
                    s.option_add_null_type = false;
                });
                let generator = settings.into_generator();
                let mut schema = generator.into_root_schema_for::<#param_ident>();

                let mut transform = RecursiveTransform(|schema: &mut Schema| {
                    schema.remove("default");
                    schema.remove("format");
                    schema.remove("minimum");
                    schema.remove("maximum");
                });

                transform.transform(&mut schema);

                let mut parameters = serde_json::to_value(&schema).unwrap();
                let mut obj = parameters.as_object_mut().unwrap();
                if !obj.contains_key("properties") {
                    obj.extend([
                        ("properties".into(), serde_json::json!({}))
                    ]);
                }

                let tool_name = #tool_name.to_string();
                let description = #description_expr;

                nixcode_llm_sdk::tools::Tool::new(tool_name, description, schema)
            }

            async fn execute(&self, params: serde_json::Value, project: std::sync::Arc<crate::project::Project>) -> anyhow::Result<serde_json::Value> {
                let params: #param_ident = serde_json::from_value(params)?;
                Ok(#func_name(params, project).await)
            }
        }
    };

    expanded.into()
}
