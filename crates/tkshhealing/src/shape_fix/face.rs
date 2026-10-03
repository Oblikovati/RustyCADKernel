// Rust translation of the context and periodic-cone path in ShapeFix_Face.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Context preservation and apex-edge repair for periodic loops on conical faces.
mod coincident_wires;
mod configuration;
pub(crate) mod make_faces_on_patch;
mod missing_seam;
mod natural_bound;
mod orientation;
mod periodic_cone;
mod small_area;

use super::{FixRoot, FixRootData, ShapeFixRoot};
use crate::shape_analysis::ShapeAnalysisSurface;
use crate::shape_build::ShapeBuildReShape;
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{self, Face, Shape, Wire};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, SharedState};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

/// The context and periodic conical-loop path of `ShapeFix_Face`.
///
/// ```
/// use rustycad_tkbrep::topods::Shape;
/// use rustycad_tkshhealing::shape_fix::ShapeFixFace;
/// let fixer = ShapeFixFace::new(Shape::default());
/// assert!(!fixer.perform().unwrap());
/// ```
pub struct ShapeFixFace {
    parent: ShapeFixRoot,
    face_state: SharedState<Shape>,
    result: SharedState<Shape>,
    status: SharedState<i32>,
    modes: SharedState<FaceFixModes>,
    analysis_surface: SharedState<Handle<ShapeAnalysisSurface>>,
    forward: SharedState<bool>,
    wire_tool: Handle<super::ShapeFixWire>,
}

impl_transient!(ShapeFixFace, "ShapeFix_Face", ShapeFixRoot, parent, interfaces: [dyn FixRoot]);

impl Default for ShapeFixFace {
    fn default() -> Self {
        Self {
            parent: ShapeFixRoot::new(),
            face_state: SharedState::new(Shape::default()),
            result: SharedState::new(Shape::default()),
            status: SharedState::new(0),
            modes: SharedState::new(FaceFixModes::default()),
            analysis_surface: SharedState::new(Handle::default()),
            forward: SharedState::new(true),
            wire_tool: Handle::new(super::ShapeFixWire::new()),
        }
    }
}

/// `MapWires`: each resulting outside wire and its contained internal wires.
pub type ShapeFixFaceWireMap =
    DataMap<Shape, List<Shape>, rustycad_tkbrep::toptools::ShapeMapHasher>;

#[derive(Clone, Copy)]
struct FaceFixModes {
    orientation: i32,
    add_natural_bound: i32,
    small_area_wire: i32,
}

impl Default for FaceFixModes {
    fn default() -> Self {
        Self {
            orientation: -1,
            add_natural_bound: -1,
            small_area_wire: -1,
        }
    }
}

impl ShapeFixFace {
    /// `ShapeFix_Face(face)`: initialize the root shape and working face;
    /// `ShapeFixFace::new(face.into_shape())`.
    pub fn new(face: Shape) -> Self {
        let fixer = Self::default();
        fixer.init(face);
        fixer
    }

    /// `Init(face)`: replace the root shape and working face with `face`.
    pub fn init(&self, face: Shape) {
        self.parent
            .fix_root_data()
            .with_mut(|fields| fields.shape = face.clone());
        *self.face_state.write() = face.clone();
        *self.status.write() = 0;
        *self.analysis_surface.write() = initialize_face_surface(&face);
        *self.forward.write() = face.orientation() != Orientation::Reversed;
    }

    /// Sets the wire-orientation mode; negative selects OCCT's default-on behavior.
    /// Example: `fixer.set_fix_orientation_mode(0)`.
    pub fn set_fix_orientation_mode(&self, mode: i32) {
        self.modes.write().orientation = mode;
    }

    /// Reads the wire-orientation mode; `assert_eq!(fixer.fix_orientation_mode(), -1)`.
    pub fn fix_orientation_mode(&self) -> i32 {
        self.modes.read().orientation
    }

    /// Sets natural-bound mode; negative selects OCCT's conditional default.
    /// Example: `fixer.set_fix_add_natural_bound_mode(1)`.
    pub fn set_fix_add_natural_bound_mode(&self, mode: i32) {
        self.modes.write().add_natural_bound = mode;
    }

    /// Reads natural-bound mode; `assert_eq!(fixer.fix_add_natural_bound_mode(), -1)`.
    pub fn fix_add_natural_bound_mode(&self) -> i32 {
        self.modes.read().add_natural_bound
    }

    /// Starts a face on `surface`; `fixer.init_surface(&surface, 1e-7, true)?`.
    pub fn init_surface(
        &self,
        surface: &Handle<dyn GeomSurface>,
        tolerance: f64,
        forward: bool,
    ) -> Result<(), Failure> {
        let analysis = Handle::new(ShapeAnalysisSurface::new(surface)?);
        initialize_surface_state(self, analysis, tolerance, forward)
    }

    /// Starts a face from an existing analysis surface handle; `fixer.init_analysis_surface(&analysis, 1e-7, false)?`.
    pub fn init_analysis_surface(
        &self,
        surface: &Handle<ShapeAnalysisSurface>,
        tolerance: f64,
        forward: bool,
    ) -> Result<(), Failure> {
        if surface.is_null() {
            return Err(Failure::with_type(
                "Standard_NullObject",
                "ShapeFix_Face::Init: null ShapeAnalysis_Surface; expected initialized analysis surface",
                "",
            ));
        }
        initialize_surface_state(self, surface.clone(), tolerance, forward)
    }

    /// Adds a wire to the face as if its orientation were forward; `fixer.add_wire(&wire)?`.
    pub fn add_wire(&self, wire: &Wire) -> Result<(), Failure> {
        if wire.is_null() {
            return Ok(());
        }
        let mut face = self.face()?;
        face.set_orientation(Orientation::Forward);
        rustycad_tkbrep::topods::Builder.add(&mut face, &wire.clone().into_shape())?;
        let mut stored = face.into_shape();
        if !*self.forward.read() {
            stored.set_orientation(Orientation::Reversed);
        }
        *self.face_state.write() = stored;
        Ok(())
    }

    /// Classifies and orients wires, returning outside-to-inner wires in `map`.
    /// Example: `let changed = fixer.fix_orientation_with_map(&mut map)?`.
    pub fn fix_orientation_with_map(&self, map: &mut ShapeFixFaceWireMap) -> Result<bool, Failure> {
        orientation::fix_orientation(self, map)
    }

    /// Classifies and orients wires without retaining the wire map.
    /// Example: `let changed = fixer.fix_orientation()?`.
    pub fn fix_orientation(&self) -> Result<bool, Failure> {
        orientation::fix_orientation_empty_map(self)
    }

    /// Applies OCCT's empty-face and doubly periodic natural-bound paths;
    /// `let changed = fixer.fix_add_natural_bound()?`.
    pub fn fix_add_natural_bound(&self) -> Result<bool, Failure> {
        natural_bound::fix_add_natural_bound(self)
    }

    /// Adds a missing periodic seam and removes ComposeShell's small by-products.
    /// Example: `let changed = fixer.fix_missing_seam()?`.
    pub fn fix_missing_seam(&self) -> Result<bool, Failure> {
        missing_seam::fix::fix(self)
    }

    /// `Perform()`: preserve context replacements and repair a single periodic cone loop.
    pub fn perform(&self) -> Result<bool, Failure> {
        *self.status.write() = 0;
        let (context, tolerance) = self
            .parent
            .fix_root_data()
            .with(|fields| (fields.context.clone(), fields.precision));
        let input = self.face_state.read().clone();
        let applied = apply_context(&input, &context)?;
        if !is_face(&applied)? {
            *self.result.write() = applied;
            return Ok(false);
        }
        let face = topods::face(&applied)?;
        if let Some(split) = super::face_loops::split_bowtie_face(&face, tolerance, &context)? {
            *self.status.write() = ShapeExtend::encode_status(ShapeExtendStatus::Done7);
            *self.result.write() = split.clone();
            replace_context(&context, &face, &split)?;
            return Ok(true);
        }
        let fixed = periodic_cone::repair_periodic_cone(&face, tolerance)?;
        let result = fixed.clone().unwrap_or(applied);
        *self.face_state.write() = result.clone();
        *self.result.write() = result.clone();
        replace_context(&context, &face, &result)?;
        Ok(false)
    }

    /// `Result()`: return the last performed shape, including non-face context results.
    pub fn result(&self) -> Shape {
        self.result.read().clone()
    }

    /// `Face()`: return the last result as a face when it has face topology.
    pub fn face(&self) -> Result<Face, Failure> {
        topods::face(&self.face_state.read())
    }

    /// `Status(status)`: decode a flag from the last `Perform()`.
    pub fn status(&self, status: ShapeExtendStatus) -> bool {
        ShapeExtend::decode_status(*self.status.read(), status)
    }
}

impl std::ops::Deref for ShapeFixFace {
    type Target = ShapeFixRoot;
    fn deref(&self) -> &Self::Target {
        &self.parent
    }
}

fn initialize_face_surface(face_shape: &Shape) -> Handle<ShapeAnalysisSurface> {
    let Ok(face) = topods::face(face_shape) else {
        return Handle::default();
    };
    let Ok((surface, _)) = BRepTool::surface_with_location(&face) else {
        return Handle::default();
    };
    if surface.is_null() {
        return Handle::default();
    }
    ShapeAnalysisSurface::new(&surface)
        .map(Handle::new)
        .unwrap_or_default()
}

fn initialize_surface_state(
    fixer: &ShapeFixFace,
    surface: Handle<ShapeAnalysisSurface>,
    tolerance: f64,
    forward: bool,
) -> Result<(), Failure> {
    let mut face = Face::default();
    BRepBuilder::default().make_face_on_surface(
        &mut face,
        &surface.surface(),
        precision::confusion(),
    )?;
    let base_shape = face.clone().into_shape();
    if !forward {
        face.set_orientation(Orientation::Reversed);
    }
    let face_shape = face.into_shape();
    fixer
        .fix_root_data()
        .with_mut(|fields| fields.shape = base_shape);
    *fixer.face_state.write() = face_shape;
    *fixer.analysis_surface.write() = surface;
    *fixer.forward.write() = forward;
    fixer.set_precision(tolerance);
    *fixer.status.write() = 0;
    Ok(())
}

/// Applies a non-null reshape context using the full-shape limit of ShapeFix_Face.
fn apply_context(input: &Shape, context: &Handle<ShapeBuildReShape>) -> Result<Shape, Failure> {
    let Some(context) = context.get() else {
        return Ok(input.clone());
    };
    rustycad_tkbrep::brep_tools::ReShape::apply(context, input, ShapeEnum::Shape)
}

/// Returns whether context application left a face, including the null case.
fn is_face(shape: &Shape) -> Result<bool, Failure> {
    if shape.is_null() {
        return Ok(false);
    }
    Ok(shape.shape_type()? == ShapeEnum::Face)
}

/// Records a repaired face in the supplied reshape context.
fn replace_context(
    context: &Handle<ShapeBuildReShape>,
    original: &Face,
    result: &Shape,
) -> Result<(), Failure> {
    let Some(context) = context.get() else {
        return Ok(());
    };
    rustycad_tkbrep::brep_tools::ReShape::replace(context, &original.clone().into_shape(), result)
}
