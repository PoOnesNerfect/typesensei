use self::translate::Translator;
use darling::Result;

mod case;
mod field;
pub use field::*;
mod parse;
pub use parse::*;
mod translate;
use syn::DeriveInput;

pub fn implement(input: DeriveInput) -> Result<proc_macro2::TokenStream> {
    let translator = Translator::from_derived(&input)?;
    let implementation = translator.translate();

    Ok(implementation)
}
