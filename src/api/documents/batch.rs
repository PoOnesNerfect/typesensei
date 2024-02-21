use crate::Typesense;

use super::{BatchResult, Documents};
use std::{
    future::{Future, IntoFuture},
    marker::PhantomData,
};

#[derive(Debug)]
pub struct DocumentBatchAction<'a, T: Typesense, Fut: 'a> {
    api: &'a Documents<'a, T>,
    documents: &'a [T::Partial],
    action: Option<&'a str>,
    dirty_values: Option<&'a str>,
    batch_size: Option<&'a str>,
    fut: Fut,
    _phantom: PhantomData<Fut>,
}

impl<'a, T: Typesense, Fut: 'a> DocumentBatchAction<'a, T, Fut> {
    pub(crate) fn new(
        api: &'a Documents<'a, T>,
        action: Option<&'a str>,
        documents: &'a [T::Partial],
        fut: Fut,
    ) -> DocumentBatchAction<'a, T, Fut> {
        DocumentBatchAction {
            api,
            documents,
            action,
            dirty_values: None,
            batch_size: None,
            fut,
            _phantom: PhantomData,
        }
    }

    pub fn dirty_values(
        mut self,
        dirty_values: &'a str,
    ) -> DocumentBatchAction<'a, T, impl 'a + Future<Output = BatchResult>> {
        self.dirty_values.replace(dirty_values);
        self.reset()
    }

    pub fn batch_size(
        mut self,
        batch_size: &'a str,
    ) -> DocumentBatchAction<'a, T, impl 'a + Future<Output = BatchResult>> {
        self.batch_size.replace(batch_size);
        self.reset()
    }

    fn reset(self) -> DocumentBatchAction<'a, T, impl 'a + Future<Output = BatchResult>> {
        let Self {
            api,
            documents,
            action,
            dirty_values,
            batch_size,
            ..
        } = self;

        let query = [
            ("action", action),
            ("dirty_values", dirty_values),
            ("batch_size", batch_size),
        ];

        DocumentBatchAction::new(api, action, documents, api.batch_action(query, documents))
    }
}

impl<'a, T: Typesense, Fut: 'a + Future<Output = BatchResult>> IntoFuture
    for DocumentBatchAction<'a, T, Fut>
{
    type Output = Fut::Output;
    type IntoFuture = Fut;

    fn into_future(self) -> Self::IntoFuture {
        self.fut
    }
}
