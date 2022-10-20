use super::{case::RenameRule, Field};
use darling::ToTokens;
use quote::quote;
use syn::{
    token::{Brace, Paren},
    Generics, Ident, Type,
};

pub struct ImplTypesense<'a> {
    pub ident: &'a Ident,
    pub generics: &'a Generics,
    pub id_type: &'a Type,
    pub generic_model_type: &'a Type,
    pub schema_name: &'a String,
    pub fields: &'a Vec<Field>,
    pub case: &'a RenameRule,
}

impl<'a> ToTokens for ImplTypesense<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            ident,
            generics,
            id_type,
            generic_model_type,
            schema_name,
            fields,
            case,
        } = self;

        let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

        let fields_impl = FieldImpl::new(fields, case);

        tokens.extend(quote! {
            impl #impl_generics ::typesensei::Typesense for #ident #type_generics
            #where_clause
            {
                type DocumentId = #id_type;
                type Model = #generic_model_type;

                fn schema_name() -> &'static str {
                    #schema_name
                }

                fn schema() -> typesensei::schema::CollectionSchema<'static> {
                    use ::typesensei::{Typesense, TypesenseField};
                    ::typesensei::schema::CollectionSchema::new(Self::schema_name())
                    #fields_impl
                }
            }
        });
    }
}

struct FieldImpl<'a> {
    fields: &'a Vec<Field>,
    case: &'a RenameRule,
}

impl<'a> FieldImpl<'a> {
    fn new(fields: &'a Vec<Field>, case: &'a RenameRule) -> Self {
        Self { fields, case }
    }
}

impl<'a> ToTokens for FieldImpl<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        for field in self.fields {
            if field.skip {
                continue;
            }

            if field.flatten {
                impl_flatten_field(&field, tokens);
            } else {
                impl_field(&field, &self.case, tokens);
            }
        }
    }
}

fn impl_flatten_field(field: &Field, tokens: &mut proc_macro2::TokenStream) {
    let Field {
        ty,
        index,
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
                    let mut schema = <#ty>::schema();
                });
            } else {
                braces.extend(quote! {
                    let mut schema = <#ty>::schema();
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

            let set_rename = rename.as_ref().map(|r| {
                quote! {
                    field.name = #r;
                }
            });

            if set_facet.is_some() || set_index.is_some() || set_rename.is_some() {
                braces.extend(quote! {
                    for field in &mut schema.fields {
                        #set_facet
                        #set_index
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

fn impl_field(field: &Field, case: &RenameRule, tokens: &mut proc_macro2::TokenStream) {
    let Field {
        field,
        ty,
        index,
        facet,
        default_sorting_field,
        rename,
        ..
    } = field;

    let name = if let Some(name) = rename {
        name.to_owned()
    } else {
        case.apply_to_field(&field.to_string())
    };

    let facet = facet.map(|f| quote!(Some(#f))).unwrap_or(quote!(None));
    let index = index.map(|i| quote!(Some(#i))).unwrap_or(quote!(None));

    tokens.extend(quote! {
        .field(::typesensei::schema::Field {
            name: #name,
            field_type: < #ty >::field_type(),
            facet: #facet,
            index: #index
        })
    });

    if *default_sorting_field {
        tokens.extend(quote! {
            .default_sorting_field(#name)
        })
    }
}
