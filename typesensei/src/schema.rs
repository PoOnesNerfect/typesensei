use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::fmt::Display;

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionSchema<'a> {
    pub name: &'a str,
    pub fields: Vec<Field<'a>>,
    pub default_sorting_field: Option<&'a str>,
}

impl<'a> CollectionSchema<'a> {
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            fields: Vec::new(),
            default_sorting_field: None,
        }
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
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Field<'a> {
    #[serde(rename = "type")]
    pub field_type: FieldType,
    pub name: &'a str,
    pub facet: Option<bool>,
    pub index: Option<bool>,
    pub optional: Option<bool>,
}

impl<'a> From<Field<'a>> for FieldOwned {
    fn from(f: Field<'a>) -> Self {
        let Field {
            field_type,
            name,
            facet,
            index,
            optional,
        } = f;

        Self {
            field_type,
            name: name.to_owned(),
            facet,
            index,
            optional,
            drop: None,
        }
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

    pub fn to_owned(&self) -> FieldOwned {
        let Field {
            field_type,
            name,
            facet,
            index,
            optional,
        } = self;

        FieldOwned {
            field_type: *field_type,
            name: (*name).to_owned(),
            facet: *facet,
            index: *index,
            optional: *optional,
            drop: None,
        }
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldOwned {
    #[serde(rename = "type")]
    pub field_type: FieldType,
    pub name: String,
    pub facet: Option<bool>,
    pub index: Option<bool>,
    pub optional: Option<bool>,
    pub drop: Option<bool>,
}

macro_rules! field_init_impl {
    ($($t:ident $(=> $n:expr)?),*) => {
        #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
        #[serde(rename_all = "lowercase")]
        pub enum FieldType {
            $(
                $(#[serde(rename = $n)])?
                $t,
            )*
        }

        impl Display for FieldType {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                Display::fmt(self.as_str(), f)
            }
        }

        impl FieldType {
            pub const fn as_str(&self) -> &'static str {
                match self {
                    $(
                        Self:: $t => field_init_impl!(@display $t $(=> $n)?),
                    )*
                }
            }
        }

        impl<'a> Field<'a> {
            $(
                paste::paste! {
                    pub fn [<$t:snake:lower>](name: &'a str) -> Self {
                        Self {
                            field_type: FieldType::$t,
                            name,
                            facet: None,
                            index: None,
                            optional: None,
                        }
                    }
                }
            )*
        }

        impl FieldOwned {
            $(
                paste::paste! {
                    pub fn [<$t:snake:lower>](name: String) -> Self {
                        Self {
                            field_type: FieldType::$t,
                            name,
                            facet: None,
                            index: None,
                            optional: None,
                            drop: None,
                        }
                    }
                }
            )*
        }
    };
    (@display $t:ident => $n:expr) => ($n);
    (@display $t:ident) => (stringify!(paste::paste!([<$t:snake:lower>])));
}

field_init_impl!(
    String,
    StringArray => "string[]",
    Int32,
    Int32Array => "int32[]",
    Int64,
    Int64Array => "int64[]",
    Float,
    FloatArray => "float[]",
    Bool,
    BoolArray => "bool[]",
    Geopoint,
    GeopointArray => "geopoint[]",
    StringAst => "string*",
    Auto
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
