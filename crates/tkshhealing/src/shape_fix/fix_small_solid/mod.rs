//! Small-solid thresholds, removal, and shell merging from `ShapeFix_FixSmallSolid`.
mod config;
mod merge;
mod remove;

use self::config::SmallSolidConfig;
use super::{FixRoot, FixRootData, ShapeFixRoot};
use crate::shape_build::ShapeBuildReShape;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, SharedState};

/// `ShapeFix_FixSmallSolid`: removes small solids or merges them into adjacent solids.
/// Threshold mode `0` uses both tests, mode `1` only width factor, and mode `2` only volume.
///
/// ```
/// use rustycad_tkshhealing::shape_fix::ShapeFixFixSmallSolid;
/// let fixer = ShapeFixFixSmallSolid::new();
/// fixer.set_fix_mode(2);
/// fixer.set_volume_threshold(0.01);
/// ```
#[derive(Default)]
pub struct ShapeFixFixSmallSolid {
    parent: ShapeFixRoot,
    config: SharedState<SmallSolidConfig>,
}

impl_transient!(ShapeFixFixSmallSolid, "ShapeFix_FixSmallSolid", ShapeFixRoot, parent, interfaces: [dyn FixRoot]);

impl ShapeFixFixSmallSolid {
    /// Constructs with mode `0` and disabled thresholds; `ShapeFixFixSmallSolid::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `SetFixMode(mode)`: invalid modes reset to `0`; `fixer.set_fix_mode(2)`.
    pub fn set_fix_mode(&self, mode: i32) {
        self.config.write().mode = if (0..=2).contains(&mode) { mode } else { 0 };
    }

    /// Sets a nonnegative volume threshold; a negative or NaN value disables it.
    /// `fixer.set_volume_threshold(0.01)`.
    pub fn set_volume_threshold(&self, threshold: f64) {
        self.config.write().volume_threshold = normalize_threshold(threshold);
    }

    /// Sets a nonnegative width-factor threshold; a negative or NaN value disables it.
    /// `fixer.set_width_factor_threshold(0.01)`.
    pub fn set_width_factor_threshold(&self, threshold: f64) {
        self.config.write().width_factor_threshold = normalize_threshold(threshold);
    }

    /// Removes qualifying solids and returns the context-applied shape; `fixer.remove(&shape, &context)?`.
    pub fn remove(
        &self,
        shape: &Shape,
        context: &Handle<ShapeBuildReShape>,
    ) -> Result<Shape, Failure> {
        remove::remove(self, shape, context)
    }

    /// Merges adjacent qualifying solids and returns the context-applied shape; `fixer.merge(&shape, &context)?`.
    pub fn merge(
        &self,
        shape: &Shape,
        context: &Handle<ShapeBuildReShape>,
    ) -> Result<Shape, Failure> {
        merge::merge(self, shape, context)
    }

    fn config(&self) -> SmallSolidConfig {
        self.config.read().clone()
    }
}

impl FixRoot for ShapeFixFixSmallSolid {
    fn fix_root_data(&self) -> &FixRootData {
        self.parent.fix_root_data()
    }
}

fn normalize_threshold(threshold: f64) -> f64 {
    if threshold >= 0.0 {
        threshold
    } else {
        precision::infinite()
    }
}

#[cfg(test)]
mod tests {
    use super::{FixRoot, ShapeFixFixSmallSolid};
    use rustycad_tkernel::standard::Transient;

    #[test]
    fn root_state_is_shared_through_native_parent_type() {
        let fixer = ShapeFixFixSmallSolid::new();
        fixer.set_precision(0.02);
        assert_eq!(FixRoot::precision(&fixer), 0.02);
        assert_eq!(
            fixer.dynamic_type().parent().unwrap().name(),
            "ShapeFix_Root"
        );
    }
}
