use darling::FromField;
use quote::format_ident;
use syn::Ident;

pub fn is_object(f: &Field) -> bool {
    f.custom_type
        .as_ref()
        .map(|t| t == "object")
        .unwrap_or(false)
}

pub fn is_object_array(f: &Field) -> bool {
    f.custom_type
        .as_ref()
        .map(|t| t == "object[]")
        .unwrap_or(false)
}

pub fn fields_has_id(fields: &[Field]) -> bool {
    fields.iter().any(field_is_id)
}

pub fn field_is_id(field: &Field) -> bool {
    field.raw_ident == "id" || field.rename.as_ref().map(|r| r == "id").unwrap_or_default()
}

#[derive(FromField, Clone)]
#[darling(attributes(serde, typesensei))]
pub struct Field {
    pub ident: Option<syn::Ident>,

    // unwrapped ident
    #[darling(skip, default = "dummy_ident")]
    pub raw_ident: syn::Ident,
    pub ty: syn::Type,

    pub facet: Option<bool>,
    pub index: Option<bool>,
    pub sort: Option<bool>,
    pub rename: Option<String>,
    pub custom_type: Option<String>,
    pub optional: Option<bool>,

    #[darling(default)]
    pub default_sorting_field: bool,
    #[darling(default)]
    pub flatten: bool,
    #[darling(default)]
    pub skip: bool,

    #[darling(skip, default)]
    pub is_option: Option<syn::Type>,
    #[darling(skip, default)]
    pub is_vec: Option<syn::Type>, // contains inner type if type is vec
    #[darling(skip, default)]
    pub generic_type: Option<Ident>,
}

fn dummy_ident() -> syn::Ident {
    format_ident!("dummy")
}

impl Field {
    pub fn post_process(this: &mut Self) {
        this.raw_ident = this
            .ident
            .take()
            .expect("named struct should have named fields");
    }
}
