use darling::Result;
use quote::ToTokens;

mod case;
mod field;
pub use field::*;
mod parse;
pub use parse::*;
mod translate;
use syn::DeriveInput;
pub use translate::*;

mod impl_from;
pub use impl_from::*;
mod impl_model;
pub use impl_model::*;
mod impl_typesense;
pub use impl_typesense::*;
mod struct_model;
pub use struct_model::*;

pub fn implement(input: DeriveInput) -> Result<proc_macro2::TokenStream> {
    let translator = Translator::from_derived(&input)?;
    let implementation = translator.translate();

    Ok(implementation)
}

pub struct Implementation<'a> {
    impl_typesense: ImplTypesense<'a>,
    struct_model: StructModel<'a>,
    impl_model: ImplModel<'a>,
    impl_from: ImplFrom<'a>,
}

impl<'a> ToTokens for Implementation<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            impl_typesense,
            struct_model,
            impl_model,
            impl_from,
        } = self;

        impl_typesense.to_tokens(tokens);
        struct_model.to_tokens(tokens);
        impl_model.to_tokens(tokens);
        impl_from.to_tokens(tokens);
    }
}
