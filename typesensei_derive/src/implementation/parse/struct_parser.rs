use darling::ToTokens;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    punctuated::Punctuated, GenericParam, Generics, Ident, Path, Type, WhereClause, WherePredicate,
};

use crate::implementation::Field;

pub struct StructParser {
    pub ident: Ident,
    pub generics: Generics,
    pub fields: Vec<Field>,
    pub associated_type: Type,
    pub merged_generics: Generics,
    pub struct_where_clause: Option<WhereClause>,
}

impl StructParser {
    pub fn new(
        serde: &Path,
        ident: &Ident,
        custom_name: &Option<String>,
        suffix: Ident,
        generics: &Generics,
        fields: &Vec<Field>,
    ) -> Self {
        let mut generics = generics.clone();
        let mut fields = fields.clone();

        let ident = custom_name
            .as_ref()
            .map(|name| format_ident!("{name}"))
            .unwrap_or_else(|| format_ident!("{ident}{suffix}"));
        let associated_type = get_associated_type(&ident, &suffix, &generics, &fields);

        let trait_ident = format_ident!("Typesense{suffix}");
        let merged_generics =
            get_merged_generics(&generics, &fields, &suffix, &trait_ident, &serde);

        add_suffix_to_generics(&suffix, &mut generics, &mut fields);
        let struct_where_clause = generics.where_clause.clone();

        add_generic_bounds(
            &mut generics,
            &fields,
            quote!(::typesensei:: #trait_ident),
            &serde,
        );

        Self {
            ident,
            generics,
            fields,
            associated_type,
            merged_generics,
            struct_where_clause,
        }
    }
}

fn get_associated_type(
    ident: &Ident,
    suffix: &Ident,
    generics: &Generics,
    fields: &Vec<Field>,
) -> Type {
    let mut types = quote!(#ident);

    if generics.params.is_empty() {
        return syn::parse_quote!(#types);
    }

    types.extend(quote!(<));

    for generic in generics.params.iter() {
        if let GenericParam::Type(ty) = generic {
            let ident = &ty.ident;

            for field in fields {
                if let Some(ty) = field.generic_type.as_ref() {
                    if ident == ty {
                        if field.flatten {
                            types.extend(quote!(<#ty as ::typesensei::Typesense>:: #suffix , ));
                        } else {
                            types.extend(quote!(#ty , ));
                        }
                    }
                }
            }
        } else {
            generic.to_tokens(&mut types);
        }
    }

    types.extend(quote!(>));

    syn::parse_quote!(#types)
}

fn add_suffix_to_generics(suffix: &Ident, generics: &mut Generics, fields: &mut Vec<Field>) {
    let mut generic_types = generics
        .params
        .iter_mut()
        .filter_map(|param| {
            if let GenericParam::Type(ty) = param {
                Some((&mut ty.ident, &mut ty.bounds))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    for field in fields {
        if field.flatten {
            if let Some(ty) = &mut field.generic_type {
                let new_ty = format_ident!("{ty}{suffix}");
                if let Some((ty, bounds)) = generic_types.iter_mut().find(|(t, _)| (**t).eq(ty)) {
                    **ty = new_ty.clone();
                    bounds.clear();
                }
                *ty = new_ty;
            }
        }
    }
}

fn add_generic_bounds(
    generics: &mut Generics,
    fields: &Vec<Field>,
    impl_trait: TokenStream,
    serde: &Path,
) {
    let mut bounds: Vec<WherePredicate> = Vec::new();

    for field in fields {
        if let Some(ty) = field.generic_type.as_ref() {
            if field.flatten {
                bounds.push(syn::parse_quote!(#ty : #impl_trait));
            } else {
                bounds.push(syn::parse_quote!(for<'de> #ty : ::std::fmt::Debug + Default + #serde ::Serialize + #serde ::Deserialize<'de> + ::typesensei::traits::TypesenseField));
                bounds.push(syn::parse_quote!(<#ty as ::typesensei::traits::TypesenseField>::Type : 'static));
            }
        }
    }

    if bounds.is_empty() {
        return;
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

        return;
    }
}

fn get_merged_generics(
    generics: &Generics,
    fields: &Vec<Field>,
    suffix: &Ident,
    trait_ident: &Ident,
    serde: &Path,
) -> Generics {
    let mut generics = generics.clone();

    let mut bounds: Vec<WherePredicate> = Vec::new();

    for field in fields {
        if let Some(ty) = field.generic_type.as_ref() {
            if field.flatten {
                let suffixed = format_ident!("{ty}{suffix}");
                generics.params.push(syn::parse_quote!(#suffixed));

                bounds.push(syn::parse_quote!(#ty : ::typesensei::Typesense));

                bounds.push(syn::parse_quote!(#suffixed : ::typesensei:: #trait_ident + From<#ty> + Default));
            } else {
                bounds.push(syn::parse_quote!(for<'de> #ty : ::std::fmt::Debug + Default + #serde ::Serialize + #serde ::Deserialize<'de> + ::typesensei::traits::TypesenseField));
                bounds.push(syn::parse_quote!(<#ty as ::typesensei::traits::TypesenseField>::Type : ::std::fmt::Display));
            }
        }
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
