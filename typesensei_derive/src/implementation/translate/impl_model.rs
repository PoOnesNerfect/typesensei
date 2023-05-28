use crate::implementation::{is_object, is_object_array};

use super::Field;
use darling::ToTokens;
use quote::{format_ident, quote};
use syn::{token::Brace, Ident, ImplGenerics, TypeGenerics, WhereClause};

pub struct ImplModel<'a> {
    pub ident: &'a Ident,
    pub impl_generics: &'a ImplGenerics<'a>,
    pub type_generics: &'a TypeGenerics<'a>,
    pub where_clause: &'a Option<&'a WhereClause>,
    pub fields: &'a Vec<Field>,
}

impl<'a> ToTokens for ImplModel<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            ident,
            impl_generics,
            type_generics,
            where_clause,
            fields,
        } = self;

        tokens.extend(quote! {
            impl #impl_generics ::typesensei::traits::TypesenseModel for #ident #type_generics
            #where_clause
            {
            }
        });

        tokens.extend(quote! {
            impl #impl_generics #ident #type_generics
            #where_clause
        });

        Brace::default().surround(tokens, |braces| {
            FieldsImpl::new(fields).to_tokens(braces);
        });
    }
}

struct FieldsImpl<'a> {
    fields: &'a Vec<Field>,
}

impl<'a> FieldsImpl<'a> {
    fn new(fields: &'a Vec<Field>) -> Self {
        Self { fields }
    }
}

impl<'a> ToTokens for FieldsImpl<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        for field in self.fields {
            if field.flatten {
                impl_flatten_field(field, tokens);
            } else {
                impl_field(field, tokens);
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

    let fn_name = format_ident!("set_{raw_ident}");
    let ty = if let Some(ty) = generic_type.as_ref() {
        quote! (<#ty as ::typesensei::Typesense>::Model)
    } else {
        quote! (<#ty as ::typesensei::Typesense>::Model)
    };

    tokens.extend(quote! { pub fn #fn_name <F: FnOnce(#ty) -> #ty> (mut self, f: F) -> Self });
    Brace::default().surround(tokens, |braces| {
        braces.extend(quote! {
            self. #raw_ident = f(self. #raw_ident);
            self
        });
    });
}

fn impl_field(field: &Field, tokens: &mut proc_macro2::TokenStream) {
    let Field {
        raw_ident,
        ty,
        is_vec,
        ..
    } = field;

    let fn_name = format_ident!("set_{raw_ident}");
    if is_object(field) {
        let ty = quote! (<#ty as ::typesensei::Typesense>::Model);
        tokens.extend(quote! { pub fn #fn_name <F: FnOnce(#ty) -> #ty> (mut self, f: F) -> Self });
        Brace::default().surround(tokens, |braces| {
            braces.extend(quote! {
                self. #raw_ident = f(self. #raw_ident);
                self
            });
        });
    } else if is_object_array(field) {
        if let Some(inner_type) = is_vec.as_ref() {
            let ty = quote! (Vec<<#inner_type as ::typesensei::Typesense>::Model>);
            tokens.extend(quote! { pub fn #fn_name (mut self, #raw_ident: #ty) -> Self });
            Brace::default().surround(tokens, |braces| {
                braces.extend(quote! {
                    self. #raw_ident = ::typesensei::field::set(#raw_ident);
                    self
                });
            });

            // // push fn
            // let str_ident = raw_ident.to_string();
            // let singular_ident = if str_ident.ends_with('s') {
            //     &str_ident[..str_ident.len() - 1]
            // } else {
            //     str_ident.as_str()
            // };
            // let push_name = format_ident!("push_{}", singular_ident);
            // let ty = quote! (<#inner_type as ::typesensei::Typesense>::Model);
            // tokens.extend(quote! { pub fn #push_name (mut self, #singular_ident: #ty) -> Self });
            // Brace::default().surround(tokens, |braces| {
            //     braces.extend(quote! {
            //         if let Some(items) = self. #raw_ident .inner_mut() {
            //             if let Some(new_id) = #singular_ident .id.inner() {
            //                 if let Some(item) = items.iter_mut().find(|t| t.id.inner() == new_id) {
            //                    *item = #singular_ident;
            //                 }
            //             } else {
            //                 items.push(#singular_ident);
            //             }
            //         } else {
            //             self. #raw_ident = ::typesensei::field::set(vec![#singular_ident]);
            //         }

            //         self
            //     });
            // });
        } else {
            unreachable!("vec type should be checked when parsing");
        }
    } else {
        tokens.extend(quote! { pub fn #fn_name (mut self, #raw_ident: #ty) -> Self });
        Brace::default().surround(tokens, |braces| {
            braces.extend(quote! {
                self. #raw_ident = ::typesensei::field::set(#raw_ident);
                self
            });
        });
    };
}
