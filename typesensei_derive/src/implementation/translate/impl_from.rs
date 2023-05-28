use crate::implementation::{fields_has_id, is_object, is_object_array};

use super::Field;
use darling::ToTokens;
use quote::quote;
use syn::{token::Brace, Ident, ImplGenerics, TypeGenerics, WhereClause};

pub struct ImplFrom<'a> {
    pub merged_impl_generics: &'a ImplGenerics<'a>,
    pub ident: &'a Ident,
    pub model_ident: &'a Ident,
    pub main_type_generics: &'a TypeGenerics<'a>,
    pub model_type_generics: &'a TypeGenerics<'a>,
    pub merged_where_clause: &'a Option<&'a WhereClause>,
    pub fields: &'a Vec<Field>,
}

impl<'a> ToTokens for ImplFrom<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            merged_impl_generics,
            ident,
            model_ident,
            main_type_generics,
            model_type_generics,
            merged_where_clause,
            fields,
        } = self;

        tokens.extend(quote! {
            impl #merged_impl_generics From<#ident #main_type_generics> for #model_ident #model_type_generics
            #merged_where_clause
        });

        Brace::default().surround(tokens, |braces| {
            braces.extend(quote!(fn from(other: #ident #main_type_generics) -> Self));

            Brace::default().surround(braces, |braces| {
                braces.extend(quote!(Self));

                Brace::default().surround(braces, |braces| {
                    let fields_impl = FieldImpl::new(fields);

                    fields_impl.to_tokens(braces);
                });
            });
        });
    }
}

struct FieldImpl<'a> {
    fields: &'a Vec<Field>,
}

impl<'a> FieldImpl<'a> {
    fn new(fields: &'a Vec<Field>) -> Self {
        Self { fields }
    }
}

impl<'a> ToTokens for FieldImpl<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        // inject id field if not exists
        if !fields_has_id(&self.fields) {
            tokens.extend(quote! {
                id : ::typesensei::field::unset(),
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
    let Field { raw_ident, .. } = field;

    tokens.extend(quote!(
        #raw_ident: other. #raw_ident .into(),
    ));
}

fn impl_field(field: &Field, tokens: &mut proc_macro2::TokenStream) {
    let Field {
        raw_ident,
        is_option,
        ..
    } = field;

    if is_object(field) {
        tokens.extend(quote!(
            #raw_ident: other. #raw_ident .into(),
        ));
    } else if is_object_array(field) {
        tokens.extend(quote!(
            #raw_ident: ::typesensei::field::set(other. #raw_ident .into_iter().map(From::from).collect()),
        ));
    } else {
        if is_option.is_some() {
            tokens.extend(quote! {
                #raw_ident : other. #raw_ident .into(),
            });
        } else {
            tokens.extend(quote! {
                #raw_ident : ::typesensei::field::set(other. #raw_ident),
            });
        }
    }
}
