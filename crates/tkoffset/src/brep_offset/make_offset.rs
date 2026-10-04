//! Standard skin-offset maker configuration and persistent state.
use super::{BRepOffsetError, BRepOffsetMode};
use rustycad_tkbool::brep_algo::BRepAlgoImage;
use rustycad_tkbrep::topods::{Face, Shape};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::geom_abs::JoinType;

/// Persistent state of `BRepOffset_MakeOffset`; `BRepOffsetMakeOffset::new()`.
pub struct BRepOffsetMakeOffset {
    pub(super) config: Option<BRepOffsetMakeOffsetConfiguration>,
    allow_linearization: Option<bool>,
    pub(super) initial_shape: Shape,
    pub(super) face_component: Shape,
    pub(super) offset_faces: BRepAlgoImage,
    pub(super) offset_edges: BRepAlgoImage,
    pub(super) generated_vertices: Vec<(Shape, Shape)>,
    pub(super) result_shape: Shape,
    pub(super) closing_faces: Vec<Face>,
    pub(super) face_offsets: Vec<(Face, f64)>,
    pub(super) error: Option<BRepOffsetError>,
    pub(super) bad_shape: Shape,
    pub(super) done: bool,
}

/// A snapshot of `Initialize` arguments for callers that need to inspect stored parameters.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BRepOffsetMakeOffsetConfiguration {
    /// Global offset distance.
    pub offset: f64,
    /// Linear tolerance supplied to the offset algorithm.
    pub tolerance: f64,
    /// Skin, Pipe, or RectoVerso construction mode.
    pub mode: BRepOffsetMode,
    /// Whether tangential intersections are analyzed.
    pub intersection: bool,
    /// Whether self-intersection repair is enabled.
    pub self_intersection: bool,
    /// Arc or intersection join type.
    pub join_type: JoinType,
    /// Whether the input is thickened.
    pub thickening: bool,
    /// Whether internal edges are removed from the result.
    pub remove_internal_edges: bool,
}

impl Default for BRepOffsetMakeOffset {
    fn default() -> Self {
        Self::new()
    }
}

impl BRepOffsetMakeOffset {
    /// Creates an idle maker with a null result; `BRepOffsetMakeOffset::new()`.
    pub fn new() -> Self {
        Self {
            config: None,
            allow_linearization: None,
            initial_shape: Shape::default(),
            face_component: Shape::default(),
            offset_faces: BRepAlgoImage::new(),
            offset_edges: BRepAlgoImage::new(),
            generated_vertices: Vec::new(),
            result_shape: Shape::default(),
            closing_faces: Vec::new(),
            face_offsets: Vec::new(),
            error: None,
            bad_shape: Shape::default(),
            done: false,
        }
    }

    /// Stores standard offset parameters and clears prior run data; `maker.initialize(&shape, 2.0, 1.0e-7, BRepOffsetMode::Skin, false, false, JoinType::Arc, false, false)`.
    #[allow(clippy::too_many_arguments)]
    pub fn initialize(
        &mut self,
        shape: &Shape,
        offset: f64,
        tolerance: f64,
        mode: BRepOffsetMode,
        intersection: bool,
        self_intersection: bool,
        join_type: JoinType,
        thickening: bool,
        remove_internal_edges: bool,
    ) {
        let config = offset_configuration((
            offset,
            tolerance,
            mode,
            intersection,
            self_intersection,
            join_type,
            thickening,
            remove_internal_edges,
        ));
        self.initialize_with_config(shape, config);
    }

    fn initialize_with_config(&mut self, shape: &Shape, config: BRepOffsetMakeOffsetConfiguration) {
        self.initial_shape = shape.clone();
        self.config = Some(config);
        self.allow_linearization = Some(true);
        self.done = false;
        self.clear_run_data();
    }

    /// Clears results and histories while preserving configuration and last error; `maker.clear()`.
    pub fn clear(&mut self) {
        self.clear_run_data();
        self.done = false;
    }

    /// Sets the planar-face approximation option; `maker.allow_linearization(false)`.
    pub fn allow_linearization(&mut self, allowed: bool) {
        self.allow_linearization = Some(allowed);
    }

    /// Adds a distinct closing face from the original shape; `maker.add_face(&face)`.
    pub fn add_face(&mut self, face: &Face) {
        if self
            .closing_faces
            .iter()
            .any(|old| old.is_same(&face.clone().into_shape()))
        {
            return;
        }
        self.closing_faces.push(face.clone());
    }

    /// Assigns or replaces a face-specific offset; `maker.set_offset_on_face(&face, -1.5)`.
    pub fn set_offset_on_face(&mut self, face: &Face, offset: f64) {
        if let Some((_, old_offset)) = self
            .face_offsets
            .iter_mut()
            .find(|(old_face, _)| old_face.is_same(&face.clone().into_shape()))
        {
            *old_offset = offset;
            return;
        }
        self.face_offsets.push((face.clone(), offset));
    }

    /// Returns the original input shape; `maker.init_shape()`.
    pub fn init_shape(&self) -> &Shape {
        &self.initial_shape
    }

    /// Returns the standard-offset result, null before a successful build; `maker.shape()`.
    pub fn shape(&self) -> &Shape {
        &self.result_shape
    }

    /// Returns the last build completion state; `maker.is_done()`.
    pub const fn is_done(&self) -> bool {
        self.done
    }

    /// Returns the stored offset mode and tolerance-bearing configuration; `maker.configuration()?`.
    pub fn configuration(&self) -> Result<BRepOffsetMakeOffsetConfiguration, Failure> {
        self.config.ok_or_else(uninitialized_offset_maker)
    }

    /// Returns the last known native error, or `None` before an operation sets it.
    /// `maker.error()`.
    pub const fn error(&self) -> Option<BRepOffsetError> {
        self.error
    }

    /// Returns the shape blamed by input validation; `maker.bad_shape()`.
    pub fn bad_shape(&self) -> &Shape {
        &self.bad_shape
    }

    /// Returns the faces registered by `AddFace`; `maker.closing_faces().len()`.
    pub fn closing_faces(&self) -> &[Face] {
        &self.closing_faces
    }

    /// Returns the configured native join type after initialization; `maker.join_type()?`.
    pub fn join_type(&self) -> Result<JoinType, Failure> {
        self.configuration().map(|config| config.join_type)
    }

    /// Returns OCCT's planar linearization flag when it has been initialized or explicitly set.
    /// `maker.linearization_allowed()`.
    pub const fn linearization_allowed(&self) -> Option<bool> {
        self.allow_linearization
    }

    fn clear_run_data(&mut self) {
        self.result_shape = Shape::default();
        self.offset_faces.clear();
        self.offset_edges.clear();
        self.generated_vertices.clear();
        self.closing_faces.clear();
        self.face_offsets.clear();
    }
}

type OffsetConfigurationInputs = (f64, f64, BRepOffsetMode, bool, bool, JoinType, bool, bool);

fn offset_configuration(inputs: OffsetConfigurationInputs) -> BRepOffsetMakeOffsetConfiguration {
    let (
        offset,
        tolerance,
        mode,
        intersection,
        self_intersection,
        join_type,
        thickening,
        remove_internal_edges,
    ) = inputs;
    BRepOffsetMakeOffsetConfiguration {
        offset,
        tolerance,
        mode,
        intersection,
        self_intersection,
        join_type,
        thickening,
        remove_internal_edges,
    }
}

fn uninitialized_offset_maker() -> Failure {
    Failure::with_message(
        "Standard_ConstructionError",
        "BRepOffset_MakeOffset has no configuration; call Initialize before reading settings",
    )
}

#[cfg(test)]
mod tests {
    use super::BRepOffsetMakeOffset;
    use rustycad_tkmath::gp::Plane3;
    use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeFace;

    #[test]
    fn set_offset_on_face_replaces_the_same_face_and_clear_removes_override() {
        let face = test_face();
        let mut maker = BRepOffsetMakeOffset::new();
        maker.set_offset_on_face(&face, 1.5);
        maker.set_offset_on_face(&face, -2.0);
        assert_eq!(maker.face_offsets.len(), 1);
        assert_eq!(maker.face_offsets[0].1, -2.0);
        maker.clear();
        assert!(maker.face_offsets.is_empty());
    }

    fn test_face() -> rustycad_tkbrep::topods::Face {
        BRepBuilderAPIMakeFace::from_plane_bounds(Plane3::default(), (0.0, 1.0, 0.0, 1.0))
            .unwrap()
            .face()
            .unwrap()
    }
}
