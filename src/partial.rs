use __private::SerdeImpl;
use std::collections::HashMap;

pub trait Partial: SerdeImpl {
    type Partial: SerdeImpl;

    fn into_partial(self) -> Self::Partial;
    fn from_partial(partial: Self::Partial) -> Result<Self, TryFromPartialError>;
}

mod __private {
    use std::fmt;

    pub trait SerdeImpl:
        Sized + fmt::Debug + serde::Serialize + for<'de> serde::Deserialize<'de>
    {
    }
    impl<T> SerdeImpl for T where
        T: Sized + fmt::Debug + serde::Serialize + for<'de> serde::Deserialize<'de>
    {
    }
}

impl<T: Partial> Partial for Option<T>
where
    T: serde::Serialize,
    for<'de> T: serde::Deserialize<'de>,
    for<'de> T::Partial: serde::Deserialize<'de>,
{
    type Partial = Option<T::Partial>;

    fn into_partial(self) -> Self::Partial {
        self.map(|t| t.into_partial())
    }

    fn from_partial(partial: Self::Partial) -> Result<Self, TryFromPartialError> {
        partial.map(T::from_partial).transpose()
    }
}

impl<T: SerdeImpl> Partial for Vec<T> {
    type Partial = Self;

    fn into_partial(self) -> Self::Partial {
        self
    }

    fn from_partial(partial: Self::Partial) -> Result<Self, TryFromPartialError> {
        Ok(partial)
    }
}

impl<T: SerdeImpl> Partial for Box<T> {
    type Partial = Self;

    fn into_partial(self) -> Self::Partial {
        self
    }

    fn from_partial(partial: Self::Partial) -> Result<Self, TryFromPartialError> {
        Ok(partial)
    }
}

impl<K: SerdeImpl + std::cmp::Eq + std::hash::Hash, V: SerdeImpl> Partial for HashMap<K, V> {
    type Partial = Self;

    fn into_partial(self) -> Self::Partial {
        self
    }

    fn from_partial(partial: Self::Partial) -> Result<Self, TryFromPartialError> {
        Ok(partial)
    }
}

impl<'a> Partial for &'a str
where
    for<'de> &'a str: serde::Deserialize<'de>,
{
    type Partial = Self;

    fn into_partial(self) -> Self::Partial {
        self
    }

    fn from_partial(partial: Self::Partial) -> Result<Self, TryFromPartialError> {
        Ok(partial)
    }
}

macro_rules! impl_partial {
    ($($t:ty),*) => {
        $(
            impl Partial for $t {
                type Partial = $t;

                fn into_partial(self) -> Self::Partial {
                    self
                }

                fn from_partial(partial: Self::Partial) -> Result<Self, TryFromPartialError> {
                    Ok(partial)
                }
            }
        )*
    };
}

impl_partial!(
    bool,
    u8,
    u16,
    i8,
    i16,
    i32,
    u32,
    u64,
    usize,
    i64,
    f32,
    f64,
    String,
    serde_json::Value
);

use thiserror::Error;
use tosserror::Toss;

#[derive(Debug, Clone, Error, Toss)]
#[error("field {field} is missing in partial object for {type_name}")]
pub struct TryFromPartialError {
    pub type_name: &'static str,
    pub field: &'static str,
}
