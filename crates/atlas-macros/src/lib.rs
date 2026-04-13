//! Atlas Derive Macros
//! 
//! Procedural macros for declarative entity definitions.

use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

/// Derive macro for entity definitions
#[proc_macro_derive(Entity, attributes(entity, field))]
pub fn entity_derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    
    let name = &input.ident;
    
    let expanded = quote! {
        impl #name {
            /// Generate entity definition from struct
            pub fn entity_definition() -> atlas_shared::EntityDefinition {
                unimplemented!("Use declarative approach via JSON schema")
            }
        }
    };
    
    TokenStream::from(expanded)
}
