use darling::ToTokens;
use quote::quote;
use syn::{Ident, ImplGenerics, TypeGenerics, WhereClause};

pub struct ImplModel<'a> {
    pub model_ident: &'a Ident,
    pub model_impl_generics: &'a ImplGenerics<'a>,
    pub model_type_generics: &'a TypeGenerics<'a>,
    pub model_where_clause: &'a Option<&'a WhereClause>,
}

impl<'a> ToTokens for ImplModel<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            model_ident,
            model_impl_generics,
            model_type_generics,
            model_where_clause,
        } = self;

        tokens.extend(quote! {
            impl #model_impl_generics ::typesensei::TypesenseModel for #model_ident #model_type_generics
            #model_where_clause
            {
            }
        });
    }
}
