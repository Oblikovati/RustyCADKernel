//! Minimum screen size uses the native scaled-above persistence type.
use super::AisPlane;
use crate::prs_mgr::*;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkservice::graphic3d::Graphic3dTransformPersScaledAbove;
impl AisPlane {
    /// Keep a lower zoom scale, clearing persistence for nonpositive values; `plane.set_minimum_size(10.)?`.
    pub fn set_minimum_size(&self, value: f64) -> Result<(), Failure> {
        if value <= 0. {
            self.unset_minimum_size();
            return Ok(());
        }
        let (mut x, mut y) = (0., 0.);
        self.size(&mut x, &mut y);
        // std::min compares the second operand, preserving native NaN operand ordering.
        let minimum = if y < x { y } else { x };
        let persistence = Handle::new(Graphic3dTransformPersScaledAbove::new(
            minimum / value,
            &self.center(),
        )?);
        self.set_transform_persistence(&persistence.cast());
        Ok(())
    }
    /// Clear any persistence, including unrelated kinds; `plane.unset_minimum_size()`.
    pub fn unset_minimum_size(&self) {
        self.set_transform_persistence(&Handle::default());
    }
    /// Match the native concrete downcast, rather than zoom mode alone; `plane.has_minimum_size()`.
    pub fn has_minimum_size(&self) -> bool {
        !self
            .transform_persistence()
            .cast::<Graphic3dTransformPersScaledAbove>()
            .is_null()
    }
}
