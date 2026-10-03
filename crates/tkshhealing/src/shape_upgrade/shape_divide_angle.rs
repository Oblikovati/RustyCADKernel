//! Angular subdivision policy from OCCT 3d097a0328e71b8.
use super::{
    FaceDivider, ShapeDivider, ShapeUpgradeFaceDivide, ShapeUpgradeShapeDivide,
    ShapeUpgradeSplitSurfaceAngle,
};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::{Failure, Handle};

/// Divide rotational faces by angular extent; `ShapeUpgradeShapeDivideAngle::new(1.)`.
pub struct ShapeUpgradeShapeDivideAngle {
    parent: ShapeUpgradeShapeDivide,
}
impl ShapeDivider for ShapeUpgradeShapeDivideAngle {
    fn shape_divide_base(&self) -> &ShapeUpgradeShapeDivide {
        &self.parent
    }
}
impl std::ops::Deref for ShapeUpgradeShapeDivideAngle {
    type Target = ShapeUpgradeShapeDivide;
    fn deref(&self) -> &Self::Target {
        &self.parent
    }
}
impl ShapeUpgradeShapeDivideAngle {
    /// Create an angular criterion without a source; `ShapeUpgradeShapeDivideAngle::new(1.)`.
    pub fn new(maximum_angle: f64) -> Self {
        let parent = ShapeUpgradeShapeDivide::empty();
        configure_angle_face(&parent.split_face_tool(), maximum_angle);
        Self { parent }
    }
    /// Create an angular criterion for a shape; `ShapeUpgradeShapeDivideAngle::with_shape(1., shape)`.
    pub fn with_shape(maximum_angle: f64, shape: Shape) -> Self {
        let divider = Self::new(maximum_angle);
        divider.init(&shape);
        divider
    }
    /// Replace the surface criterion and clear the wire tool; `divider.init_tool(2.)?`.
    pub fn init_tool(&self, maximum_angle: f64) -> Result<(), Failure> {
        let face = self.split_face_tool();
        if face.is_null() {
            return Err(Failure::with_type(
                "Standard_NullObject",
                "ShapeUpgrade_ShapeDivideAngle face tool is null; expected initialized face criterion",
                "",
            ));
        }
        configure_angle_face(&face, maximum_angle);
        self.set_split_face_tool(&face);
        Ok(())
    }
    /// Reset the angular criterion; `divider.set_max_angle(2.)?`.
    pub fn set_max_angle(&self, maximum_angle: f64) -> Result<(), Failure> {
        self.init_tool(maximum_angle)
    }
    /// Query the configured angle, returning zero for absent/incompatible tools; `divider.max_angle()`.
    pub fn max_angle(&self) -> f64 {
        let face = self.split_face_tool();
        if face.is_null() {
            return 0.;
        }
        let surface = face
            .cast::<dyn FaceDivider>()
            .split_surface_tool()
            .cast::<ShapeUpgradeSplitSurfaceAngle>();
        surface
            .get()
            .map_or(0., ShapeUpgradeSplitSurfaceAngle::max_angle)
    }
}
fn configure_angle_face(face: &Handle<ShapeUpgradeFaceDivide>, maximum_angle: f64) {
    face.set_split_surface_tool(
        &Handle::new(ShapeUpgradeSplitSurfaceAngle::new(maximum_angle)).cast(),
    );
    face.set_wire_divide_tool(&Handle::default());
}
