use super::Field;
use darling::ToTokens;
use quote::quote;
use syn::{token::Brace, Ident, ImplGenerics, Token, Type, WhereClause};

pub struct StructQuery<'a> {
    pub ident: &'a Ident,
    pub impl_generics: &'a ImplGenerics<'a>,
    pub where_clause: &'a Option<&'a WhereClause>,
    pub fields: &'a Vec<Field>,
    pub id: &'a Type,
}

impl<'a> ToTokens for StructQuery<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            ident,
            impl_generics,
            where_clause,
            fields,
            id,
        } = self;

        let hash: Token![#] = Default::default();

        tokens.extend(quote! {
            #hash [derive(Debug)]
            pub struct #ident #impl_generics #where_clause
        });

        Brace::default().surround(tokens, |braces| {
            FieldsImpl::new(fields, id).to_tokens(braces);
        });
    }
}

struct FieldsImpl<'a> {
    fields: &'a Vec<Field>,
    id: &'a Type,
}

impl<'a> FieldsImpl<'a> {
    fn new(fields: &'a Vec<Field>, id: &'a Type) -> Self {
        Self { fields, id }
    }
}

impl<'a> ToTokens for FieldsImpl<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        // inject id field if not exists
        if !self
            .fields
            .iter()
            .any(|f| f.field == "id" || f.rename.as_ref().map(|r| r == "id").unwrap_or_default())
        {
            let id = self.id;
            tokens.extend(quote! {
                pub id : ::typesensei::state::QueryState<#id>,
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
        field,
        is_option,
        generic_type,
        ty,
        ..
    } = field;

    if let Some(ty) = generic_type.as_ref() {
        if *is_option {
            tokens.extend(quote! {
                pub #field : Option<#ty>,
            });
        } else {
            tokens.extend(quote! {
                pub #field : #ty,
            });
        }
    } else {
        if *is_option {
            tokens.extend(quote! {
                pub #field : Option<<#ty as ::typesensei::Typesense>::Query>,
            });
        } else {
            tokens.extend(quote! {
                pub #field : <#ty as ::typesensei::Typesense>::Query,
            });
        }
    }
}

fn impl_field(field: &Field, tokens: &mut proc_macro2::TokenStream) {
    let Field { field, ty, .. } = field;

    tokens.extend(quote! {
        pub #field : ::typesensei::state::QueryState<#ty>,
    });
}
