use darling::FromField;
use quote::format_ident;
use syn::Ident;

#[derive(FromField, Clone)]
#[darling(attributes(serde, typesense))]
pub struct Field {
    pub ident: Option<syn::Ident>,
    #[darling(skip, default = "dummy_ident")]
    pub field: syn::Ident,
    pub ty: syn::Type,

    pub facet: Option<bool>,
    pub index: Option<bool>,
    pub rename: Option<String>,
    #[darling(default)]
    pub default_sorting_field: bool,
    #[darling(default)]
    pub flatten: bool,
    #[darling(default)]
    pub skip: bool,

    #[darling(skip, default)]
    pub is_option: bool,
    #[darling(skip, default)]
    pub generic_type: Option<Ident>,
}

fn dummy_ident() -> syn::Ident {
    format_ident!("dummy")
}

impl Field {
    pub fn post_process(this: &mut Self) {
        this.field = this
            .ident
            .take()
            .expect("named struct should have named fields");
    }
}
