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
        if !self
            .fields
            .iter()
            .any(|f| f.field == "id" || f.rename.as_ref().map(|r| r == "id").unwrap_or_default())
        {
            tokens.extend(quote! {
                id : ::typesensei::state::FieldState::not_set(),
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
        field, is_option, ..
    } = field;

    if *is_option {
        tokens.extend(quote!(
            #field: other. #field .map(|v| v.into()),
        ));
    } else {
        tokens.extend(quote!(
            #field: other. #field .into(),
        ));
    }
}

fn impl_field(field: &Field, tokens: &mut proc_macro2::TokenStream) {
    let Field { field, .. } = field;

    tokens.extend(quote! {
        #field : ::typesensei::state::FieldState::new(other. #field),
    });
}
