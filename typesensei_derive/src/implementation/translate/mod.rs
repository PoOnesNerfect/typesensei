use self::{
    impl_from::ImplFrom,
    impl_model::ImplModel,
    impl_typesense::{ImplTypesense, SchemaName},
    struct_model::StructModel,
};
use super::{
    case::RenameRule, struct_parser::StructParser, Field, SymbolsToIndex, TypesenseFields,
};
use darling::ToTokens;
use proc_macro2::TokenStream;
use syn::{Generics, Ident, Path, Type};

pub mod impl_from;
pub mod impl_model;
pub mod impl_query;
pub mod impl_typesense;
pub mod struct_model;
pub mod struct_query;

pub struct Implementation<'a> {
    impl_typesense: ImplTypesense<'a>,
    struct_model: StructModel<'a>,
    impl_model: ImplModel<'a>,
    impl_from: ImplFrom<'a>,
    // struct_query: StructQuery<'a>,
    // impl_query: ImplQuery<'a>,
}

impl<'a> ToTokens for Implementation<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            impl_typesense,
            struct_model,
            impl_model,
            impl_from,
            // struct_query,
            // impl_query,
        } = self;

        impl_typesense.to_tokens(tokens);
        struct_model.to_tokens(tokens);
        impl_model.to_tokens(tokens);
        impl_from.to_tokens(tokens);
        // struct_query.to_tokens(tokens);
        // impl_query.to_tokens(tokens);
    }
}

pub struct Translator {
    pub serde: Path,
    pub rename_all: Option<String>,
    pub case: RenameRule,
    pub id_type: Type,

    pub ident: Ident,
    pub main_fields: Vec<Field>,
    pub main_generics: Generics,

    pub model: StructParser,
    pub query: StructParser,

    pub schema_name: SchemaName,
    pub enable_nested_fields: bool,
    pub extra_fields: Option<TypesenseFields>,
    pub symbols_to_index: Option<SymbolsToIndex>,
}

impl Translator {
    pub fn translate(&self) -> TokenStream {
        let Self {
            serde,
            rename_all,
            case,
            id_type,
            ident,
            main_generics,
            main_fields,
            model,
            query,
            schema_name,
            enable_nested_fields,
            extra_fields,
            symbols_to_index,
        } = self;

        let impl_typesense = ImplTypesense {
            ident,
            generics: main_generics,
            fields: main_fields,
            id_type,
            model_associated_type: &model.associated_type,
            query_associated_type: &query.associated_type,
            schema_name,
            enable_nested_fields: *enable_nested_fields,
            case,
            extra_fields: &extra_fields,
            symbols_to_index: &symbols_to_index,
        };

        let (_, main_type_generics, _) = main_generics.split_for_impl();

        let (model_impl_generics, model_type_generics, model_where_clause) =
            model.generics.split_for_impl();

        let struct_model = StructModel {
            serde,
            ident: &model.ident,
            impl_generics: &model_impl_generics,
            where_clause: &model.struct_where_clause,
            fields: &model.fields,
            rename_all,
            id_type,
        };

        let impl_model = ImplModel {
            ident: &model.ident,
            impl_generics: &model_impl_generics,
            type_generics: &model_type_generics,
            where_clause: &model_where_clause,
            fields: &model.fields,
        };

        let (merged_impl_generics, _, merged_where_clause) = model.merged_generics.split_for_impl();

        let impl_from = ImplFrom {
            merged_impl_generics: &merged_impl_generics,
            ident,
            model_ident: &model.ident,
            main_type_generics: &main_type_generics,
            model_type_generics: &model_type_generics,
            merged_where_clause: &merged_where_clause,
            fields: main_fields,
        };

        // let (query_impl_generics, query_type_generics, query_where_clause) =
        //     query.generics.split_for_impl();

        // let struct_query = StructQuery {
        //     ident: &query.ident,
        //     impl_generics: &query_impl_generics,
        //     where_clause: &query_where_clause,
        //     fields: &query.fields,
        //     id_type,
        // };

        // let impl_query = ImplQuery {
        //     ident: &query.ident,
        //     impl_generics: &query_impl_generics,
        //     type_generics: &query_type_generics,
        //     where_clause: &query_where_clause,
        //     fields: &query.fields,
        // };

        let implementation = Implementation {
            impl_typesense,
            struct_model,
            impl_model,
            impl_from,
            // struct_query,
            // impl_query,
        };

        implementation.to_token_stream()
    }
}
