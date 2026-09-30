// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepMAT2d/BRepMAT2d_Explorer.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepMAT2d_Explorer`: the contours of a face as sequences of trimmed 2d curves (the
//! p-curves of the edges of each wire, in wire order), with the gaps between
//! consecutive curves closed and the modified edges and wires recorded.
use super::explorer_wire::{fill_contour, wire_explorer};
use crate::brep_builder_api::BRepBuilderAPIMakeFace;
use crate::brep_lib;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Face, Shape};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{IndexedDataMap, Sequence};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

/// The curves of one contour (`NCollection_Sequence<occ::handle<Geom2d_Curve>>`).
pub type BRepMAT2dContour = Sequence<Handle<dyn Geom2dCurve>>;

/// The shapes replaced while the contours were built, with their replacements.
pub(super) type ModifiedShapes = IndexedDataMap<Shape, Shape, ShapeMapHasher>;

/// `BRepMAT2d_Explorer` (docs/topalgo-design.md section 14): the contours of a face for
/// `BRepMAT2d_BisectingLocus`. Each wire gives a contour of the p-curves of its edges
/// (trimmed and oriented as the edges); a gap above 1e-8 between two curves is closed
/// by moving an end of one of them (a new segment, or the end pole of its B-spline
/// form), and the moved edges, their wire and the face are recorded as modified.
///
/// ```
/// use rustycad_tkbrep::topods::Face;
/// use rustycad_tktopalgo::brep_mat2d::BRepMAT2dExplorer;
/// let explorer = BRepMAT2dExplorer::new();
/// assert_eq!(explorer.number_of_contours(), 0);
/// assert!(!explorer.is_modified(&Face::default()));
/// ```
#[derive(Clone, Default)]
pub struct BRepMAT2dExplorer {
    the_curves: Sequence<BRepMAT2dContour>,
    // Not set by the constructors natively (read before Init it is undefined); 0 here.
    current: i32,
    current_contour: i32,
    my_shape: Shape,
    my_is_closed: Sequence<bool>,
    my_modif_shapes: ModifiedShapes,
}

impl BRepMAT2dExplorer {
    /// `BRepMAT2d_Explorer()`: no contour; `BRepMAT2dExplorer::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `BRepMAT2d_Explorer(aFace)`: the explorer after `Perform(aFace)`;
    /// `BRepMAT2dExplorer::with_face(&face)?`.
    pub fn with_face(face: &Face) -> Result<Self, Failure> {
        let mut explorer = Self::new();
        explorer.perform(face)?;
        Ok(explorer)
    }

    /// `Clear()`: removes the contours, the closed flags and the modified shapes and
    /// resets the current contour; `explorer.clear()`.
    pub fn clear(&mut self) {
        self.the_curves.clear(None);
        self.current_contour = 0;
        self.my_is_closed.clear(None);
        self.my_modif_shapes.clear(false);
    }

    /// `Perform(aFace)`: clears, then adds one contour per wire of the FORWARD face (in
    /// explorer order) and builds a new face on the same surface (tolerance
    /// `Precision::Confusion()`) holding the wires, the modified ones rebuilt; its 3d
    /// curves are built and it is recorded as the modification of `face`. The errors of
    /// the p-curves, the face maker and BRepLib are returned;
    /// `explorer.perform(&face)?`.
    pub fn perform(&mut self, face: &Face) -> Result<(), Failure> {
        self.clear();
        self.my_shape = face.clone().into();
        let mut forward = face.clone();
        forward.set_orientation(Orientation::Forward);
        let surface = BRepTool::surface(&forward)?;
        let mut new_face =
            BRepBuilderAPIMakeFace::from_surface(&surface, precision::confusion())?.face()?;
        for wire in Explorer::new(&forward, ShapeEnum::Wire, ShapeEnum::Shape)? {
            self.add_wire(&topods::wire(&wire)?, &forward, &mut new_face)?;
        }
        brep_lib::build_curves3d_default(&new_face)?;
        self.my_modif_shapes
            .add(face.clone().into(), new_face.into());
        Ok(())
    }

    /// `Add(Spine, aFace, aNewFace)` (private): the contour of one wire, added to
    /// `new_face` (rebuilt from the new edges when a gap was closed).
    fn add_wire(
        &mut self,
        spine: &rustycad_tkbrep::topods::Wire,
        face: &Face,
        new_face: &mut Face,
    ) -> Result<(), Failure> {
        let Some(explorer) = wire_explorer(spine, face)? else {
            return Ok(());
        };
        self.new_contour();
        let closed: bool = spine.closed()?;
        let index = self.current_contour as usize;
        *self.my_is_closed.change_value(index)? = closed;
        let contour = self.the_curves.change_value(index)?;
        let result = fill_contour(explorer, spine, closed, (face, new_face), contour)?;
        for (key, value) in result.modified {
            self.my_modif_shapes.add(key, value);
        }
        BRepBuilder::default().add(new_face, &result.wire)
    }

    /// `NewContour()` (private): appends an empty contour, not closed, and makes it
    /// current. (`Add(aCurve)` is inlined: the wire contour appends to the sequence.)
    fn new_contour(&mut self) {
        self.the_curves.append(Sequence::default());
        self.my_is_closed.append(false);
        self.current_contour += 1;
    }

    /// `NumberOfContours()`; `explorer.number_of_contours()`.
    pub fn number_of_contours(&self) -> i32 {
        self.the_curves.length()
    }

    /// `NumberOfCurves(IndexContour)`: Standard_OutOfRange outside the contours;
    /// `explorer.number_of_curves(1)?`.
    pub fn number_of_curves(&self, index_contour: i32) -> Result<i32, Failure> {
        Ok(self.contour(index_contour)?.length())
    }

    /// `Init(IndexContour)`: starts the iteration on the curves of a contour;
    /// `explorer.init(1)`.
    pub fn init(&mut self, index_contour: i32) {
        self.current_contour = index_contour;
        self.current = 1;
    }

    /// `More()`: whether the current curve exists (Standard_OutOfRange when the
    /// current contour does not); `explorer.more()?`.
    pub fn more(&self) -> Result<bool, Failure> {
        Ok(self.current <= self.number_of_curves(self.current_contour)?)
    }

    /// `Next()`; `explorer.next()`.
    pub fn next(&mut self) {
        self.current += 1;
    }

    /// `Value()`: the current curve; Standard_OutOfRange past the end;
    /// `explorer.value()?`.
    pub fn value(&self) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        let contour = self.contour(self.current_contour)?;
        Ok(contour.value(sequence_index(self.current))?.clone())
    }

    /// `Shape()`: the face given to `Perform` (null before); `explorer.shape()`.
    pub fn shape(&self) -> Shape {
        self.my_shape.clone()
    }

    /// `Contour(IndexContour)`: the curves of a contour; Standard_OutOfRange outside;
    /// `explorer.contour(1)?`.
    pub fn contour(&self, index_contour: i32) -> Result<&BRepMAT2dContour, Failure> {
        self.the_curves.value(sequence_index(index_contour))
    }

    /// `IsModified(aShape)`: whether the shape (the face, a wire or an edge) was
    /// recorded with a replacement that is not the same shape;
    /// `explorer.is_modified(&edge)`.
    pub fn is_modified(&self, shape: &Shape) -> bool {
        self.my_modif_shapes
            .seek(shape)
            .is_some_and(|new_shape| !new_shape.is_same(shape))
    }

    /// `ModifiedShape(aShape)`: the recorded replacement, or the shape itself;
    /// `explorer.modified_shape(&face)`.
    pub fn modified_shape(&self, shape: &Shape) -> Shape {
        self.my_modif_shapes
            .seek(shape)
            .cloned()
            .unwrap_or_else(|| shape.clone())
    }

    /// `GetIsClosed()`: the closed flag of each contour (the `Closed` flag of its wire);
    /// `explorer.get_is_closed()`.
    pub fn get_is_closed(&self) -> &Sequence<bool> {
        &self.my_is_closed
    }
}

/// A native 1-based index as a sequence index; a negative one maps to 0, which the
/// sequence rejects with Standard_OutOfRange as natively.
fn sequence_index(index: i32) -> usize {
    usize::try_from(index).unwrap_or(0)
}
