use std::sync::Weak;

use crate::inner::Inner;

/// Permit for acquiring tokens
#[non_exhaustive]
pub struct Permit(pub(crate) Weak<Inner>);

impl Permit {
    pub(crate) fn new(bucket: Weak<Inner>) -> Self {
        Self(bucket)
    }
}
