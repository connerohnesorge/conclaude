use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Lit, Meta};

/// Derives a `FieldList` trait implementation that provides field names as a vector.
///
/// This macro extracts field names from structs and respects `#[serde(rename = "...")]`
/// attributes to match YAML keys. For example:
///
/// ```rust,ignore
/// #[derive(FieldList)]
/// struct Config {
///     #[serde(rename = "infiniteMessage")]
///     infinite_message: bool,
///     timeout: u64,
/// }
/// ```
///
/// Will generate field names: `["infiniteMessage", "timeout"]`
#[proc_macro_derive(FieldList)]
pub fn derive_field_list(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    // Extract field names from the struct
    let field_names = match &input.data {
        Data::Struct(data) => {
            match &data.fields {
                Fields::Named(fields) => {
                    fields
                        .named
                        .iter()
                        .map(|f| {
                            let field_name = f.ident.as_ref().unwrap();

                            // Check for serde rename attribute
                            let renamed = extract_serde_rename(&f.attrs);

                            match renamed {
                                Some(rename) => rename,
                                None => field_name.to_string(),
                            }
                        })
                        .collect::<Vec<_>>()
                }
                _ => {
                    return syn::Error::new_spanned(
                        &input.ident,
                        "FieldList can only be derived for structs with named fields",
                    )
                    .to_compile_error()
                    .into();
                }
            }
        }
        _ => {
            return syn::Error::new_spanned(
                &input.ident,
                "FieldList can only be derived for structs",
            )
            .to_compile_error()
            .into();
        }
    };

    // Generate the trait implementation
    let expanded = quote! {
        impl #name {
            /// Returns a vector of field names for this struct.
            /// Field names respect `#[serde(rename = "...")]` attributes.
            pub fn field_names() -> Vec<&'static str> {
                vec![#(#field_names),*]
            }
        }
    };

    TokenStream::from(expanded)
}

/// Extracts the `rename` value from `#[serde(rename = "...")]` attributes
fn extract_serde_rename(attrs: &[syn::Attribute]) -> Option<String> {
    for attr in attrs {
        // Check if this is a serde attribute
        if !attr.path().is_ident("serde") {
            continue;
        }

        // Parse the attribute meta
        if let Ok(meta_list) = attr.meta.require_list() {
            // Parse the tokens as a comma-separated list of meta items
            let nested = meta_list.parse_args_with(
                syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated,
            );

            if let Ok(nested_metas) = nested {
                for meta in nested_metas {
                    if let Meta::NameValue(nv) = meta {
                        if nv.path.is_ident("rename") {
                            if let syn::Expr::Lit(expr_lit) = &nv.value {
                                if let Lit::Str(lit_str) = &expr_lit.lit {
                                    return Some(lit_str.value());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    None
}
