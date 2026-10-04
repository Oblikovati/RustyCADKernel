//! Explicit bounds overloads group native scalar arguments without changing initialization order.
use super::{BRepAdaptorCurve, BRepAdaptorSurface, BeanFaceIntersector, Failure, Range};

impl BeanFaceIntersector {
    /// Initializes adaptors, then assigns explicit bounds; `Self::from_adaptors_with_bounds(curve,surface,range,bounds,tolerances)?`.
    pub fn from_adaptors_with_bounds(
        curve: &BRepAdaptorCurve,
        surface: &BRepAdaptorSurface,
        range: Range,
        bounds: [f64; 4],
        tolerances: [f64; 2],
    ) -> Result<Self, Failure> {
        let mut solver = Self::new();
        solver.init_adaptors_with_bounds(curve, surface, range, bounds, tolerances)?;
        Ok(solver)
    }

    /// Reinitializes while preserving completion state; `solver.init_adaptors_with_bounds(curve,surface,range,bounds,tolerances)?`.
    pub fn init_adaptors_with_bounds(
        &mut self,
        curve: &BRepAdaptorCurve,
        surface: &BRepAdaptorSurface,
        range: Range,
        bounds: [f64; 4],
        tolerances: [f64; 2],
    ) -> Result<(), Failure> {
        self.init_adaptors(curve, surface, tolerances[0], tolerances[1])?;
        self.set_bean_parameters(range.first(), range.last());
        self.set_surface_parameters(bounds[0], bounds[1], bounds[2], bounds[3]);
        Ok(())
    }
}
