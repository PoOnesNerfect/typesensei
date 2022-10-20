use super::{case, case::RenameRule, Field, Translator};
use darling::{Error, FromDeriveInput, Result};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{
    punctuated::Punctuated, DeriveInput, GenericArgument, GenericParam, Generics, Ident, Path,
    PathArguments, Type, WhereClause, WherePredicate,
};

#[derive(FromDeriveInput)]
#[darling(supports(struct_named), attributes(serde, typesensei))]
pub struct Derived {
    ident: Ident,
    generics: Generics,
    data: darling::ast::Data<(), Field>,

    #[darling(rename = "crate", default = "default_serde")]
    serde: String,
    model: Option<String>,
    rename: Option<String>,
    rename_all: Option<String>,
    id: Option<String>,
}

fn default_serde() -> String {
    "::serde".to_owned()
}

impl Translator {
    pub fn from_derived(input: &DeriveInput) -> Result<Translator> {
        let Derived {
            ident,
            mut generics,
            data,

            serde,
            model,
            rename,
            rename_all,
            id,
        } = Derived::from_derive_input(&input)?;

        let serde = syn::parse_str(&serde)?;

        let mut fields = data
            .take_struct()
            .expect("only named struct should be derived")
            .fields;
        fields.iter_mut().for_each(Field::post_process);
        mark_field_types(&generics, &mut fields);

        let case = if let Some(rename_all) = &rename_all {
            RenameRule::from_str(&rename_all).map_err(|e| darling::Error::custom(e))?
        } else {
            Default::default()
        };

        let fields_template = fields.clone();
        let generics_template = generics.clone();

        add_generic_bounds(
            &mut generics,
            &fields,
            quote!(::typesensei::Typesense),
            &serde,
        );

        let model_ident = model
            .map(|m| format_ident!("{m}"))
            .unwrap_or_else(|| format_ident!("{ident}Model"));
        let generic_model_type = get_generic_model_type(&model_ident, &generics, &fields);
        let mut model_generics = generics_template.clone();
        let mut model_fields = fields_template.clone();
        add_str_to_generics(&mut model_generics, &mut model_fields, "Model");
        let model_struct_where_clause = model_generics.where_clause.clone();
        add_generic_bounds(
            &mut model_generics,
            &model_fields,
            quote!(::typesensei::TypesenseModel),
            &serde,
        );

        let merged_generics = get_merged_generics(&generics_template, &fields_template, &serde);

        let schema_name = rename
            .to_owned()
            .unwrap_or_else(|| case::RenameRule::SnakeCase.apply_to_variant(&ident.to_string()));

        Ok(Self {
            serde,
            rename_all,
            case,
            id: id_type(id, &fields)?,

            ident,
            fields,
            generics,

            generic_model_type,
            model_ident,
            model_fields,
            model_generics,
            model_struct_where_clause,

            merged_generics,
            schema_name,
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
                if seg.ident == "Option" {
                    field.is_option = true;

                    if let PathArguments::AngleBracketed(args) = &seg.arguments {
                        let args = &args.args;
                        if let Some(GenericArgument::Type(Type::Path(path))) = args.first() {
                            if let Some(ident) = path.path.get_ident() {
                                if generic_types.contains(&ident) {
                                    field.generic_type.replace(ident.clone());
                                }
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
                bounds.push(syn::parse_quote!(for<'de> #ty : ::std::fmt::Debug + Default + #serde ::Serialize + #serde ::Deserialize<'de> + ::typesensei::TypesenseField));
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

fn add_str_to_generics(generics: &mut Generics, fields: &mut Vec<Field>, val: &str) {
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
                let new_ty = format_ident!("{ty}{val}");
                if let Some((ty, bounds)) = generic_types.iter_mut().find(|(t, _)| (**t).eq(ty)) {
                    **ty = new_ty.clone();
                    bounds.clear();
                }
                *ty = new_ty;
            }
        }
    }
}

fn get_generic_model_type(model_ident: &Ident, generics: &Generics, fields: &Vec<Field>) -> Type {
    let mut types = quote!(#model_ident);

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
                            types.extend(quote!(<#ty as ::typesensei::Typesense>::Model , ));
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

fn get_merged_generics(generics: &Generics, fields: &Vec<Field>, serde: &Path) -> Generics {
    let mut generics = generics.clone();

    let mut bounds: Vec<WherePredicate> = Vec::new();

    for field in fields {
        if let Some(ty) = field.generic_type.as_ref() {
            if field.flatten {
                let model_ty = format_ident!("{ty}Model");
                generics.params.push(syn::parse_quote!(#model_ty));

                bounds.push(syn::parse_quote!(#ty : ::typesensei::Typesense));

                bounds
                    .push(syn::parse_quote!(#model_ty : ::typesensei::TypesenseModel + From<#ty>));
            } else {
                bounds.push(syn::parse_quote!(for<'de> #ty : ::std::fmt::Debug + Default + #serde ::Serialize + #serde ::Deserialize<'de> + ::typesensei::TypesenseField));
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

fn id_type(id: Option<String>, fields: &Vec<Field>) -> Result<Type> {
    if let Some(id) = id {
        return syn::parse_str(&id).map_err(|e| Error::custom(e));
    }

    for field in fields {
        let Field { field, ty, .. } = field;

        if field == "id" {
            return Ok(ty.clone());
        }
    }

    syn::parse_str("i32").map_err(|e| Error::custom(e))
}
