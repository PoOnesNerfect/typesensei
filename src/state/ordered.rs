use std::fmt;

#[derive(Debug)]
pub struct OrderedState {
    order: u16,
    field: Option<&'static str>,
    inner: Option<Box<dyn 'static + __priv::DebugDisplay>>,
}

impl OrderedState {
    pub fn new(order: u16) -> Self {
        Self {
            order,
            field: None,
            inner: None,
        }
    }

    pub fn with<T: 'static + __priv::DebugDisplay>(mut self, inner: T) -> Self {
        self.inner.replace(Box::new(inner));

        self
    }

    pub fn with_field(mut self, field: &'static str) -> Self {
        self.field.replace(field);
        self
    }
}

impl fmt::Display for OrderedState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(field) = self.field {
            write!(f, "{field}")?;
        }

        if let Some(inner) = &self.inner {
            write!(f, "{inner}")?;
        }

        Ok(())
    }
}

impl PartialEq for OrderedState {
    fn eq(&self, other: &Self) -> bool {
        self.order.eq(&other.order)
    }
}

impl Eq for OrderedState {}

impl PartialOrd for OrderedState {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.order.partial_cmp(&other.order)
    }
}

impl Ord for OrderedState {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.order.cmp(&other.order)
    }
}

mod __priv {
    use std::fmt;

    pub trait DebugDisplay: fmt::Display + fmt::Debug {}
    impl<T: fmt::Debug + fmt::Display> DebugDisplay for T {}
}
