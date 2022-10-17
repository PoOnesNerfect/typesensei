use super::{case::RenameRule, Field, Typesense};
use darling::{FromDeriveInput, Result};
use proc_macro2::Ident;
use syn::{DeriveInput, Generics, Type};

#[derive(FromDeriveInput)]
#[darling(supports(struct_named), attributes(serde, typesense))]
pub struct Derived {
    ident: Ident,
    generics: Generics,
    data: darling::ast::Data<(), Field>,
    rename: Option<String>,
    rename_all: Option<String>,
    id: Option<String>,
}

pub fn parse(input: &DeriveInput) -> Result<Typesense> {
    let Derived {
        ident,
        generics,
        data,
        rename,
        rename_all,
        id,
    } = Derived::from_derive_input(&input)?;

    let mut fields = data
        .take_struct()
        .expect("only named struct should be derived")
        .fields;

    if let Some(rename_all) = rename_all {
        let case = RenameRule::from_str(&rename_all).map_err(|e| darling::Error::custom(e))?;

        for field in &mut fields {
            field.case.replace(case);
        }
    }

    Ok(Typesense {
        ident,
        generics,
        id_type: id_type(id, &fields)?,
        fields,
        rename,
    })
}

fn id_type(id: Option<String>, fields: &Vec<Field>) -> Result<Type> {
    if let Some(id) = id {
        return syn::parse_str(&id).map_err(|e| darling::Error::custom(e));
    }

    for field in fields {
        let Field { ident, ty, .. } = field;

        let ident = ident
            .as_ref()
            .expect("field of non-tuple struct must have name");

        if ident == "id" {
            return Ok(ty.clone());
        }
    }

    Ok(syn::parse_str("i32").expect("u32 should be parsed as Type"))
}
