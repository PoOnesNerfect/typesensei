use crate::traits::TypesenseField;
use serde::{de::Deserializer, Deserialize, Serialize};
use std::mem;

pub fn set<T>(t: T) -> FieldState<T> {
    FieldState::Set(t)
}

pub fn unset<T>() -> FieldState<T> {
    FieldState::NotSet
}

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
            set(val)
        } else {
            unset()
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
            Ok(set(ret))
        } else {
            Ok(unset())
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

    pub fn is_unset(&self) -> bool {
        matches!(self, FieldState::NotSet)
    }

    pub fn take(&mut self) -> Option<T> {
        let ret = mem::replace(self, FieldState::NotSet);

        match ret {
            FieldState::Set(t) => Some(t),
            FieldState::NotSet => None,
        }
    }

    pub fn inner(&self) -> Option<&T> {
        match self {
            FieldState::Set(t) => Some(t),
            FieldState::NotSet => None,
        }
    }

    pub fn inner_mut(&mut self) -> Option<&mut T> {
        match self {
            FieldState::Set(t) => Some(t),
            FieldState::NotSet => None,
        }
    }

    pub fn into_inner(mut self) -> Option<T> {
        self.take()
    }

    pub fn unwrap(mut self) -> T {
        self.take().unwrap()
    }

    pub fn expect(mut self, msg: &str) -> T {
        self.take().expect(msg)
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
