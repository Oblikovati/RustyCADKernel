//! Whole-shape conversion to requested boundary and surface continuity.
use super::{
    ShapeDivider, ShapeUpgradeFaceDivide, ShapeUpgradeShapeDivide,
    ShapeUpgradeSplitCurve2dContinuity, ShapeUpgradeSplitCurve3dContinuity,
    ShapeUpgradeSplitSurfaceContinuity, ShapeUpgradeWireDivide,
};
use rustycad_tkernel::{precision, standard::Handle};
use rustycad_tkmath::geom_abs::Shape;
use std::ops::Deref;

#[derive(Clone, Copy)]
struct ContinuityPolicies {
    boundary: Shape,
    pcurve: Shape,
    surface: Shape,
    tolerance_3d: f64,
    tolerance_2d: f64,
}

impl ContinuityPolicies {
    fn native_defaults() -> Self {
        Self {
            boundary: Shape::C1,
            pcurve: Shape::C1,
            surface: Shape::C1,
            tolerance_3d: precision::confusion(),
            tolerance_2d: precision::p_confusion(),
        }
    }
}

impl Default for ContinuityPolicies {
    fn default() -> Self {
        Self::native_defaults()
    }
}

/// Configures whole-shape division using continuity criteria for curves and surfaces.
///
/// ```
/// use rustycad_tkshhealing::shape_upgrade::ShapeUpgradeShapeDivideContinuity;
/// use rustycad_tkmath::geom_abs::Shape;
/// let divider = ShapeUpgradeShapeDivideContinuity::empty();
/// divider.set_boundary_criterion(Shape::C2);
/// divider.set_pcurve_criterion(Shape::C1);
/// divider.set_surface_criterion(Shape::C1);
/// ```
pub struct ShapeUpgradeShapeDivideContinuity {
    parent: ShapeUpgradeShapeDivide,
    policies: rustycad_tkernel::standard::SharedState<ContinuityPolicies>,
}

impl Default for ShapeUpgradeShapeDivideContinuity {
    fn default() -> Self {
        Self {
            parent: ShapeUpgradeShapeDivide::empty(),
            policies: rustycad_tkernel::standard::SharedState::new(ContinuityPolicies::default()),
        }
    }
}

impl Deref for ShapeUpgradeShapeDivideContinuity {
    type Target = ShapeUpgradeShapeDivide;

    fn deref(&self) -> &Self::Target {
        &self.parent
    }
}

impl ShapeUpgradeShapeDivideContinuity {
    /// Create an uninitialized continuity divider with native defaults.
    /// `ShapeUpgradeShapeDivideContinuity::empty()`.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Initialize with a shape and native C1/Confusion defaults.
    /// `ShapeUpgradeShapeDivideContinuity::new(shape)`.
    pub fn new(shape: rustycad_tkbrep::topods::Shape) -> Self {
        let divider = Self::empty();
        divider.parent.init(&shape);
        divider
    }

    /// Set the spatial boundary continuity and preserve the raw criterion.
    /// `divider.set_boundary_criterion(Shape::C2)`.
    pub fn set_boundary_criterion(&self, criterion: Shape) {
        self.policies.write().boundary = criterion;
    }

    /// Set the pcurve continuity and preserve the raw criterion.
    /// `divider.set_pcurve_criterion(Shape::C2)`.
    pub fn set_pcurve_criterion(&self, criterion: Shape) {
        self.policies.write().pcurve = criterion;
    }

    /// Set the supporting-surface continuity and preserve the raw criterion.
    /// `divider.set_surface_criterion(Shape::C2)`.
    pub fn set_surface_criterion(&self, criterion: Shape) {
        self.policies.write().surface = criterion;
    }

    /// Set the 3D tolerance without clamping, as `SetTolerance` does.
    /// `divider.set_tolerance(1e-6)`.
    pub fn set_tolerance(&self, tolerance: f64) {
        self.policies.write().tolerance_3d = tolerance;
    }

    /// Set the 2D pcurve tolerance without clamping, as `SetTolerance2d` does.
    /// `divider.set_tolerance_2d(1e-8)`.
    pub fn set_tolerance_2d(&self, tolerance: f64) {
        self.policies.write().tolerance_2d = tolerance;
    }

    /// Perform whole-shape division with inherited context semantics.
    /// `divider.perform(true)?`.
    pub fn perform(&self, new_context: bool) -> Result<bool, rustycad_tkernel::standard::Failure> {
        ShapeDivider::perform(self, new_context)
    }

    /// Build the virtual face policy for this continuity divider.
    /// `let face_tool = divider.split_face_tool()`.
    pub fn split_face_tool(&self) -> Handle<ShapeUpgradeFaceDivide> {
        self.continuity_face_tool()
    }

    fn continuity_face_tool(&self) -> Handle<ShapeUpgradeFaceDivide> {
        let policy = *self.policies.read();
        let curve_2d = curve_2d_tool(policy);
        let curve_3d = curve_3d_tool(policy);
        let surface = surface_tool(policy);
        let wire = wire_tool(&curve_2d, &curve_3d);
        face_tool(&surface, &wire)
    }
}

impl ShapeDivider for ShapeUpgradeShapeDivideContinuity {
    fn shape_divide_base(&self) -> &ShapeUpgradeShapeDivide {
        &self.parent
    }

    fn split_face_tool(&self) -> Handle<ShapeUpgradeFaceDivide> {
        ShapeUpgradeShapeDivideContinuity::split_face_tool(self)
    }
}

fn curve_2d_tool(policy: ContinuityPolicies) -> Handle<ShapeUpgradeSplitCurve2dContinuity> {
    let tool = Handle::new(ShapeUpgradeSplitCurve2dContinuity::new());
    tool.set_criterion(policy.pcurve);
    tool.set_tolerance(policy.tolerance_2d);
    tool
}

fn curve_3d_tool(policy: ContinuityPolicies) -> Handle<ShapeUpgradeSplitCurve3dContinuity> {
    let tool = Handle::new(ShapeUpgradeSplitCurve3dContinuity::new());
    tool.set_criterion(policy.boundary);
    tool.set_tolerance(policy.tolerance_3d);
    tool
}

fn surface_tool(policy: ContinuityPolicies) -> Handle<ShapeUpgradeSplitSurfaceContinuity> {
    let tool = Handle::new(ShapeUpgradeSplitSurfaceContinuity::new());
    tool.set_criterion(policy.surface);
    tool.set_tolerance(policy.tolerance_3d);
    tool
}

fn wire_tool(
    curve_2d: &Handle<ShapeUpgradeSplitCurve2dContinuity>,
    curve_3d: &Handle<ShapeUpgradeSplitCurve3dContinuity>,
) -> Handle<ShapeUpgradeWireDivide> {
    let wire = Handle::new(ShapeUpgradeWireDivide::new());
    wire.set_split_curve_2d_tool(&curve_2d.cast());
    wire.set_split_curve_3d_tool(&curve_3d.cast());
    wire
}

fn face_tool(
    surface: &Handle<ShapeUpgradeSplitSurfaceContinuity>,
    wire: &Handle<ShapeUpgradeWireDivide>,
) -> Handle<ShapeUpgradeFaceDivide> {
    let face = Handle::new(ShapeUpgradeFaceDivide::empty());
    face.set_split_surface_tool(&surface.cast());
    face.set_wire_divide_tool(wire);
    face
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_and_setters_preserve_native_policy_values() {
        let divider = ShapeUpgradeShapeDivideContinuity::empty();
        assert_default_policies(&divider);
        divider.set_boundary_criterion(Shape::G1);
        divider.set_pcurve_criterion(Shape::G2);
        divider.set_surface_criterion(Shape::CN);
        divider.set_tolerance(-1.25);
        divider.set_tolerance_2d(f64::NAN);
        assert_updated_policies(&divider);
    }

    fn assert_default_policies(divider: &ShapeUpgradeShapeDivideContinuity) {
        let initial = *divider.policies.read();
        assert_eq!(
            (initial.boundary, initial.pcurve, initial.surface),
            (Shape::C1, Shape::C1, Shape::C1)
        );
        assert_eq!(initial.tolerance_3d, precision::confusion());
        assert_eq!(initial.tolerance_2d, precision::p_confusion());
    }

    fn assert_updated_policies(divider: &ShapeUpgradeShapeDivideContinuity) {
        let updated = *divider.policies.read();
        assert_eq!(
            (updated.boundary, updated.pcurve, updated.surface),
            (Shape::G1, Shape::G2, Shape::CN)
        );
        assert_eq!(updated.tolerance_3d, -1.25);
        assert!(updated.tolerance_2d.is_nan());
    }
}
