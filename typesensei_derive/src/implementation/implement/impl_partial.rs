use quote::{format_ident, quote, ToTokens};
use syn::{
    punctuated::Punctuated,
    token::{Brace, Paren},
    GenericParam, Generics, Ident, Path, Token, Type, WhereClause, WherePredicate,
};

use crate::implementation::{case::RenameRule, Field};

pub struct ImplPartial<'a> {
    pub vis: &'a syn::Visibility,
    pub ident: &'a Ident,
    pub partial_ident: Ident,
    pub generics: Generics,
    pub fields: &'a Vec<Field>,
    pub case: &'a RenameRule,
    pub serde: &'a Path,
    pub rename_all: &'a Option<String>,
}

impl ToTokens for ImplPartial<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.impl_struct(tokens);
        self.impl_partial(tokens);
        self.impl_from(tokens);
        self.impl_try_from(tokens);
    }
}

impl<'a> ImplPartial<'a> {
    pub fn new(
        vis: &'a syn::Visibility,
        ident: &'a Ident,
        generics: &'a Generics,
        fields: &'a Vec<Field>,
        case: &'a RenameRule,
        serde: &'a Path,
        rename_all: &'a Option<String>,
    ) -> Self {
        Self {
            vis,
            ident,
            partial_ident: format_ident!("{}Partial", ident),
            generics: get_partial_generics(&generics, fields),
            fields,
            serde,
            case,
            rename_all,
        }
    }

    fn impl_struct(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            vis,
            partial_ident,
            generics,
            fields,
            rename_all,
            serde,
            ..
        } = self;

        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let hash: Token![#] = Default::default();

        tokens.extend(quote! {
            #hash [derive(Debug, Default, #serde ::Serialize, #serde ::Deserialize)]
            #rename_all
            #vis struct #partial_ident #impl_generics #where_clause
        });

        Brace::default().surround(tokens, |braces| {
            for field in fields.iter() {
                let rename = field
                    .rename
                    .as_ref()
                    .map(|r| quote!(#hash [serde(rename = #r)]));

                braces.extend(quote!(#hash [serde(skip_serializing_if = "Option::is_none")]));

                let i = &field.raw_ident;
                let t = &field.ty;

                // if type is Option, then it is already an Option
                if is_option(t) {
                    braces.extend(quote! {
                        #rename
                        #i: <#t as ::typesensei::Partial>::Partial,
                    })
                } else {
                    braces.extend(quote! {
                        #rename
                        #i: Option<<#t as ::typesensei::Partial>::Partial>,
                    })
                }
            }
        });

        tokens.extend(quote! {
            impl #impl_generics #partial_ident #ty_generics #where_clause
        });

        Brace::default().surround(tokens, |braces| {
            for field in fields.iter() {
                let i = &field.raw_ident;
                let t = &field.ty;

                let ty = quote! (<#t as ::typesensei::Partial>::Partial);

                let with_fn = format_ident!("with_{}", i);
                braces.extend(quote! { pub fn #with_fn (mut self, #i: #ty) -> Self });
                Brace::default().surround(braces, |braces| {
                    if is_option(t) {
                        braces.extend(quote! {
                            self. #i = #i;
                            self
                        });
                    } else {
                        braces.extend(quote! {
                            self. #i = Some(#i);
                            self
                        });
                    }
                });

                let without_fn = format_ident!("without_{}", i);
                braces.extend(quote! { pub fn #without_fn (mut self) -> Self });
                Brace::default().surround(braces, |braces| {
                    braces.extend(quote! {
                        self. #i = None;
                        self
                    });
                });

                let set_fn = format_ident!("set_{}", i);
                braces.extend(quote! { pub fn #set_fn (&mut self, #i: #ty) });
                Brace::default().surround(braces, |braces| {
                    if is_option(t) {
                        braces.extend(quote! {
                            self. #i = #i;
                        });
                    } else {
                        braces.extend(quote! {
                            self. #i = Some(#i);
                        });
                    }
                });

                let unset_fn = format_ident!("unset_{}", i);
                braces.extend(quote! { pub fn #unset_fn (&mut self) });
                Brace::default().surround(braces, |braces| {
                    braces.extend(quote! {
                        self. #i = None;
                    });
                });
            }
        });
    }

    fn impl_partial(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            ident,
            partial_ident,
            generics,
            fields,
            ..
        } = self;

        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        tokens.extend(quote! {
            impl #impl_generics ::typesensei::partial::Partial for #ident #ty_generics
            #where_clause
        });

        Brace::default().surround(tokens, |braces| {
            braces.extend(quote!(type Partial = #partial_ident #ty_generics;));

            braces.extend(quote!(fn into_partial(self) -> Self::Partial));

            Brace::default().surround(braces, |braces| {
                braces.extend(quote!(#partial_ident));

                Brace::default().surround(braces, |braces| {
                    for field in fields.iter() {
                        let i = &field.raw_ident;

                        if is_option(&field.ty) {
                            braces.extend(quote!(#i: self. #i .map(|v| v.into_partial()),));
                        } else {
                            braces.extend(quote!(#i: Some(self. #i .into_partial()),));
                        }
                    }
                });
            });

            braces
                .extend(quote!(fn from_partial(partial: #partial_ident #ty_generics) -> Result<Self, ::typesensei::partial::TryFromPartialError>));

            Brace::default().surround(braces, |braces| {
                braces.extend(quote!(Ok));

                Paren::default().surround(braces, |parens| {
                    parens.extend(quote!(Self));

                    Brace::default().surround(parens, |braces| {
                        for field in fields.iter() {
                            let i = &field.raw_ident;

                            if is_option(&field.ty) {
                                braces.extend(quote!(#i: <_>::from_partial(partial. #i)?,));
                            } else {
                                braces.extend(quote!(#i: <_>::from_partial(partial. #i .ok_or_else(|| ::typesensei::partial::TryFromPartialError {
                                    type_name: core::any::type_name::<Self>(),
                                    field: stringify!(#i),
                                })?)?,));
                            }
                        }
                    });
                });
            });
        });
    }

    fn impl_from(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            ident,
            partial_ident,
            generics,
            ..
        } = self;

        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        tokens.extend(quote! {
            impl #impl_generics From<#ident #ty_generics> for #partial_ident #ty_generics
            #where_clause
        });

        Brace::default().surround(tokens, |braces| {
            braces.extend(quote!(fn from(val: #ident #ty_generics) -> Self));

            Brace::default().surround(braces, |braces| {
                braces.extend(quote!(val.into_partial()));
            });
        });
    }

    fn impl_try_from(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            ident,
            partial_ident,
            generics,
            ..
        } = self;

        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        tokens.extend(quote! {
            impl #impl_generics std::convert::TryFrom<#partial_ident #ty_generics> for #ident #ty_generics
            #where_clause
        });

        Brace::default().surround(tokens, |braces| {
            braces.extend(quote!(
                type Error = ::typesensei::partial::TryFromPartialError;
            ));

            braces.extend(
                quote!(fn try_from(val: #partial_ident #ty_generics) -> Result<Self, Self::Error>),
            );

            Brace::default().surround(braces, |braces| {
                braces.extend(quote!(Self::from_partial(val)));
            });
        });
    }
}

fn get_partial_generics(generics: &Generics, fields: &Vec<Field>) -> Generics {
    let mut generics = generics.clone();

    let mut bounds: Vec<WherePredicate> = Vec::new();

    for field in fields {
        let ty = &field.ty;

        let Some(ty) = contains_ty(&generics, ty) else {
            continue;
        };

        bounds.push(syn::parse_quote!(#ty : ::typesensei::Partial));
    }

    if bounds.is_empty() {
        return generics;
    }

    if let Some(where_clause) = generics.where_clause.as_mut() {
        where_clause.predicates.extend(bounds);
    } else {
        let mut predicates = Punctuated::new();
        predicates.extend(bounds);

        generics.where_clause.replace(WhereClause {
            where_token: Default::default(),
            predicates,
        });
    }

    generics
}

fn is_option(ty: &Type) -> bool {
    if let Type::Path(ty) = ty {
        if let Some(segment) = ty.path.segments.last() {
            if segment.ident == "Option" {
                return true;
            }
        }
    }

    false
}

fn contains_ty(generics: &Generics, ty: &Type) -> Option<Ident> {
    for param in &generics.params {
        if let GenericParam::Type(t) = param {
            if let Type::Path(ty) = ty {
                if let Some(segment) = ty.path.segments.last() {
                    if segment.ident == "Option" {
                        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                            if let Some(arg) = args.args.first() {
                                if let syn::GenericArgument::Type(ty) = arg {
                                    if let Type::Path(ty) = ty {
                                        if let Some(ty) = ty.path.get_ident() {
                                            if &t.ident == ty {
                                                return Some(t.ident.clone());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    if let Some(ty) = ty.path.get_ident() {
                        if &t.ident == ty {
                            return Some(t.ident.clone());
                        }
                    }
                }
            }
        }
    }

    None
}
