use proc_macro2::Ident;
use quote::{quote, ToTokens};
use syn::{Generics, Type};

mod case;
mod field;
pub use field::*;
mod parse;
pub use parse::parse;

pub struct Typesense {
    ident: Ident,
    generics: Generics,
    fields: Vec<Field>,
    id_type: Type,
    rename: Option<String>,
}

impl ToTokens for Typesense {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            ident,
            generics,
            fields,
            id_type,
            rename,
        } = self;

        let schema_name = rename
            .to_owned()
            .unwrap_or_else(|| case::RenameRule::SnakeCase.apply_to_variant(&ident.to_string()));

        let (_impl, _type, _where) = generics.split_for_impl();

        tokens.extend(quote! {
            impl #_impl ::typesensei::Typesense for #ident #_type #_where {
                type DocumentId = #id_type;

                fn schema_name() -> &'static str {
                    #schema_name
                }

                fn schema() -> ::typesensei::schema::CollectionSchema<'static> {
                    use ::typesensei::{Typesense, TypesenseField};
                    ::typesensei::schema::CollectionSchema::new(Self::schema_name())
                    #(
                        #fields
                    )*
                }
            }
        });
    }
}
