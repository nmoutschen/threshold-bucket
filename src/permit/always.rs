//! # Always permits

use std::sync::Arc;

use crate::inner::Inner;

use super::{InnerPermit, Permit, Permitter};

pub(crate) struct AlwaysPermitter {
    inner: Arc<Inner>,
}

impl AlwaysPermitter {
    pub(crate) fn new(inner: Arc<Inner>) -> Self {
        Self { inner }
    }
}

impl Permitter for AlwaysPermitter {
    fn get_permit(&self) -> Option<Permit> {
        Some(Permit::new(AlwaysPermit, Arc::downgrade(&self.inner)))
    }

    fn belongs(&self, permit: &Permit) -> bool {
        permit
            .addr()
            .map(|inner| std::ptr::eq(Arc::as_ptr(&self.inner).cast(), inner))
            .unwrap_or(false)
    }
}

pub(crate) struct AlwaysPermit;

impl InnerPermit for AlwaysPermit {
    fn notify(&self, _num: u64) {}
}
