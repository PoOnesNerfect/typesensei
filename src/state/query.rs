use super::OrderedState;
use crate::{
    traits::{TypesenseField, TypesenseQuery},
    SearchQuery,
};
use std::{
    cell::RefCell,
    fmt::{self, Write},
    marker::PhantomData,
    mem,
    rc::Rc,
};

#[derive(Debug)]
pub struct QueryState<T: TypesenseField> {
    filter_by: Vec<OrderedState>,
    query_by: Option<OrderedState>,
    sort_by: Option<OrderedState>,
    counter: Rc<RefCell<u16>>,
    _phantom: PhantomData<T>,
}

impl<T: TypesenseField> QueryState<T>
where
    <T as TypesenseField>::Type: 'static,
{
    pub fn new(counter: Rc<RefCell<u16>>) -> Self {
        Self {
            counter,
            filter_by: Vec::new(),
            query_by: None,
            sort_by: None,
            _phantom: PhantomData,
        }
    }

    pub fn equals(&mut self, val: T::Type) -> &mut Self {
        self.filter_by(FilterBy::equals(val))
    }

    pub fn not_equals(&mut self, val: T::Type) -> &mut Self {
        self.filter_by(FilterBy::not_equals(val))
    }

    pub fn greater_than(&mut self, val: T::Type) -> &mut Self {
        self.filter_by(FilterBy::greater_than(val))
    }

    pub fn greater_or_equals(&mut self, val: T::Type) -> &mut Self {
        self.filter_by(FilterBy::greater_equals(val))
    }

    pub fn less_than(&mut self, val: T::Type) -> &mut Self {
        self.filter_by(FilterBy::less_than(val))
    }

    pub fn less_or_equals(&mut self, val: T::Type) -> &mut Self {
        self.filter_by(FilterBy::less_equals(val))
    }

    pub fn in_range(&mut self, val: String) -> &mut Self {
        self.filter_by(FilterBy::in_range(val))
    }

    pub fn is_one_of(&mut self, items: &[T::Type]) -> &mut Self {
        let one_of = if !items.is_empty() {
            let mut one_of = format!("{}", items[0]);
            for item in items.iter().skip(1) {
                let _ = write!(&mut one_of, ",{}", item);
            }

            one_of
        } else {
            Default::default()
        };

        self.filter_by(FilterBy::one_of(one_of))
    }

    fn filter_by<S: 'static + __priv::DebugDisplay>(
        &mut self,
        filter_by: FilterBy<S>,
    ) -> &mut Self {
        let order = self.inc_counter();

        self.filter_by
            .push(OrderedState::new(order).with(filter_by));

        self
    }

    pub fn query_by(&mut self) -> &mut Self {
        let order = self.inc_counter();

        self.query_by.replace(OrderedState::new(order));

        self
    }

    pub fn sort_asc(&mut self) -> &mut Self {
        self.sort_by(SortBy::Asc)
    }

    pub fn sort_desc(&mut self) -> &mut Self {
        self.sort_by(SortBy::Desc)
    }

    fn sort_by(&mut self, sort_by: SortBy) -> &mut Self {
        let order = self.inc_counter();

        self.sort_by.replace(OrderedState::new(order).with(sort_by));

        self
    }

    fn inc_counter(&mut self) -> u16 {
        let mut counter = self.counter.borrow_mut();
        let ret = *counter;
        *counter += 1;
        ret
    }
}

impl<T: TypesenseField> QueryState<T>
where
    <T as TypesenseField>::Type: 'static,
{
    pub fn take_filter_by(this: &mut Self) -> Vec<OrderedState> {
        mem::replace(&mut this.filter_by, Vec::new())
    }

    pub fn filter_by_len(this: &Self) -> usize {
        this.filter_by.len()
    }

    pub fn take_query_by(this: &mut Self) -> Option<OrderedState> {
        this.query_by.take()
    }

    pub fn query_by_len(this: &Self) -> usize {
        this.query_by.is_some().then_some(1).unwrap_or(0)
    }

    pub fn take_sort_by(this: &mut Self) -> Option<OrderedState> {
        this.sort_by.take()
    }

    pub fn sort_by_len(this: &Self) -> usize {
        this.sort_by.is_some().then_some(1).unwrap_or(0)
    }
}

#[derive(Debug, Default)]
pub struct QueryBuilder {
    filter_by: Vec<OrderedState>,
    query_by: Vec<OrderedState>,
    sort_by: Vec<OrderedState>,
    counter: Rc<RefCell<u16>>,
}

impl QueryBuilder {
    pub fn q(self, query: String) -> SearchQuery {
        TypesenseQuery::q(self, query)
    }

    pub fn filter_by(&mut self, cond: String) -> &mut Self {
        let order = self.inc_counter();
        self.filter_by.push(OrderedState::new(order).with(cond));

        self
    }

    pub fn query_by(&mut self, cond: String) -> &mut Self {
        let order = self.inc_counter();
        self.query_by.push(OrderedState::new(order).with(cond));

        self
    }

    pub fn sort_by(&mut self, cond: String) -> &mut Self {
        let order = self.inc_counter();
        self.sort_by.push(OrderedState::new(order).with(cond));

        self
    }

    fn inc_counter(&mut self) -> u16 {
        let mut counter = self.counter.borrow_mut();
        let ret = *counter;
        *counter += 1;
        ret
    }
}

impl TypesenseQuery for QueryBuilder {
    fn with_counter(counter: Rc<RefCell<u16>>) -> Self {
        Self {
            counter,
            ..Default::default()
        }
    }

    fn extend_filter_by(this: &mut Self, extend: &mut Vec<OrderedState>) {
        extend.extend(this.filter_by.drain(..));
    }

    fn filter_by_len(this: &Self) -> usize {
        this.filter_by.len()
    }

    fn extend_query_by(this: &mut Self, extend: &mut Vec<OrderedState>) {
        extend.extend(this.query_by.drain(..));
    }

    fn query_by_len(this: &Self) -> usize {
        this.query_by.len()
    }

    fn extend_sort_by(this: &mut Self, extend: &mut Vec<OrderedState>) {
        extend.extend(this.sort_by.drain(..));
    }

    fn sort_by_len(this: &Self) -> usize {
        this.sort_by.len()
    }
}

#[derive(Debug, Clone, Copy)]
enum SortBy {
    Asc,
    Desc,
}

impl fmt::Display for SortBy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SortBy::Asc => write!(f, ":asc"),
            SortBy::Desc => write!(f, ":desc"),
        }
    }
}

__priv::impl_filter_init!(
    Equals: ":={}",
    NotEquals: ":!={}",
    LessThan: ":<{}",
    LessEquals: ":<={}",
    GreaterThan: ":>{}",
    GreaterEquals: ":>={}",
    InRange: ":[{}]",
    OneOf: ":[{}]"
);

mod __priv {
    use std::fmt;

    pub trait DebugDisplay: fmt::Display + fmt::Debug {}
    impl<T: fmt::Debug + fmt::Display> DebugDisplay for T {}

    macro_rules! impl_filter_init {
        ($($f:ident : $p:expr),*) => {
            enum FilterBy<T> {
                $(
                    $f (T)
                ),*
            }

            impl<T> FilterBy<T> {
                $(crate::state::query::__priv::impl_filter_init!(@fn $f: $p);)*
            }

            impl<T: fmt::Display> fmt::Display for FilterBy<T> {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    use FilterBy::*;

                    match &self {
                        $(
                            $f (t) => write!(f, $p, t),
                        )*
                    }
                }
            }

            impl<T: fmt::Display> fmt::Debug for FilterBy<T> {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    use FilterBy::*;

                    match &self {
                        $(
                            $f (t) => write!(f, "{}({})", stringify!($f), t),
                        )*
                    }
                }
            }
        };
        (@fn $f:ident : $p:expr) => {
            paste::paste! {
            pub fn [<$f:snake:lower>] (val: T) -> Self {
                FilterBy:: $f (val)
            }
            }
        }
    }

    pub(crate) use impl_filter_init;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rc() {
        let mut rc = Rc::new(0u16);

        let rc2 = rc.clone();

        // let mut muta = Rc::get_mut(&mut rc).unwrap();
    }
}
