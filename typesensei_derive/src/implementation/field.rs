use darling::FromField;
use quote::{quote, ToTokens};
use syn::token::{Brace, Paren};

#[derive(FromField)]
#[darling(attributes(serde, typesense))]
pub struct Field {
    pub ident: Option<syn::Ident>,
    pub ty: syn::Type,
    pub facet: Option<bool>,
    pub index: Option<bool>,
    #[darling(default)]
    pub default_sorting_field: bool,
    #[darling(default)]
    pub flatten: bool,
    pub rename: Option<String>,
    #[darling(default)]
    pub skip: bool,
    #[darling(skip)]
    pub case: Option<super::case::RenameRule>,
}

impl ToTokens for Field {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            ident,
            ty,
            index,
            facet,
            default_sorting_field,
            flatten,
            rename,
            skip,
            case,
        } = self;

        if *skip {
            return;
        }

        if *flatten {
            tokens.extend(quote!(.extend));

            Paren::default().surround(tokens, |parens| {
                Brace::default().surround(parens, |braces| {
                    braces.extend(quote! {
                        let mut other = <#ty>::schema();
                    });

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

                    if set_facet.is_some() || set_rename.is_some() {
                        braces.extend(quote! {
                            for field in &mut other.fields {
                                #set_facet
                                #set_index
                                #set_rename
                            }
                        });
                    }

                    braces.extend(quote! {
                        other
                    });
                });
            });
            return;
        }

        let name = if let Some(name) = rename {
            name.to_owned()
        } else {
            let mut field = ident
                .as_ref()
                .expect("Named struct should have named field")
                .to_string();

            if let Some(case) = case {
                field = case.apply_to_field(&field);
            }

            field
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
}
