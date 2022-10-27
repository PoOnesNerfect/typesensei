use darling::ToTokens;
use quote::quote;
use syn::{Ident, ImplGenerics, TypeGenerics, WhereClause};

pub struct ImplModel<'a> {
    pub ident: &'a Ident,
    pub impl_generics: &'a ImplGenerics<'a>,
    pub type_generics: &'a TypeGenerics<'a>,
    pub where_clause: &'a Option<&'a WhereClause>,
}

impl<'a> ToTokens for ImplModel<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            ident,
            impl_generics,
            type_generics,
            where_clause,
        } = self;

        tokens.extend(quote! {
            impl #impl_generics ::typesensei::traits::TypesenseModel for #ident #type_generics
            #where_clause
            {
            }
        });
    }
}
