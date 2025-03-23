use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Attribute, Error, FnArg, Ident, ItemFn, PatType, Type};

#[proc_macro_attribute]
pub fn tool(_args: TokenStream, input: TokenStream) -> TokenStream {
    let func = parse_macro_input!(input as ItemFn);

    let func_name = &func.sig.ident;
    let tool_name = func_name.to_string();
    let tool_name_snake_case = tool_name.to_lowercase();
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
    if func.sig.inputs.len() != 1 {
        return Error::new_spanned(
            &func.sig.inputs,
            "Tool function must have exactly one parameter",
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

    let get_tool_name = format!("get_{}_tool", tool_name_snake_case);
    let get_tool_ident = Ident::new(&get_tool_name, func_name.span());
    let struct_ident = Ident::new(&format!("{}Tool", tool_name_pascal_case), func_name.span());

    let expanded = quote! {
        #func

        pub struct #struct_ident {}

        impl crate::tools::Tool for #struct_ident {
            fn get_name(&self) -> String {
                #tool_name.to_string()
            }

            fn get_schema(&self) -> nixcode_llm_sdk::tools::Tool {
                let schema = schemars::schema_for!(#param_ident);
                let parameters = serde_json::to_value(&schema).unwrap();

                let tool_name = #tool_name.to_string();
                let description = String::from(concat!("Use this tool for ", #tool_name, ". Auto generated."));

                nixcode_llm_sdk::tools::Tool::new(tool_name, description, parameters)
            }

            fn execute(&self, params: serde_json::Value) -> anyhow::Result<serde_json::Value> {
                let params: #param_ident = serde_json::from_value(params)?;
                Ok(#func_name(params))
            }
        }
    };

    expanded.into()
}

#[proc_macro_attribute]
pub fn struct_tool(_args: TokenStream, input: TokenStream) -> TokenStream {
    let func = parse_macro_input!(input as ItemFn);
    dbg!(&func);
    let expanded = quote! {
        #func
    };

    expanded.into()
}
