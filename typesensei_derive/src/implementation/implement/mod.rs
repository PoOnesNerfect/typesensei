use self::{impl_partial::ImplPartial, impl_typesense::ImplTypesense};
use super::{case::RenameRule, Field, SymbolsToIndex, TypesenseFields};
use darling::ToTokens;
use proc_macro2::TokenStream;
use syn::{Generics, Ident, Path, Type};

pub mod impl_partial;
pub mod impl_typesense;

pub struct Implementor {
    pub vis: syn::Visibility,
    pub serde: Path,
    pub rename_all: Option<String>,
    pub case: RenameRule,
    pub id_type: Type,

    pub ident: Ident,
    pub main_fields: Vec<Field>,
    pub main_generics: Generics,

    pub enable_nested_fields: bool,
    pub extra_fields: Option<TypesenseFields>,
    pub symbols_to_index: Option<SymbolsToIndex>,
}

impl Implementor {
    pub fn impl_typesense(&self) -> TokenStream {
        pub struct Implementation<'a> {
            impl_typesense: ImplTypesense<'a>,
            impl_partial: ImplPartial<'a>,
        }

        impl<'a> ToTokens for Implementation<'a> {
            fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
                let Self {
                    impl_typesense,
                    impl_partial,
                } = self;

                impl_partial.to_tokens(tokens);
                impl_typesense.to_tokens(tokens);
            }
        }

        let Self {
            vis,
            serde,
            rename_all,
            case,
            id_type,
            ident,
            main_generics,
            main_fields,
            enable_nested_fields,
            extra_fields,
            symbols_to_index,
        } = self;

        let impl_typesense = ImplTypesense {
            ident,
            generics: main_generics,
            fields: main_fields,
            id_type,
            enable_nested_fields: *enable_nested_fields,
            case,
            extra_fields: &extra_fields,
            symbols_to_index: &symbols_to_index,
        };

        let impl_partial = ImplPartial::new(
            vis,
            ident,
            main_generics,
            main_fields,
            case,
            serde,
            rename_all,
        );

        let implementation = Implementation {
            impl_typesense,
            impl_partial,
        };

        implementation.to_token_stream()
    }

    pub fn impl_partial(&self) -> TokenStream {
        let Self {
            vis,
            serde,
            rename_all,
            case,
            ident,
            main_generics,
            main_fields,
            ..
        } = self;

        let impl_partial = ImplPartial::new(
            vis,
            ident,
            main_generics,
            main_fields,
            case,
            serde,
            rename_all,
        );

        impl_partial.to_token_stream()
    }
}
