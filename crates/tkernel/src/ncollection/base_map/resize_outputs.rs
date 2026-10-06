//! Legacy output parameters retain previous pending tables on a native resize no-op.
use super::{
    BaseDoubleMapResize, BaseDoubleMapStorage, BaseMapResize, BaseMapStorage, MapLinkNode,
};
use crate::standard::Failure;

impl BaseMapStorage {
    /// Preserves native `beginResize` output assignment order; `map.begin_resize_into(0, &mut count, &mut pending)?`.
    /// The owned pending table replaces raw ownership transfer; a superseded pending table is freed.
    pub fn begin_resize_into(
        &self,
        extent: usize,
        count: &mut usize,
        pending: &mut Option<BaseMapResize>,
    ) -> Result<bool, Failure> {
        *count = super::next_resize_count(extent, self.bucket_count, self.table.is_some());
        let plan = self.begin_resize(extent)?;
        if !plan.is_resize() {
            return Ok(false);
        }
        *pending = Some(plan);
        Ok(true)
    }
    /// Clears the obsolete second output even when resize is unnecessary; `map.begin_resize_legacy_into(0, &mut count, &mut pending, &mut second)?`.
    pub fn begin_resize_legacy_into(
        &self,
        extent: usize,
        count: &mut usize,
        pending: &mut Option<BaseMapResize>,
        second: &mut *mut *mut MapLinkNode,
    ) -> Result<bool, Failure> {
        *second = std::ptr::null_mut();
        self.begin_resize_into(extent, count, pending)
    }
    /// Ignores native legacy extent/second-table arguments and adopts the pending first table; `map.end_resize_legacy(0, plan, std::ptr::null_mut())`.
    pub fn end_resize_legacy(
        &mut self,
        _extent: usize,
        pending: BaseMapResize,
        _second: *mut *mut MapLinkNode,
    ) {
        self.end_resize(pending);
    }
}

impl BaseDoubleMapStorage {
    /// Preserves both native table outputs on a resize no-op; `map.begin_resize_into(0, &mut count, &mut pending)?`.
    pub fn begin_resize_into(
        &self,
        extent: usize,
        count: &mut usize,
        pending: &mut Option<BaseDoubleMapResize>,
    ) -> Result<bool, Failure> {
        *count = super::next_resize_count(
            extent,
            self.base().bucket_count,
            self.base().table.is_some(),
        );
        let plan = self.begin_resize(extent)?;
        if !plan.is_resize() {
            return Ok(false);
        }
        *pending = Some(plan);
        Ok(true)
    }
}
