// Rust translation of ShapeUpgrade/ShapeUpgrade_FaceDivide's unchanged-face path
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Face-level surface and boundary-curve division.
use super::ShapeUpgradeTool;
use crate::shape_build::ShapeBuildReShape;
use rustycad_tkbrep::topods::{Face, Shape};
use rustycad_tkernel::standard::{Failure, Handle, SharedState};

#[derive(Clone, Default)]
struct FaceDivideFields {
    face: Face,
    result: Shape,
}

/// `ShapeUpgrade_FaceDivide` for faces that need no surface or curve subdivision.
/// The upstream `Perform()` creates a missing re-shaping context before attempting its
/// split operations; this implementation currently preserves input faces unchanged.
///
/// ```
/// use rustycad_tkbrep::topods::Face;
/// use rustycad_tkshhealing::shape_upgrade::ShapeUpgradeFaceDivide;
/// let divider = ShapeUpgradeFaceDivide::new(Face::default());
/// assert!(!divider.perform(0.0)?);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct ShapeUpgradeFaceDivide {
    parent: ShapeUpgradeTool,
    fields: SharedState<FaceDivideFields>,
}

impl Default for ShapeUpgradeFaceDivide {
    fn default() -> Self {
        Self::new(Face::default())
    }
}

impl std::ops::Deref for ShapeUpgradeFaceDivide {
    type Target = ShapeUpgradeTool;

    fn deref(&self) -> &Self::Target {
        &self.parent
    }
}

impl ShapeUpgradeFaceDivide {
    /// Construct with the face and result initialized to `face`;
    /// `ShapeUpgradeFaceDivide::new(face)`.
    pub fn new(face: Face) -> Self {
        Self {
            parent: ShapeUpgradeTool::new(),
            fields: SharedState::new(FaceDivideFields {
                face: face.clone(),
                result: face.into(),
            }),
        }
    }

    /// Replace the working face and its current result; `divider.init(&face)`.
    pub fn init(&self, face: &Face) {
        let mut fields = self.fields.write();
        fields.face = face.clone();
        fields.result = face.clone().into();
    }

    /// Prepare the native default context and preserve an unchanged face;
    /// `assert!(!divider.perform(0.0)?)`.
    pub fn perform(&self, _area: f64) -> Result<bool, Failure> {
        let face = self.fields.read().face.clone();
        if face.is_null() {
            self.fields.write().result = Shape::default();
            return Ok(false);
        }
        self.initialize_context();
        self.fields.write().result = face.into();
        Ok(false)
    }

    /// Return the current result face as a shape; `let result = divider.result()`.
    pub fn result(&self) -> Shape {
        self.fields.read().result.clone()
    }

    fn initialize_context(&self) {
        if self.context().is_null() {
            self.set_context(&Handle::new(ShapeBuildReShape::new()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ShapeUpgradeFaceDivide;
    use rustycad_tkbrep::topods::Face;
    use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
    use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;

    fn valid_box_face() -> Face {
        let mut box_maker = BRepPrimAPIMakeBox::from_sizes(2.0, 3.0, 4.0).unwrap();
        let shape = box_maker.shape().unwrap();
        let faces = rustycad_tkbrep::topexp::Explorer::new(
            &shape,
            rustycad_tkg3d::top_abs::ShapeEnum::Face,
            rustycad_tkg3d::top_abs::ShapeEnum::Shape,
        )
        .unwrap();
        rustycad_tkbrep::topods::face(faces.current()).unwrap()
    }

    #[test]
    fn default_perform_preserves_null_face_and_result() {
        let divider = ShapeUpgradeFaceDivide::default();
        assert!(!divider.perform(0.0).unwrap());
        assert!(divider.result().is_null());
    }

    #[test]
    fn perform_creates_the_default_reshape_context() {
        let divider = ShapeUpgradeFaceDivide::new(valid_box_face());
        divider.perform(0.0).unwrap();
        assert!(!divider.context().is_null());
    }

    #[test]
    fn init_replaces_the_face_and_result() {
        let divider = ShapeUpgradeFaceDivide::default();
        divider.init(&Face::default());
        assert!(divider.result().is_null());
    }
}
