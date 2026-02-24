use proc_macro::TokenStream;
use heck::ToSnakeCase;
use quote::{format_ident, quote};
use syn::parse::Parser;
use syn::{spanned::Spanned, Expr, ExprLit, Item, Lit, Meta};

#[proc_macro_attribute]
pub fn export_schema(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as Item);

    let mut file_name_override = None;
    let parser = syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated;
    let args = match parser.parse(attr) {
        Ok(args) => args,
        Err(error) => return error.to_compile_error().into(),
    };

    for argument in args {
        match argument {
            Meta::NameValue(name_value) if name_value.path.is_ident("file_name") => {
                let Expr::Lit(ExprLit { lit: Lit::Str(value), .. }) = name_value.value else {
                    return syn::Error::new(
                        name_value.value.span(),
                        "expected string literal for `file_name`",
                    )
                    .to_compile_error()
                    .into();
                };

                file_name_override = Some(value);
            }
            _ => {
                return syn::Error::new(
                    argument.span(),
                    "unsupported attribute argument, expected `file_name = \"...\"`",
                )
                .to_compile_error()
                .into();
            }
        }
    }

    let (type_ident, has_generics) = match &input {
        Item::Struct(item_struct) => (&item_struct.ident, !item_struct.generics.params.is_empty()),
        Item::Enum(item_enum) => (&item_enum.ident, !item_enum.generics.params.is_empty()),
        _ => {
            return syn::Error::new(
                input.span(),
                "`#[export_schema]` is only supported on structs and enums",
            )
            .to_compile_error()
            .into();
        }
    };

    if has_generics {
        return syn::Error::new(
            type_ident.span(),
            "`#[export_schema]` does not support generic types",
        )
        .to_compile_error()
        .into();
    }

    let snake_case_type_name = type_ident.to_string().to_snake_case();
    let schema_fn_ident = format_ident!("__worth_schema_json_for_{}", snake_case_type_name);
    let file_name = if let Some(file_name) = file_name_override {
        quote!(#file_name)
    } else {
        quote!(concat!(stringify!(#type_ident), ".schema.json"))
    };

    quote! {
        #input

        const _: () = {
            fn #schema_fn_ident() -> ::anyhow::Result<::std::string::String> {
                let generator = ::schemars::generate::SchemaSettings::draft2020_12().into_generator();
                let schema = generator.into_root_schema_for::<#type_ident>();
                let json = ::serde_json::to_string_pretty(&schema)
                    .map_err(|error| ::anyhow::anyhow!("serialize schema as JSON: {error}"))?;
                Ok(format!("{json}\n"))
            }

            ::inventory::submit! {
                ::worth_lib::contracts::schema_export::SchemaExport {
                    type_name: stringify!(#type_ident),
                    file_name: #file_name,
                    schema_json: #schema_fn_ident,
                }
            }
        };
    }
    .into()
}
