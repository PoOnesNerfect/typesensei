use crate::implementation::{fields_has_id, is_object, is_object_array};

use super::Field;
use darling::ToTokens;
use quote::quote;
use syn::{token::Brace, Ident, ImplGenerics, Path, Token, Type, WhereClause};

pub struct StructModel<'a> {
    pub serde: &'a Path,
    pub ident: &'a Ident,
    pub impl_generics: &'a ImplGenerics<'a>,
    pub where_clause: &'a Option<WhereClause>,
    pub fields: &'a Vec<Field>,
    pub rename_all: &'a Option<String>,
    pub id_type: &'a Type,
}

impl<'a> ToTokens for StructModel<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            serde,
            ident,
            impl_generics,
            where_clause,
            fields,
            rename_all,
            id_type,
        } = self;

        let hash: Token![#] = Default::default();

        tokens.extend(quote! {
            #hash [derive(Debug, Default, #serde ::Serialize, #serde ::Deserialize)]
            #rename_all
            pub struct #ident #impl_generics #where_clause
        });

        Brace::default().surround(tokens, |braces| {
            FieldsImpl::new(fields, &hash, id_type).to_tokens(braces);
        });
    }
}

struct FieldsImpl<'a> {
    fields: &'a Vec<Field>,
    hash: &'a Token![#],
    id_type: &'a Type,
}

impl<'a> FieldsImpl<'a> {
    fn new(fields: &'a Vec<Field>, hash: &'a Token![#], id_type: &'a Type) -> Self {
        Self {
            fields,
            hash,
            id_type,
        }
    }
}

impl<'a> ToTokens for FieldsImpl<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        // inject id field if not exists
        if !fields_has_id(&self.fields) {
            let hash = self.hash;
            let id_type = self.id_type;
            tokens.extend(quote! {
                #hash [serde(skip_serializing_if = "::typesensei::field::FieldState::is_unset")]
                pub id : ::typesensei::field::FieldState<#id_type>,
            });
        }

        for field in self.fields {
            if field.flatten {
                impl_flatten_field(&field, self.hash, tokens);
            } else {
                impl_field(&field, self.hash, tokens);
            }
        }
    }
}

fn impl_flatten_field(field: &Field, hash: &Token![#], tokens: &mut proc_macro2::TokenStream) {
    let Field {
        raw_ident,
        generic_type,
        ty,
        ..
    } = field;

    if let Some(ty) = generic_type.as_ref() {
        tokens.extend(quote! {
            #hash [serde(flatten)]
            pub #raw_ident : <#ty as ::typesensei::Typesense>::Model,
        });
    } else {
        tokens.extend(quote! {
            #hash [serde(flatten)]
            pub #raw_ident : <#ty as ::typesensei::Typesense>::Model,
        });
    }
}

fn impl_field(field: &Field, hash: &Token![#], tokens: &mut proc_macro2::TokenStream) {
    let Field {
        raw_ident,
        ty,
        rename,
        is_option,
        is_vec,
        ..
    } = field;

    if is_object(field) {
        let rename = rename.as_ref().map(|r| quote!(#hash [serde(rename = #r)]));

        tokens.extend(quote! {
            #rename
            pub #raw_ident : <#ty as ::typesensei::Typesense>::Model,
        });
    } else if is_object_array(field) {
        if let Some(inner_type) = is_vec.as_ref() {
            let rename = rename.as_ref().map(|r| quote!(#hash [serde(rename = #r)]));

            tokens.extend(quote! {
                #rename
                pub #raw_ident : ::typesensei::field::FieldState<Vec<<#inner_type as ::typesensei::Typesense>::Model>>,
            });
        } else {
            unreachable!("vec type should be checked when parsing");
        }
    } else {
        if is_option.is_some() {
            tokens.extend(quote! (#hash [serde(skip_serializing_if = "::typesensei::field::FieldState::is_inner_option_none")]));
        } else {
            tokens.extend(
                quote!(#hash [serde(skip_serializing_if = "::typesensei::field::FieldState::is_unset")]),
            );
        }

        let rename = rename.as_ref().map(|r| quote!(#hash [serde(rename = #r)]));

        tokens.extend(quote! {
            #rename
            pub #raw_ident : ::typesensei::field::FieldState<#ty>,
        });
    }
}
