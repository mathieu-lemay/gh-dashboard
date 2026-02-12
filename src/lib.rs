use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(Error, attributes(error))]
pub fn error_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    let _attrs = input.attrs;

    // Get the name of the struct/enum
    let name = input.ident;

    // Generate the implementation
    let expanded = quote! {
        impl ::std::error::Error for #name {}

        impl ::std::fmt::Display for #name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl<S: Into<String>> From<S> for #name {
            fn from(s: S) -> Self {
                #name(s.into())
            }
        }
    };

    TokenStream::from(expanded)
}
