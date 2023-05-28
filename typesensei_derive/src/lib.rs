use proc_macro::{self, TokenStream};
use proc_macro_error::proc_macro_error;
use quote::ToTokens;
use syn::{parse_macro_input, DeriveInput};

mod implementation;

#[proc_macro_derive(Typesense, attributes(typesensei))]
#[proc_macro_error]
pub fn typesense(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match implementation::implement(input) {
        Ok(typesense) => typesense.to_token_stream().into(),
        Err(e) => e.write_errors().into(),
    }
}
