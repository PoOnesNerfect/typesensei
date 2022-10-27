use serde::{de::Deserializer, Deserialize, Serialize};
use std::mem;
use crate::traits::TypesenseField;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FieldState<T> {
    Set(T),
    NotSet,
}

impl<T> Default for FieldState<T> {
    fn default() -> Self {
        Self::NotSet
    }
}

impl<T> From<Option<T>> for FieldState<T> {
    fn from(t: Option<T>) -> Self {
        if let Some(val) = t {
            Self::new(val)
        } else {
            Self::not_set()
        }
    }
}

impl<T: TypesenseField> From<T> for FieldState<T> {
    fn from(t: T) -> Self {
        Self::new(t)
    }
}

impl<T: Serialize> Serialize for FieldState<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            FieldState::Set(t) => t.serialize(serializer),
            FieldState::NotSet => serializer.serialize_none(),
        }
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for FieldState<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if let Some(ret) = Option::<T>::deserialize(deserializer)? {
            Ok(Self::new(ret))
        } else {
            Ok(Self::not_set())
        }
    }
}

impl<T> FieldState<T> {
    pub fn new(value: T) -> Self {
        FieldState::Set(value)
    }

    pub fn set(&mut self, value: T) -> &mut Self {
        *self = FieldState::Set(value);

        self
    }

    pub fn unset(&mut self) -> &mut Self {
        *self = FieldState::NotSet;

        self
    }

    pub fn is_set(&self) -> bool {
        matches!(self, FieldState::Set(_))
    }

    pub fn not_set() -> Self {
        FieldState::NotSet
    }

    pub fn is_not_set(&self) -> bool {
        matches!(self, FieldState::NotSet)
    }

    pub fn take(&mut self) -> Option<T> {
        let ret = mem::replace(self, FieldState::NotSet);

        match ret {
            FieldState::Set(t) => Some(t),
            FieldState::NotSet => None,
        }
    }

    pub fn value(&self) -> Option<&T> {
        match self {
            FieldState::Set(t) => Some(t),
            FieldState::NotSet => None,
        }
    }

    pub fn into_value(mut self) -> Option<T> {
        self.take()
    }

    pub fn unwrap(mut self) -> T {
        self.take().unwrap()
    }
}

impl<T> FieldState<Option<T>> {
    pub fn is_inner_option_none(&self) -> bool {
        match self {
            FieldState::Set(t) => t.is_none(),
            FieldState::NotSet => true,
        }
    }
}
