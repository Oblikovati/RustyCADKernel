//! `ShapeUpgrade_RemoveInternalWires` at OCCT 3d097a0328e71b8.
mod perform;

use super::ShapeUpgradeTool;
use crate::shape_build::ShapeBuildReShape;
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, ShapeMapHasher};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::{HSequence, IndexedDataMap, List};
use rustycad_tkernel::standard::{Failure, Handle, SharedState};
use rustycad_tkg3d::top_abs::ShapeEnum;

type RemovedEdgeFaces = IndexedDataMap<Shape, List<Shape>, ShapeMapHasher>;

struct RemoveInternalWiresFields {
    shape: Shape,
    result: Shape,
    min_area: f64,
    remove_face_mode: bool,
    status: i32,
    edge_faces: IndexedDataMapOfShapeListOfShape,
    remove_edges: RemovedEdgeFaces,
    removed_faces: Handle<HSequence<Shape>>,
    removed_wires: Handle<HSequence<Shape>>,
}

impl Default for RemoveInternalWiresFields {
    fn default() -> Self {
        Self {
            shape: Shape::default(),
            result: Shape::default(),
            min_area: 0.0,
            remove_face_mode: true,
            status: ShapeExtend::encode_status(ShapeExtendStatus::Ok),
            edge_faces: IndexedDataMapOfShapeListOfShape::default(),
            remove_edges: RemovedEdgeFaces::default(),
            removed_faces: Handle::new(HSequence::default()),
            removed_wires: Handle::new(HSequence::default()),
        }
    }
}

/// Removes internal face wires whose sampled contour area falls below the configured bound.
/// `let remover = ShapeUpgradeRemoveInternalWires::new();`
pub struct ShapeUpgradeRemoveInternalWires {
    parent: ShapeUpgradeTool,
    fields: SharedState<RemoveInternalWiresFields>,
}
impl_transient!(
    ShapeUpgradeRemoveInternalWires,
    "ShapeUpgrade_RemoveInternalWires",
    ShapeUpgradeTool,
    parent
);
impl std::ops::Deref for ShapeUpgradeRemoveInternalWires {
    type Target = ShapeUpgradeTool;
    fn deref(&self) -> &Self::Target {
        &self.parent
    }
}

impl Default for ShapeUpgradeRemoveInternalWires {
    fn default() -> Self {
        Self::new()
    }
}

impl ShapeUpgradeRemoveInternalWires {
    /// Creates the native context and default settings; `ShapeUpgradeRemoveInternalWires::new()`.
    pub fn new() -> Self {
        let parent = ShapeUpgradeTool::new();
        let context = Handle::new(ShapeBuildReShape::new());
        parent.set_context(&context);
        Self {
            parent,
            fields: SharedState::new(RemoveInternalWiresFields::default()),
        }
    }

    /// Creates a remover initialized with `shape`; `ShapeUpgradeRemoveInternalWires::with_shape(&shape)?`.
    pub fn with_shape(shape: &Shape) -> Result<Self, Failure> {
        let remover = Self::new();
        remover.init(shape)?;
        Ok(remover)
    }

    /// Initializes input topology, preserving the native reshape context; `remover.init(&shape)?`.
    pub fn init(&self, shape: &Shape) -> Result<(), Failure> {
        self.fields.write().shape = shape.clone();
        let context_handle = self.parent.context();
        context_handle
            .get()
            .ok_or_else(null_context)?
            .apply(shape, ShapeEnum::Shape)?;
        let mut fields = self.fields.write();
        rustycad_tkbrep::topexp::map_shapes_and_ancestors(
            shape,
            ShapeEnum::Edge,
            ShapeEnum::Face,
            &mut fields.edge_faces,
        )?;
        fields.status = ShapeExtend::encode_status(ShapeExtendStatus::Ok);
        fields.min_area = 0.0;
        fields.remove_face_mode = true;
        Ok(())
    }

    /// Sets the minimum contour area, without clamping; `remover.set_min_area(1.0e-5)`.
    pub fn set_min_area(&self, area: f64) {
        self.fields.write().min_area = area;
    }
    /// Returns the configured minimum contour area; `remover.min_area() == 0.0`.
    pub fn min_area(&self) -> f64 {
        self.fields.read().min_area
    }
    /// Sets whether faces bounded only by removed-wire edges are removed;
    /// `remover.set_remove_face_mode(false)`.
    pub fn set_remove_face_mode(&self, enabled: bool) {
        self.fields.write().remove_face_mode = enabled;
    }
    /// Returns the small-face removal mode; `remover.remove_face_mode()`.
    pub fn remove_face_mode(&self) -> bool {
        self.fields.read().remove_face_mode
    }
    /// Returns the result of the last completed perform call; `remover.get_result()`.
    pub fn get_result(&self) -> Shape {
        self.fields.read().result.clone()
    }
    /// Returns the shared sequence of removed faces in native discovery order; `remover.removed_faces()`.
    pub fn removed_faces(&self) -> Handle<HSequence<Shape>> {
        self.fields.read().removed_faces.clone()
    }
    /// Returns the shared sequence of removed wires in native discovery order; `remover.removed_wires()`.
    pub fn removed_wires(&self) -> Handle<HSequence<Shape>> {
        self.fields.read().removed_wires.clone()
    }
    /// Read native protected `myStatus` for specialized removal algorithms.
    /// Example: `let bits = remover.status_bits();`.
    pub fn status_bits(&self) -> i32 {
        self.fields.read().status
    }
    /// Write native protected `myStatus` without changing removal outputs.
    /// Example: `remover.set_status_bits(0x101);`.
    pub fn set_status_bits(&self, bits: i32) {
        self.fields.write().status = bits;
    }
    /// Decodes the last operation's ShapeExtend status; `remover.status(ShapeExtendStatus::Done1)`.
    pub fn status(&self, status: ShapeExtendStatus) -> bool {
        ShapeExtend::decode_status(self.fields.read().status, status)
    }
}

fn null_context() -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        "ShapeUpgrade_RemoveInternalWires: null context; expected ShapeBuild_ReShape",
    )
}
