// Rust translation of the cached-range members of FoundationClasses/TKMath/Poly/Poly_Triangulation.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

use super::{BoundingBox3, Triangulation};
use crate::gp::{Transform3, TransformForm};
impl Triangulation {
    /// Returns the cached range, void when none is cached; `triangulation.cached_min_max()`.
    pub fn cached_min_max(&self) -> BoundingBox3 {
        self.cached_min_max.read().clone().unwrap_or_default()
    }
    /// Caches a range; a void box removes the cache; `triangulation.set_cached_min_max(&range)`.
    pub fn set_cached_min_max(&self, range: &BoundingBox3) {
        if range.is_void() {
            self.unset_cached_min_max();
            return;
        }
        *self.cached_min_max.write() = Some(range.clone());
    }
    /// True when a range is cached; `triangulation.has_cached_min_max()`.
    pub fn has_cached_min_max(&self) -> bool {
        self.cached_min_max.read().is_some()
    }
    /// Caches the exact bounding box of the nodes; `triangulation.update_cached_min_max()`.
    pub fn update_cached_min_max(&self) {
        let mut range = BoundingBox3::default();
        self.min_max(&mut range, Transform3::default(), true);
        self.set_cached_min_max(&range);
    }
    /// Protected `unsetCachedMinMax`: drops the cached range; `triangulation.unset_cached_min_max()`.
    pub fn unset_cached_min_max(&self) {
        *self.cached_min_max.write() = None;
    }
    /// Extends `range` by this mesh; the cache serves when there is no geometry, when not
    /// `accurate`, or when `transform` has no rotation part. Returns false when nothing
    /// could be added; `triangulation.min_max(&mut range, transform, false)`.
    pub fn min_max(&self, range: &mut BoundingBox3, transform: Transform3, accurate: bool) -> bool {
        let cached = self.cached_min_max.read().clone();
        let from_cache = cached.filter(|_| {
            !self.has_geometry() || !accurate || keeps_axis_alignment(transform.form())
        });
        let found = match from_cache {
            Some(cached) => cached.transformed(transform),
            None => self.compute_bounding_box(transform),
        };
        if found.is_void() {
            return false;
        }
        range.add_box(&found);
        true
    }
    /// Protected `computeBoundingBox`: the box of every (transformed) node;
    /// dispatches custom hooks; `triangulation.compute_bounding_box(Transform3::default())`.
    pub fn compute_bounding_box(&self, transform: Transform3) -> BoundingBox3 {
        if let Some(bounds) = self
            .hooks
            .as_ref()
            .and_then(|hooks| hooks.compute_bounding_box(self, transform))
        {
            return bounds;
        }
        self.compute_bounding_box_base(transform)
    }
    /// Explicit base-qualified equivalent of `Poly_Triangulation::computeBoundingBox`;
    /// `triangulation.compute_bounding_box_base(transform)`.
    pub fn compute_bounding_box_base(&self, transform: Transform3) -> BoundingBox3 {
        let fields = self.fields.read();
        let mut range = BoundingBox3::default();
        let identity = transform.form() == TransformForm::Identity;
        for offset in 0..fields.nodes.len() {
            // Offsets below the length are always readable.
            let Ok(node) = fields.nodes.value(offset) else {
                continue;
            };
            range.add_point(if identity {
                node
            } else {
                node.transformed(transform)
            });
        }
        range
    }
}
fn keeps_axis_alignment(form: TransformForm) -> bool {
    matches!(
        form,
        TransformForm::Identity
            | TransformForm::Translation
            | TransformForm::PointMirror
            | TransformForm::Scale
    )
}
