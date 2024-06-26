use super::{super::case::RenameRule, Field};
use crate::implementation::{ts, SymbolsToIndex, TypesenseFields};
use darling::ToTokens;
use quote::quote;
use syn::{
    token::{Brace, Bracket, Paren},
    Generics, Ident, Type,
};

pub struct ImplTypesense<'a> {
    pub ident: &'a Ident,
    pub generics: &'a Generics,
    pub id_type: &'a Type,
    pub enable_nested_fields: bool,
    pub fields: &'a Vec<Field>,
    pub case: &'a RenameRule,
    pub extra_fields: &'a Option<TypesenseFields>,
    pub symbols_to_index: &'a Option<SymbolsToIndex>,
}

impl<'a> ToTokens for ImplTypesense<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            ident,
            generics,
            id_type: _,
            enable_nested_fields,
            fields,
            case,
            extra_fields,
            symbols_to_index,
        } = self;

        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let fields_impl = FieldImpl::new(fields, case /*, id_type */);
        let extra_fields_impl = ExtraFieldImpl::new(extra_fields);

        let enable_nested_fields = enable_nested_fields.then(|| quote!(.enable_nested_fields()));

        let symbols_to_index_impl = symbols_to_index.as_ref().map(|t| {
            let symbols = &t.0;
            let mut tokens = quote!(.symbols_to_index);

            Paren::default().surround(&mut tokens, |parens| {
                Bracket::default().surround(parens, |brackets| {
                    for symbol in symbols {
                        brackets.extend(quote!(#symbol ,));
                    }
                });
            });

            tokens
        });

        tokens.extend(quote! {
            impl #impl_generics ::typesensei::Typesense for #ident #ty_generics
            #where_clause
            {
                fn partial() -> Self::Partial {
                    Default::default()
                }

                fn schema<'a>(collection_name: &'a str) -> ::typesensei::schema::CollectionSchema<'a> {
                    use ::typesensei::{Typesense, TypesenseField};
                    ::typesensei::schema::CollectionSchema::new(collection_name)
                    #enable_nested_fields
                    #fields_impl
                    #extra_fields_impl
                    #symbols_to_index_impl
                }
            }
        });
    }
}

struct FieldImpl<'a> {
    // id_type: &'a Type,
    fields: &'a Vec<Field>,
    case: &'a RenameRule,
}

impl<'a> FieldImpl<'a> {
    fn new(fields: &'a Vec<Field>, case: &'a RenameRule /*, id_type: &'a Type */) -> Self {
        Self {
            fields,
            case,
            // id_type,
        }
    }
}

impl<'a> ToTokens for FieldImpl<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        for field in self.fields {
            if field.flatten {
                impl_flatten_field(&field, tokens);
            } else if field.schema {
                impl_schema_field(&field, &self.case, tokens);
            } else {
                impl_field(&field, &self.case, tokens);
            }
        }
    }
}

fn impl_field(field: &Field, case: &RenameRule, tokens: &mut proc_macro2::TokenStream) {
    let Field {
        raw_ident,
        ty,
        index,
        sort,
        facet,
        is_option,
        default_sorting_field,
        rename,
        custom_type,
        optional,
        ..
    } = field;

    // if field_is_id(field) {
    //     return;
    // }

    let name = if let Some(name) = rename {
        name.to_owned()
    } else {
        case.apply_to_field(&raw_ident.to_string())
    };

    let ty = custom_type
        .as_ref()
        .map(|t| quote!(#t))
        .unwrap_or_else(|| quote!(< #ty >::TYPE));

    let should_be_optional = index.map(|b| !b).unwrap_or(false);
    let optional = optional.unwrap_or(false) || is_option.is_some() || should_be_optional;

    impl_field_inner(
        tokens,
        &name,
        ty,
        index,
        facet,
        sort,
        optional,
        *default_sorting_field,
    );
}

fn impl_schema_field(field: &Field, case: &RenameRule, tokens: &mut proc_macro2::TokenStream) {
    let Field {
        raw_ident,
        rename,
        is_option,
        ty,
        ..
    } = field;

    let name = if let Some(name) = rename {
        name.to_owned()
    } else {
        case.apply_to_field(&raw_ident.to_string())
    };

    let ty = is_option.as_ref().map(|t| t).unwrap_or_else(|| ty);

    tokens.extend(quote!(.schema_field));

    Paren::default().surround(tokens, |parens| {
        parens.extend(quote!(#name, <#ty as Typesense>::schema("")));
    });
}

fn impl_flatten_field(field: &Field, tokens: &mut proc_macro2::TokenStream) {
    let Field {
        ty,
        index,
        sort,
        facet,
        rename,
        generic_type,
        ..
    } = field;

    tokens.extend(quote!(.extend));

    Paren::default().surround(tokens, |parens| {
        Brace::default().surround(parens, |braces| {
            if let Some(ty) = generic_type.as_ref() {
                braces.extend(quote! {
                    let mut schema = <#ty>::schema("");
                });
            } else {
                braces.extend(quote! {
                    let mut schema = <#ty>::schema("");
                });
            }

            let set_facet = facet.as_ref().map(|f| {
                quote! {
                    field.facet = Some(#f);
                }
            });

            let set_index = index.as_ref().map(|i| {
                quote! {
                    field.index = Some(#i);
                }
            });

            let set_sort = sort.as_ref().map(|i| {
                quote! {
                    field.sort = Some(#i);
                }
            });

            let set_rename = rename.as_ref().map(|r| {
                quote! {
                    field.name = #r;
                }
            });

            if set_facet.is_some()
                || set_index.is_some()
                || set_sort.is_some()
                || set_rename.is_some()
            {
                braces.extend(quote! {
                    for field in &mut schema.fields {
                        #set_facet
                        #set_index
                        #set_sort
                        #set_rename
                    }
                });
            }

            braces.extend(quote! {
                schema
            });
        });
    });
}

struct ExtraFieldImpl<'a> {
    fields: &'a Option<TypesenseFields>,
}

impl<'a> ExtraFieldImpl<'a> {
    fn new(fields: &'a Option<TypesenseFields>) -> Self {
        Self { fields }
    }
}

impl<'a> ToTokens for ExtraFieldImpl<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if let Some(fields) = self.fields.as_ref() {
            for field in &fields.0 {
                let ts::Field {
                    name,
                    ty,
                    facet,
                    index,
                    sort,
                    optional,
                    default_sorting_field,
                } = field;

                let ty = quote!(#ty);

                let optional = optional.unwrap_or(false);
                let default_sorting_field = default_sorting_field.unwrap_or(false);

                impl_field_inner(
                    tokens,
                    &name,
                    ty,
                    &index,
                    &facet,
                    &sort,
                    optional,
                    default_sorting_field,
                );
            }
        }
    }
}

fn impl_field_inner(
    tokens: &mut proc_macro2::TokenStream,
    name: &str,
    ty: proc_macro2::TokenStream,
    index: &Option<bool>,
    facet: &Option<bool>,
    sort: &Option<bool>,
    optional: bool,
    default_sorting_field: bool,
) {
    let facet = facet.map(|f| quote!(Some(#f))).unwrap_or(quote!(None));
    let index = index.map(|i| quote!(Some(#i))).unwrap_or(quote!(None));
    let sort = sort.map(|i| quote!(Some(#i))).unwrap_or(quote!(None));
    let optional = optional
        .then_some(quote!(Some(true)))
        .unwrap_or(quote!(None));

    tokens.extend(quote! {
        .field(::typesensei::schema::Field {
            name: ::std::borrow::Cow::Borrowed(#name),
            field_type: #ty,
            facet: #facet,
            index: #index,
            sort: #sort,
            optional: #optional,
            drop: None
        })
    });

    if default_sorting_field {
        tokens.extend(quote! {
            .default_sorting_field(#name)
        })
    }
}
