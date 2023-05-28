use self::struct_parser::StructParser;
use super::{case, case::RenameRule, is_object, is_object_array, Field, Translator};
use darling::{Error, FromDeriveInput, FromMeta, Result};
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::{format_ident, quote};
use syn::{
    punctuated::Punctuated, DeriveInput, GenericArgument, GenericParam, Generics, Ident, Meta,
    Path, PathArguments, Type, WhereClause, WherePredicate,
};

pub mod struct_parser;

#[derive(FromDeriveInput)]
#[darling(supports(struct_named), attributes(serde, typesensei))]
pub struct Derived {
    ident: Ident,
    generics: Generics,
    data: darling::ast::Data<(), Field>,

    #[darling(rename = "crate", default = "default_serde")]
    serde: String,
    model: Option<String>,
    query: Option<String>,
    rename: Option<String>,
    rename_all: Option<String>,
    #[darling(default)]
    enable_nested_fields: bool,
    extra_fields: Option<TypesenseFields>,
}

#[derive(Debug, Clone)]
pub struct TypesenseFields(pub Vec<ts::Field>);

impl FromMeta for TypesenseFields {
    fn from_meta(item: &syn::Meta) -> Result<Self> {
        (match *item {
            Meta::Path(_) => Self::from_word(),
            Meta::List(ref value) => Self::from_list(
                &value
                    .nested
                    .iter()
                    .cloned()
                    .collect::<Vec<syn::NestedMeta>>()[..],
            ),
            Meta::NameValue(ref value) => ts::Field::from_value(&value.lit).map(|t| Self(vec![t])),
        })
        .map_err(|e| e.with_span(item))
    }

    fn from_list(items: &[syn::NestedMeta]) -> Result<Self> {
        items
            .iter()
            .map(|i| ts::Field::from_nested_meta(i))
            .collect::<Result<Vec<_>>>()
            .map(|t| Self(t))
    }
}

pub mod ts {
    use darling::FromMeta;

    #[derive(Debug, Clone, FromMeta)]
    pub struct Field {
        pub name: String,
        pub ty: String,
        pub facet: Option<bool>,
        pub index: Option<bool>,
        pub sort: Option<bool>,
        pub optional: Option<bool>,
        pub default_sorting_field: Option<bool>,
    }
}

fn default_serde() -> String {
    "::serde".to_owned()
}

impl Translator {
    pub fn from_derived(input: &DeriveInput) -> Result<Translator> {
        let Derived {
            ident,
            generics,
            data,

            serde,
            model,
            query,
            rename,
            rename_all,
            mut enable_nested_fields,
            extra_fields,
        } = Derived::from_derive_input(&input)?;

        let serde = syn::parse_str(&serde)?;

        let fields = data
            .take_struct()
            .expect("only named struct should be derived")
            .fields;
        let mut fields = fields.into_iter().filter(|f| !f.skip).collect::<Vec<_>>();
        fields.iter_mut().for_each(Field::post_process);
        mark_field_types(&generics, &mut fields);

        if !enable_nested_fields {
            enable_nested_fields = fields.iter().any(|f| is_object(f) || is_object_array(f));
        }

        let case = if let Some(rename_all) = &rename_all {
            RenameRule::from_str(&rename_all).map_err(|e| darling::Error::custom(e))?
        } else {
            Default::default()
        };

        let mut main_generics = generics.clone();
        let main_fields = fields.clone();

        add_generic_bounds(
            &mut main_generics,
            &main_fields,
            quote!(::typesensei::Typesense),
            &serde,
        );

        let model = StructParser::new(
            &serde,
            &ident,
            &model,
            format_ident!("Model"),
            &generics,
            &fields,
        );

        let query = StructParser::new(
            &serde,
            &ident,
            &query,
            format_ident!("Query"),
            &generics,
            &fields,
        );

        let schema_name = rename
            .to_owned()
            .unwrap_or_else(|| case::RenameRule::SnakeCase.apply_to_variant(&ident.to_string()));

        Ok(Self {
            serde,
            rename_all,
            case,
            id_type: id_type(&fields)?,

            ident,
            main_fields,
            main_generics,

            model,
            query,
            schema_name,
            enable_nested_fields,
            extra_fields,
        })
    }
}

fn mark_field_types(generics: &Generics, fields: &mut Vec<Field>) {
    let generic_types = generics
        .params
        .iter()
        .filter_map(|param| {
            if let GenericParam::Type(ty) = param {
                Some(&ty.ident)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    for field in fields {
        if let Type::Path(path) = &field.ty {
            let path = &path.path;
            for seg in &path.segments {
                // object array types must be Vec
                if is_object_array(field) && seg.ident != "Vec" {
                    abort!(path, "'object[]' field must be of type Vec<_>")
                }

                if seg.ident == "Option" || seg.ident == "Vec" {
                    if let PathArguments::AngleBracketed(args) = &seg.arguments {
                        let args = &args.args;
                        if let Some(GenericArgument::Type(Type::Path(path))) = args.first() {
                            if let Some(ident) = path.path.get_ident() {
                                if generic_types.contains(&ident) {
                                    field.generic_type.replace(ident.clone());
                                }
                            }

                            if seg.ident == "Option" {
                                field.is_option.replace(Type::Path(path.clone()));
                            }

                            // if the type is Vec, then store the inner type
                            if seg.ident == "Vec" {
                                field.is_vec.replace(Type::Path(path.clone()));
                            }
                        }
                    }
                } else if generic_types.contains(&&seg.ident) {
                    field.generic_type.replace(seg.ident.clone());
                }
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
                bounds
                    .push(syn::parse_quote!(<#ty as ::typesensei::traits::TypesenseField>::Type : 'static));
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

fn id_type(fields: &Vec<Field>) -> Result<Type> {
    for field in fields {
        let Field { raw_ident, ty, .. } = field;

        if raw_ident == "id" {
            return Ok(ty.clone());
        }
    }

    syn::parse_str("String").map_err(|e| Error::custom(e))
}
