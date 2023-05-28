use super::Field;
use crate::implementation::{fields_has_id, is_object};
use darling::ToTokens;
use quote::{format_ident, quote};
use syn::{token::Brace, Ident, ImplGenerics, TypeGenerics, WhereClause};

pub struct ImplQuery<'a> {
    pub ident: &'a Ident,
    pub impl_generics: &'a ImplGenerics<'a>,
    pub type_generics: &'a TypeGenerics<'a>,
    pub where_clause: &'a Option<&'a WhereClause>,
    pub fields: &'a Vec<Field>,
}

impl<'a> ToTokens for ImplQuery<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            ident,
            impl_generics,
            type_generics,
            where_clause,
            fields,
        } = self;

        tokens.extend(quote! {
            impl #impl_generics Default for #ident #type_generics
            #where_clause
        });

        Brace::default().surround(tokens, |braces| {
            braces.extend(quote!(fn default() -> Self));

            Brace::default().surround(braces, |braces| {
                braces.extend(quote! {
                    use ::typesensei::traits::TypesenseQuery;
                    let prio = ::std::rc::Rc::new(::std::cell::RefCell::new(0));
                    Self::with_counter(prio)
                });
            });
        });

        tokens.extend(quote! {
            impl #impl_generics #ident #type_generics
            #where_clause
        });

        Brace::default().surround(tokens, |braces| {
            braces.extend(quote!(pub fn q(self, query: String) -> ::typesensei::SearchQuery));

            Brace::default().surround(braces, |braces| {
                braces.extend(quote! {
                    ::typesensei::traits::TypesenseQuery::q(self, query)
                });
            });
        });

        tokens.extend(quote! {
            impl #impl_generics ::typesensei::traits::TypesenseQuery for #ident #type_generics
            #where_clause
        });

        Brace::default().surround(tokens, |braces| {
            FieldImplWithCounter::new(fields).to_tokens(braces);

            FieldsImplExtend::new(fields, "query_by").to_tokens(braces);
            FieldsImplExtend::new(fields, "sort_by").to_tokens(braces);
            FieldsImplExtend::new(fields, "filter_by").to_tokens(braces);
        });
    }
}

struct FieldImplWithCounter<'a> {
    fields: &'a Vec<Field>,
}

impl<'a> FieldImplWithCounter<'a> {
    fn new(fields: &'a Vec<Field>) -> Self {
        Self { fields }
    }
}

impl<'a> ToTokens for FieldImplWithCounter<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(
            quote!(fn with_counter(prio: ::std::rc::Rc<::std::cell::RefCell<u16>>) -> Self),
        );

        Brace::default().surround(tokens, |braces| {
            braces.extend(quote!(
                use ::typesensei::traits::TypesenseQuery;
            ));
            braces.extend(quote!(Self));

            Brace::default().surround(braces, |braces| {
                // inject id field if not exists
                if !fields_has_id(&self.fields) {
                    braces.extend(quote! {
                        id : ::typesensei::query::QueryState::new(prio.clone()),
                    });
                }

                for field in self.fields {
                    let Field {
                        raw_ident,
                        is_option,
                        flatten,
                        ..
                    } = field;

                    if *flatten || is_object(field) {
                        if is_option.is_some() {
                            braces.extend(quote! {
                                #raw_ident : Some(<_>::with_counter(prio.clone())),
                            });
                        } else {
                            braces.extend(quote! {
                                #raw_ident : <_>::with_counter(prio.clone()),
                            });
                        }
                    } else {
                        braces.extend(quote! {
                            #raw_ident : ::typesensei::query::QueryState::new(prio.clone()),
                        });
                    }
                }
            });
        });
    }
}

struct FieldsImplExtend<'a> {
    fields: &'a Vec<Field>,
    ident: &'a str,
}

impl<'a> FieldsImplExtend<'a> {
    fn new(fields: &'a Vec<Field>, ident: &'a str) -> Self {
        Self { fields, ident }
    }
}

impl<'a> ToTokens for FieldsImplExtend<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { fields, ident } = self;

        let extend_name = format_ident!("extend_{ident}");
        let take_name = format_ident!("take_{ident}");
        let len_name = format_ident!("{ident}_len");

        tokens.extend(quote! (fn #len_name(this: &Self) -> usize));
        Brace::default().surround(tokens, |braces| {
            braces.extend(quote! {
                use ::typesensei::{traits::TypesenseQuery, query::QueryState};

                0
            });

            if !fields_has_id(&fields) {
                braces.extend(quote! (+ QueryState:: #len_name(&this.id)));
            }

            for field in fields.iter() {
                let Field {
                    raw_ident, flatten, ..
                } = field;

                if *flatten || is_object(field) {
                    braces.extend(quote! (+ TypesenseQuery:: #len_name(&this. #raw_ident)));
                } else {
                    braces.extend(quote! (+ QueryState:: #len_name(&this. #raw_ident)));
                }
            }
        });

        tokens.extend(
            quote! (fn #extend_name(this: &mut Self, extend: &mut Vec<::typesensei::OrderedState>)),
        );
        Brace::default().surround(tokens, |braces| {
                braces.extend(quote!(use ::typesensei::{traits::TypesenseQuery, query::QueryState};));

                if !fields_has_id(&fields) {
                    braces.extend(quote! {
                        extend.extend(QueryState:: #take_name(&mut this.id).into_iter().map(|s| s.with_field("id")));
                    });
                }

                for field in fields.iter().filter(|f| !f.flatten && !is_object(f)) {
                    let Field {
                        raw_ident,
                        rename,
                        ..
                    } = field;

                    let name = rename.clone().unwrap_or_else(|| raw_ident.to_string());

                    braces.extend(quote! {
                        extend.extend(QueryState:: #take_name(&mut this. #raw_ident).into_iter().map(|s| s.with_field(#name)));
                    });
                }

                for field in fields.iter().filter(|f| f.flatten || is_object(f)) {
                    let Field {
                        raw_ident,
                        ..
                    } = field;

                    braces.extend(quote! {
                        TypesenseQuery:: #extend_name(&mut this. #raw_ident, extend);
                    });
                }
            });
    }
}
