use borrowme::borrowme;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionSchema<'a> {
    pub name: String,
    pub fields: Vec<Field<'a>>,
    pub default_sorting_field: Option<&'a str>,
    pub enable_nested_fields: bool,
    pub symbols_to_index: Option<Vec<String>>,
}

impl<'a> CollectionSchema<'a> {
    pub fn new(name: String) -> Self {
        Self {
            name,
            fields: Vec::new(),
            default_sorting_field: None,
            enable_nested_fields: false,
            symbols_to_index: None,
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn field(mut self, field: Field<'a>) -> Self {
        self.fields.push(field);
        self
    }

    pub fn extend(mut self, other: Self) -> Self {
        let Self {
            fields,
            default_sorting_field,
            ..
        } = other;

        if self.default_sorting_field.is_none() {
            self.default_sorting_field = default_sorting_field;
        }

        self.fields.extend(fields);

        self
    }

    pub fn default_sorting_field(mut self, default_sorting_field: &'a str) -> Self {
        self.default_sorting_field.replace(default_sorting_field);
        self
    }

    pub fn enable_nested_fields(mut self) -> Self {
        self.enable_nested_fields = true;
        self
    }

    pub fn symbols_to_index<T: AsRef<str>>(
        mut self,
        symbols_to_index: impl IntoIterator<Item = T>,
    ) -> Self {
        self.symbols_to_index.replace(
            symbols_to_index
                .into_iter()
                .map(|t| t.as_ref().to_owned())
                .collect(),
        );
        self
    }
}

#[borrowme]
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Field<'a> {
    #[serde(rename = "type")]
    pub field_type: &'a str,
    pub name: &'a str,
    pub facet: Option<bool>,
    pub index: Option<bool>,
    pub sort: Option<bool>,
    pub optional: Option<bool>,
    pub drop: Option<bool>,
}

impl<'a> Field<'a> {
    pub fn to_owned(&self) -> OwnedField {
        borrowme::to_owned(self)
    }
}

impl OwnedField {
    pub fn borrow(&self) -> Field {
        borrowme::borrow(self)
    }
}

impl<'a> Field<'a> {
    pub fn facet(mut self, facet: bool) -> Self {
        self.facet.replace(facet);
        self
    }

    pub fn index(mut self, index: bool) -> Self {
        self.index.replace(index);
        self
    }

    pub fn sort(mut self, sort: bool) -> Self {
        self.sort.replace(sort);
        self
    }

    pub fn optional(mut self, optional: bool) -> Self {
        self.optional.replace(optional);
        self
    }

    pub fn drop(mut self, should_drop: bool) -> Self {
        self.drop.replace(should_drop);
        self
    }
}

impl OwnedField {
    pub fn facet(mut self, facet: bool) -> Self {
        self.facet.replace(facet);
        self
    }

    pub fn index(mut self, index: bool) -> Self {
        self.index.replace(index);
        self
    }

    pub fn sort(mut self, sort: bool) -> Self {
        self.sort.replace(sort);
        self
    }

    pub fn optional(mut self, optional: bool) -> Self {
        self.optional.replace(optional);
        self
    }

    pub fn drop(mut self, should_drop: bool) -> Self {
        self.drop.replace(should_drop);
        self
    }
}

macro_rules! field_init_impl {
    ($($t:ident $(($array:ident))? $(=> $n:expr)?),*) => {
        impl<'a> Field<'a> {
            $(
                paste::paste! {
                    pub const [< $t:upper >] : &'static str = field_init_impl!(@display $t $(=> $n)?);
                    $(
                        pub const [< $t:upper _ARRAY>] : &'static str = field_init_impl!(@display $t ($array));
                    )?
                }
            )*

            $(
                paste::paste! {
                    pub fn [<$t:snake:lower>](name: &'a str) -> Self {
                        Self {
                            field_type: Field:: [< $t:upper >],
                            name,
                            facet: None,
                            index: None,
                            sort: None,
                            optional: None,
                            drop: None,
                        }
                    }

                    $(
                        pub fn [< $t:lower _ $array >] (name: &'a str) -> Self {
                            Self {
                                field_type: Field:: [< $t:upper _ARRAY >],
                                name,
                                facet: None,
                                index: None,
                                sort: None,
                                optional: None,
                                drop: None,
                            }
                        }
                    )?
                }
            )*
        }

        impl OwnedField {
            $(
                paste::paste! {
                    pub fn [<$t:snake:lower>](name: String) -> Self {
                        Self {
                            field_type: Field:: [< $t:upper >] .to_owned(),
                            name,
                            facet: None,
                            index: None,
                            sort: None,
                            optional: None,
                            drop: None,
                        }
                    }

                    $(
                        pub fn [< $t:lower _ $array >] (name: String) -> Self {
                            Self {
                                field_type: Field:: [< $t:upper _ARRAY >] .to_owned(),
                                name,
                                facet: None,
                                index: None,
                                sort: None,
                                optional: None,
                                drop: None,
                            }
                        }
                    )?
                }
            )*
        }
    };
    (@display $t:ident => $n:expr) => ($n);
    (@display $t:ident) => (stringify!($t));
    (@display $t:ident ($array:ident)) => (concat!(stringify!($t), stringify!([])));
}

field_init_impl!(
    string (array),
    int32 (array),
    int64 (array),
    float (array),
    bool (array),
    object (array),
    geopoint (array),
    stringast => "string*",
    auto
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_serde() {
        let field0 = Field::string_array("field0");
        assert_eq!(
            serde_json::to_string(&field0).unwrap(),
            r#"{"type":"string[]","name":"field0"}"#
        );

        let field1 = Field::int32("field1").facet(true);
        assert_eq!(
            serde_json::to_string(&field1).unwrap(),
            r#"{"type":"int32","name":"field1","facet":true}"#
        );
    }
}
