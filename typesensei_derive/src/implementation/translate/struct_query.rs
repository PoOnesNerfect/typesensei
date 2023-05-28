use crate::implementation::{fields_has_id, is_object};

use super::Field;
use darling::ToTokens;
use quote::quote;
use syn::{token::Brace, Ident, ImplGenerics, Token, Type, WhereClause};

pub struct StructQuery<'a> {
    pub ident: &'a Ident,
    pub impl_generics: &'a ImplGenerics<'a>,
    pub where_clause: &'a Option<&'a WhereClause>,
    pub fields: &'a Vec<Field>,
    pub id_type: &'a Type,
}

impl<'a> ToTokens for StructQuery<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            ident,
            impl_generics,
            where_clause,
            fields,
            id_type,
        } = self;

        let hash: Token![#] = Default::default();

        tokens.extend(quote! {
            #hash [derive(Debug)]
            pub struct #ident #impl_generics #where_clause
        });

        Brace::default().surround(tokens, |braces| {
            FieldsImpl::new(fields, id_type).to_tokens(braces);
        });
    }
}

struct FieldsImpl<'a> {
    fields: &'a Vec<Field>,
    id_type: &'a Type,
}

impl<'a> FieldsImpl<'a> {
    fn new(fields: &'a Vec<Field>, id_type: &'a Type) -> Self {
        Self { fields, id_type }
    }
}

impl<'a> ToTokens for FieldsImpl<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        // inject id field if not exists
        if !fields_has_id(&self.fields) {
            let id_type = self.id_type;
            tokens.extend(quote! {
                pub id : ::typesensei::query::QueryState<#id_type>,
            });
        }

        for field in self.fields {
            if field.flatten {
                impl_flatten_field(&field, tokens);
            } else {
                impl_field(&field, tokens);
            }
        }
    }
}

fn impl_flatten_field(field: &Field, tokens: &mut proc_macro2::TokenStream) {
    let Field {
        raw_ident,
        generic_type,
        ty,
        ..
    } = field;

    if let Some(ty) = generic_type.as_ref() {
        tokens.extend(quote! {
            pub #raw_ident : <#ty as ::typesensei::Typesense>::Query,
        });
    } else {
        tokens.extend(quote! {
            pub #raw_ident : <#ty as ::typesensei::Typesense>::Query,
        });
    }
}

fn impl_field(field: &Field, tokens: &mut proc_macro2::TokenStream) {
    let Field { raw_ident, ty, .. } = field;

    if is_object(field) {
        tokens.extend(quote! {
            pub #raw_ident : <#ty as ::typesensei::Typesense>::Query,
        });
    } else {
        tokens.extend(quote! {
            pub #raw_ident : ::typesensei::query::QueryState<#ty>,
        });
    }
}
