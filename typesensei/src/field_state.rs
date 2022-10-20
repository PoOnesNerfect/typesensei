use serde::{de::Deserializer, Deserialize, Serialize};
use std::mem;

use crate::TypesenseField;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FieldState<T> {
    state: FieldStateInner<T>,
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
        self.state.serialize(serializer)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum FieldStateInner<T> {
    Set(T),
    NotSet,
}

impl<T> Default for FieldState<T> {
    fn default() -> Self {
        Self {
            state: FieldStateInner::NotSet,
        }
    }
}

impl<T: Serialize> Serialize for FieldStateInner<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            FieldStateInner::Set(t) => t.serialize(serializer),
            FieldStateInner::NotSet => serializer.serialize_none(),
        }
    }
}

impl<T> FieldState<T> {
    pub fn new(value: T) -> Self {
        Self {
            state: FieldStateInner::Set(value),
        }
    }

    pub fn set(&mut self, value: T) -> &mut Self {
        let _ = mem::replace(&mut self.state, FieldStateInner::Set(value));

        self
    }

    pub fn unset(&mut self) -> &mut Self {
        let _ = mem::replace(&mut self.state, FieldStateInner::NotSet);

        self
    }

    pub fn is_set(&self) -> bool {
        matches!(self.state, FieldStateInner::Set(_))
    }

    pub fn not_set() -> Self {
        Self {
            state: FieldStateInner::NotSet,
        }
    }

    pub fn is_not_set(&self) -> bool {
        matches!(self.state, FieldStateInner::NotSet)
    }

    pub fn take(&mut self) -> Option<T> {
        let ret = mem::replace(&mut self.state, FieldStateInner::NotSet);

        match ret {
            FieldStateInner::Set(t) => Some(t),
            FieldStateInner::NotSet => None,
        }
    }

    pub fn value(&self) -> Option<&T> {
        match &self.state {
            FieldStateInner::Set(t) => Some(t),
            FieldStateInner::NotSet => None,
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
        match &self.state {
            FieldStateInner::Set(t) => t.is_none(),
            FieldStateInner::NotSet => true,
        }
    }
}
