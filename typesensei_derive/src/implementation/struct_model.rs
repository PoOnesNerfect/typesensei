use super::Field;
use darling::ToTokens;
use quote::quote;
use syn::{token::Brace, Ident, ImplGenerics, Path, Token, Type, WhereClause};

pub struct StructModel<'a> {
    pub serde: &'a Path,
    pub model_ident: &'a Ident,
    pub model_impl_generics: &'a ImplGenerics<'a>,
    pub model_struct_where_clause: &'a Option<WhereClause>,
    pub model_fields: &'a Vec<Field>,
    pub rename_all: &'a Option<String>,
    pub id: &'a Type,
}

impl<'a> ToTokens for StructModel<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            serde,
            model_ident,
            model_impl_generics,
            model_struct_where_clause,
            model_fields,
            rename_all,
            id,
        } = self;

        let hash: Token![#] = Default::default();

        tokens.extend(quote! {
            #hash [derive(Debug, Default, #serde ::Serialize, #serde ::Deserialize)]
            #rename_all
            pub struct #model_ident #model_impl_generics #model_struct_where_clause
        });

        Brace::default().surround(tokens, |braces| {
            FieldsImpl::new(model_fields, &hash, id).to_tokens(braces);
        });
    }
}

struct FieldsImpl<'a> {
    fields: &'a Vec<Field>,
    hash: &'a Token![#],
    id: &'a Type,
}

impl<'a> FieldsImpl<'a> {
    fn new(fields: &'a Vec<Field>, hash: &'a Token![#], id: &'a Type) -> Self {
        Self { fields, hash, id }
    }
}

impl<'a> ToTokens for FieldsImpl<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        // inject id field if not exists
        if !self
            .fields
            .iter()
            .any(|f| f.field == "id" || f.rename.as_ref().map(|r| r == "id").unwrap_or_default())
        {
            let hash = self.hash;
            let id = self.id;
            tokens.extend(quote! {
                #hash [serde(skip_serializing_if = "::typesensei::FieldState::is_not_set")]
                pub id : ::typesensei::FieldState<#id>,
            });
        }

        for field in self.fields {
            if field.skip {
                continue;
            }

            if field.flatten {
                impl_flatten_field(&field, self.hash, tokens);
            } else {
                impl_field(&field, self.hash, tokens);
            }
        }
    }
}

fn impl_flatten_field(field: &Field, hash: &Token![#], tokens: &mut proc_macro2::TokenStream) {
    let Field {
        field,
        is_option,
        generic_type,
        ty,
        ..
    } = field;

    if let Some(ty) = generic_type.as_ref() {
        if *is_option {
            tokens.extend(quote! {
                #hash [serde(flatten)]
                #field : Option<#ty>,
            });
        } else {
            tokens.extend(quote! {
                #hash [serde(flatten)]
                #field : #ty,
            });
        }
    } else {
        if *is_option {
            tokens.extend(quote! {
                #hash [serde(flatten)]
                #field : Option<#ty>,
            });
        } else {
            tokens.extend(quote! {
                #hash [serde(flatten)]
                #field : #ty,
            });
        }
    }
}

fn impl_field(field: &Field, hash: &Token![#], tokens: &mut proc_macro2::TokenStream) {
    let Field {
        field,
        ty,
        rename,
        is_option,
        ..
    } = field;

    if *is_option {
        tokens.extend(quote! (#hash [serde(skip_serializing_if = "::typesensei::FieldState::is_inner_option_none")]));
    } else {
        tokens.extend(
            quote!(#hash [serde(skip_serializing_if = "::typesensei::FieldState::is_not_set")]),
        );
    }

    let rename = rename.as_ref().map(|r| quote!(#hash [serde(rename = #r)]));

    tokens.extend(quote! {
        #rename
        pub #field : ::typesensei::FieldState<#ty>,
    });
}
