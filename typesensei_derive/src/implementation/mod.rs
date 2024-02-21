use self::implement::Implementor;
use darling::Result;
use syn::DeriveInput;

pub mod case;
pub mod field;
pub use field::*;
pub mod parse;
pub use parse::*;
pub mod implement;

pub fn impl_typesense(input: DeriveInput) -> Result<proc_macro2::TokenStream> {
    let implementer = Implementor::from_derived(&input)?;
    let implementation = implementer.impl_typesense();

    Ok(implementation)
}

pub fn impl_partial(input: DeriveInput) -> Result<proc_macro2::TokenStream> {
    let implementer = Implementor::from_derived(&input)?;
    let implementation = implementer.impl_partial();

    Ok(implementation)
}
