// SPDX-License-Identifier: GPL-2.0-only
//! Authored tokens are meaningful only within their operation and later caller dictionary.
use super::*;
use crate::sketch_region::OriginalMemberIdentity;
pub(crate) struct PrismRefusalIdentityContext<'a> {
    pub(crate) bindings: &'a PrismPublicationBindings,
}
impl PrismRefusalIdentityContext<'_> {
    pub(super) fn original(
        &self,
        actual: &OriginalMemberIdentity,
    ) -> Result<FrozenOriginalKey, PrismRefusalProjectionGap> {
        let key = super::super::identities::original_key(self.bindings, actual)
            .and_then(|key| self.check_key(key));
        key.map_err(|cause| PrismRefusalProjectionGap::Original {
            source: actual.source.0,
            cause: Box::new(cause),
        })
    }
    fn check_key(&self, key: FrozenOriginalKey) -> Result<FrozenOriginalKey, PrismFreezeCause> {
        if key.member.0 == 0 || key.directed_use.0 == 0 {
            return Err(super::super::bindings::identity(
                "nonzero original member/directed-use tokens",
                0,
            ));
        }
        let count = self.key_count(&key);
        if count != 1 {
            return Err(super::super::bindings::identity(
                "one actual original per authored original key",
                count,
            ));
        }
        Ok(key)
    }
    fn key_count(&self, key: &FrozenOriginalKey) -> usize {
        self.bindings
            .originals
            .iter()
            .filter(|binding| {
                binding.actual.source.0 == key.source
                    && binding.member == key.member
                    && binding.directed_use == key.directed_use
            })
            .count()
    }
}
impl<T> RetainedProjection<T> {
    pub(super) fn map<U>(
        mut self,
        path: PrismRefusalPath,
        convert: impl FnOnce(T) -> U,
    ) -> Box<RetainedProjection<U>> {
        self.issue.path.insert(0, path);
        Box::new(RetainedProjection {
            issue: self.issue,
            original: convert(self.original),
        })
    }
    pub(super) fn gap(
        original: T,
        path: PrismRefusalPath,
        cause: PrismRefusalProjectionGap,
    ) -> Box<Self> {
        let issue = PrismRefusalProjectionIssue {
            path: vec![path],
            cause,
        };
        Box::new(Self { issue, original })
    }
    pub(super) fn pending(
        original: T,
        path: PrismRefusalPath,
        payload: PendingPrismRefusalPayload,
    ) -> Box<Self> {
        let cause = PrismRefusalProjectionGap::Pending(payload);
        Self::gap(original, path, cause)
    }
}
