use super::{case::RenameRule, Field, ImplTypesense, Implementation};
use darling::ToTokens;
use proc_macro2::TokenStream;
use syn::{Generics, Ident, Path, Type, WhereClause};

pub struct Translator {
    pub serde: Path,
    pub rename_all: Option<String>,
    pub case: RenameRule,
    pub id: Type,

    pub ident: Ident,
    pub fields: Vec<Field>,
    pub generics: Generics,

    pub model_ident: Ident,
    pub model_fields: Vec<Field>,
    pub model_generics: Generics,
    pub model_struct_where_clause: Option<WhereClause>,

    pub merged_generics: Generics,

    pub schema_name: String,
    pub generic_model_type: Type,
}

impl Translator {
    pub fn translate(&self) -> TokenStream {
        let Self {
            serde,
            rename_all,
            case,
            id,
            ident,
            fields,
            generics,
            model_ident,
            model_fields,
            model_generics,
            model_struct_where_clause,
            merged_generics,
            schema_name,
            generic_model_type,
        } = self;

        let impl_typesense = ImplTypesense {
            ident,
            generics,
            id_type: id,
            generic_model_type,
            schema_name,
            fields,
            case,
        };

        let (_, type_generics, _) = generics.split_for_impl();

        let (model_impl_generics, model_type_generics, model_where_clause) =
            model_generics.split_for_impl();

        let struct_model = super::StructModel {
            serde,
            model_ident,
            model_impl_generics: &model_impl_generics,
            model_struct_where_clause,
            model_fields,
            rename_all,
            id,
        };

        let impl_model = super::ImplModel {
            model_ident,
            model_impl_generics: &model_impl_generics,
            model_type_generics: &model_type_generics,
            model_where_clause: &model_where_clause,
        };

        let (merged_impl_generics, _, merged_where_clause) = merged_generics.split_for_impl();

        let impl_from = super::ImplFrom {
            merged_impl_generics: &merged_impl_generics,
            ident,
            model_ident,
            type_generics: &type_generics,
            model_type_generics: &model_type_generics,
            merged_where_clause: &merged_where_clause,
            fields,
        };

        let implementation = Implementation {
            impl_typesense,
            struct_model,
            impl_model,
            impl_from,
        };

        implementation.to_token_stream()
    }
}
