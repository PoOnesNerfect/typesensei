use super::Documents;
use crate::{Error, Typesense};
use std::{
    future::{Future, IntoFuture},
    marker::PhantomData,
};

#[derive(Debug)]
pub struct DocumentAction<'a, T: Typesense, Fut: 'a> {
    api: &'a Documents<'a, T>,
    document: &'a T,
    action: Option<&'a str>,
    dirty_values: Option<&'a str>,
    fut: Fut,
    _phantom: PhantomData<Fut>,
}

impl<'a, T: Typesense, Fut: 'a> DocumentAction<'a, T, Fut> {
    pub(crate) fn new(
        api: &'a Documents<'a, T>,
        document: &'a T,
        fut: Fut,
    ) -> DocumentAction<'a, T, Fut> {
        DocumentAction {
            api,
            document,
            action: None,
            dirty_values: None,
            fut,
            _phantom: PhantomData,
        }
    }

    pub fn action(
        mut self,
        action: &'a str,
    ) -> DocumentAction<'a, T, impl 'a + Future<Output = Result<(), Error>>> {
        self.action.replace(action);
        self.reset()
    }

    pub fn dirty_values(
        mut self,
        dirty_values: &'a str,
    ) -> DocumentAction<'a, T, impl 'a + Future<Output = Result<(), Error>>> {
        self.dirty_values.replace(dirty_values);
        self.reset()
    }

    fn reset(self) -> DocumentAction<'a, T, impl 'a + Future<Output = Result<(), Error>>> {
        let Self {
            api,
            document,
            action,
            dirty_values,
            ..
        } = self;

        let query = [
            action.map(|a| ("action", a)),
            dirty_values.map(|d| ("dirty_values", d)),
        ]
        .into_iter()
        .filter_map(|x| x);

        DocumentAction::new(api, document, api.action(query, document))
    }
}

impl<'a, T: Typesense, Fut: 'a + Future<Output = Result<(), Error>>> IntoFuture
    for DocumentAction<'a, T, Fut>
{
    type Output = Fut::Output;
    type IntoFuture = Fut;

    fn into_future(self) -> Self::IntoFuture {
        self.fut
    }
}
