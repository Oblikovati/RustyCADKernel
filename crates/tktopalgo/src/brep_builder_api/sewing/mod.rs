// Rust translation of BRepBuilderAPI_Sewing.hxx, BRepBuilderAPI_Sewing.lxx and the
// constructor, Init, Load, Add, SewedShape, GetContext and SetContext of
// BRepBuilderAPI_Sewing.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepBuilderAPI_Sewing`: finds the contiguous free boundaries of faces within a
//! tolerance and assembles the faces into shells along them.
mod candidates;
mod closed_surface;
mod cutting;
mod cutting_nodes;
mod edge_processing;
mod evaluate;
mod face_analysis;
pub(crate) mod failures;
mod free_boundaries;
mod glue_vertices;
mod merge_nodes;
mod merged_nearest;
mod merging;
mod merging_sections;
mod pair_edge;
mod pair_pcurves;
mod perform;
mod queries;
mod replace_edge;
mod same_parameter;
mod sewed_shape;
#[cfg(test)]
mod tests;
mod vertex_tolerance;

use failures::null_object;
use rustycad_tkbrep::brep_tools::{BRepToolsReShape, ReShape};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::{
    IndexedDataMapOfShapeListOfShape, IndexedMapOfShape, MapOfShape, ShapeMapHasher,
};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::{DataMap, IndexedDataMap, List};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// `NCollection_IndexedDataMap<TopoDS_Shape, TopoDS_Shape, TopTools_ShapeMapHasher>`.
pub type SewingShapeIndexedMap = IndexedDataMap<Shape, Shape, ShapeMapHasher>;
/// `NCollection_DataMap<TopoDS_Shape, TopoDS_Shape, TopTools_ShapeMapHasher>`.
pub type SewingShapeMap = DataMap<Shape, Shape, ShapeMapHasher>;
/// `NCollection_DataMap<TopoDS_Shape, NCollection_List<TopoDS_Shape>, TopTools_ShapeMapHasher>`.
pub type SewingShapeListMap = DataMap<Shape, List<Shape>, ShapeMapHasher>;

/// `BRepBuilderAPI_Sewing` (docs/topalgo-design.md section 11): add the shapes to sew
/// (or load a context shape), then `perform`; the result is `sewed_shape`, and the
/// free, contiguous, multiple and degenerated edges, the deleted faces and the history
/// of the input shapes can be queried.
///
/// ```
/// use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPISewing;
/// let mut sewing = BRepBuilderAPISewing::default();
/// sewing.set_tolerance(0.01);
/// assert_eq!(sewing.tolerance(), 0.01);
/// assert!(sewing.sewed_shape().is_null());
/// ```
pub struct BRepBuilderAPISewing {
    parent: StandardTransient,
    pub(crate) my_tolerance: f64,
    pub(crate) my_sewing: bool,
    pub(crate) my_analysis: bool,
    pub(crate) my_cutting: bool,
    pub(crate) my_nonmanifold: bool,
    pub(crate) my_old_shapes: SewingShapeIndexedMap,
    pub(crate) my_sewed_shape: Shape,
    pub(crate) my_degenerated: IndexedMapOfShape,
    pub(crate) my_free_edges: IndexedMapOfShape,
    pub(crate) my_multiple_edges: IndexedMapOfShape,
    pub(crate) my_contigous_edges: IndexedDataMapOfShapeListOfShape,
    pub(crate) my_contig_sec_bound: SewingShapeMap,
    pub(crate) my_nb_shapes: i32,
    pub(crate) my_nb_vertices: i32,
    pub(crate) my_nb_edges: i32,
    pub(crate) my_bound_faces: IndexedDataMapOfShapeListOfShape,
    pub(crate) my_bound_sections: SewingShapeListMap,
    pub(crate) my_section_bound: SewingShapeMap,
    pub(crate) my_vertex_node: SewingShapeIndexedMap,
    pub(crate) my_vertex_node_free: SewingShapeIndexedMap,
    pub(crate) my_node_sections: SewingShapeListMap,
    pub(crate) my_cutting_node: SewingShapeListMap,
    pub(crate) my_little_face: IndexedMapOfShape,
    pub(crate) my_shape: Shape,
    pub(crate) my_re_shape: Handle<dyn ReShape>,
    my_face_mode: bool,
    my_floating_edges_mode: bool,
    my_same_parameter_mode: bool,
    my_local_tolerance_mode: bool,
    my_min_tolerance: f64,
    my_max_tolerance: f64,
    pub(crate) my_merged_edges: MapOfShape,
}
impl_transient!(
    BRepBuilderAPISewing,
    "BRepBuilderAPI_Sewing",
    StandardTransient,
    parent
);

impl Default for BRepBuilderAPISewing {
    /// `BRepBuilderAPI_Sewing()`: tolerance 1e-6, sewing, analysis and cutting on,
    /// manifold mode.
    fn default() -> Self {
        Self::new(1.0e-06, true, true, true, false)
    }
}

/// The live object of a re-shaper handle (a null context is a null dereference
/// natively).
pub(crate) fn live_reshape(reshape: &Handle<dyn ReShape>) -> Result<&dyn ReShape, Failure> {
    reshape
        .get()
        .ok_or_else(|| null_object("BRepBuilderAPI_Sewing: the context re-shaper is null"))
}

impl BRepBuilderAPISewing {
    /// `BRepBuilderAPI_Sewing(tolerance, option1, option2, option3, option4)`: a new
    /// context, then `Init`: `option1` sews (else only analyses), `option2` analyses
    /// the degenerated shapes, `option3` cuts the free edges, `option4` sews
    /// non-manifold; `BRepBuilderAPISewing::new(1e-6, true, true, true, false)`.
    pub fn new(tolerance: f64, option1: bool, option2: bool, option3: bool, option4: bool) -> Self {
        let mut sewing = Self {
            parent: StandardTransient::default(),
            my_tolerance: 0.0,
            my_sewing: false,
            my_analysis: false,
            my_cutting: false,
            my_nonmanifold: false,
            my_old_shapes: SewingShapeIndexedMap::default(),
            my_sewed_shape: Shape::default(),
            my_degenerated: IndexedMapOfShape::default(),
            my_free_edges: IndexedMapOfShape::default(),
            my_multiple_edges: IndexedMapOfShape::default(),
            my_contigous_edges: IndexedDataMapOfShapeListOfShape::default(),
            my_contig_sec_bound: SewingShapeMap::default(),
            my_nb_shapes: 0,
            my_nb_vertices: 0,
            my_nb_edges: 0,
            my_bound_faces: IndexedDataMapOfShapeListOfShape::default(),
            my_bound_sections: SewingShapeListMap::default(),
            my_section_bound: SewingShapeMap::default(),
            my_vertex_node: SewingShapeIndexedMap::default(),
            my_vertex_node_free: SewingShapeIndexedMap::default(),
            my_node_sections: SewingShapeListMap::default(),
            my_cutting_node: SewingShapeListMap::default(),
            my_little_face: IndexedMapOfShape::default(),
            my_shape: Shape::default(),
            my_re_shape: Handle::new(BRepToolsReShape::new()).cast(),
            my_face_mode: true,
            my_floating_edges_mode: false,
            my_same_parameter_mode: true,
            my_local_tolerance_mode: false,
            my_min_tolerance: 0.0,
            my_max_tolerance: 0.0,
            my_merged_edges: MapOfShape::default(),
        };
        sewing.init(tolerance, option1, option2, option3, option4);
        sewing
    }

    /// `Init(tolerance, option1, option2, option3, option4)`: the tolerance (at least
    /// Precision::Confusion()), the options, the minimal tolerance (1e-4 of it, at least
    /// Precision::Confusion()), an infinite maximal tolerance, the default modes, then
    /// `Load` of a null shape; `sewing.init(1e-6, true, true, true, false)`.
    pub fn init(
        &mut self,
        tolerance: f64,
        option1: bool,
        option2: bool,
        option3: bool,
        option4: bool,
    ) {
        self.my_tolerance = std_max(tolerance, precision::confusion());
        self.my_sewing = option1;
        self.my_analysis = option2;
        self.my_cutting = option3;
        self.my_nonmanifold = option4;
        // szv: proposal
        self.my_min_tolerance = self.my_tolerance * 1e-4;
        if self.my_min_tolerance < precision::confusion() {
            self.my_min_tolerance = precision::confusion();
        }
        self.my_max_tolerance = precision::infinite();
        self.my_face_mode = true;
        self.my_floating_edges_mode = false;
        self.my_same_parameter_mode = true;
        self.my_local_tolerance_mode = false;
        self.my_sewed_shape.nullify();
        self.clear_loaded_state();
        self.my_shape.nullify();
    }

    /// `Load(shape)`: clears the context and every result, and keeps the context
    /// shape (the shapes added later are sewn into it); `sewing.load(&shape)?`.
    pub fn load(&mut self, shape: &Shape) -> Result<(), Failure> {
        self.clear_loaded_state();
        if shape.is_null() {
            self.my_shape.nullify();
            return Ok(());
        }
        self.my_shape = live_reshape(&self.my_re_shape)?.apply(shape, ShapeEnum::Shape)?;
        Ok(())
    }

    /// The part of `Load` before the context shape: the context, the sewed shape, the
    /// counters and every map are cleared.
    fn clear_loaded_state(&mut self) {
        if let Some(reshape) = self.my_re_shape.get() {
            reshape.clear();
        }
        self.my_sewed_shape.nullify();
        self.my_nb_shapes = 0;
        self.my_nb_edges = 0;
        self.my_nb_vertices = 0;
        self.my_old_shapes.clear(true);
        self.my_degenerated.clear(true);
        self.my_free_edges.clear(true);
        self.my_multiple_edges.clear(true);
        self.my_contigous_edges.clear(true);
        self.my_contig_sec_bound.clear(true);
        self.my_bound_faces.clear(true);
        self.my_bound_sections.clear(true);
        self.my_vertex_node.clear(true);
        self.my_vertex_node_free.clear(true);
        self.my_node_sections.clear(true);
        self.my_cutting_node.clear(true);
        self.my_section_bound.clear(true);
        self.my_little_face.clear(true);
    }

    /// `Add(shape)`: a shape to sew (ignored when null), with its image by the context;
    /// `sewing.add(&face)?`.
    pub fn add(&mut self, shape: &Shape) -> Result<(), Failure> {
        if shape.is_null() {
            return Ok(());
        }
        let new_shape = live_reshape(&self.my_re_shape)?.apply(shape, ShapeEnum::Shape)?;
        self.my_old_shapes.add(shape.clone(), new_shape);
        self.my_nb_shapes = self.my_old_shapes.extent() as i32;
        Ok(())
    }

    /// `SewedShape()`: the sewn shape (a face, a shell, a solid or a compound), null
    /// when nothing was built; `sewing.sewed_shape().is_null()`.
    pub fn sewed_shape(&self) -> &Shape {
        &self.my_sewed_shape
    }

    /// `SetContext(theContext)`: the re-shaper that records the modifications;
    /// `sewing.set_context(&context)`.
    pub fn set_context(&mut self, context: &Handle<dyn ReShape>) {
        self.my_re_shape = context.clone();
    }

    /// `GetContext()`: the re-shaper that records the modifications;
    /// `sewing.get_context().is_null()`.
    pub fn get_context(&self) -> &Handle<dyn ReShape> {
        &self.my_re_shape
    }

    /// `SameParameterMode()`; `sewing.same_parameter_mode()`.
    pub fn same_parameter_mode(&self) -> bool {
        self.my_same_parameter_mode
    }
    /// `SetSameParameterMode(mode)`; `sewing.set_same_parameter_mode(false)`.
    pub fn set_same_parameter_mode(&mut self, mode: bool) {
        self.my_same_parameter_mode = mode;
    }
    /// `Tolerance()`; `sewing.tolerance()`.
    pub fn tolerance(&self) -> f64 {
        self.my_tolerance
    }
    /// `SetTolerance(theToler)`: the tolerance as given (no lower bound, unlike Init);
    /// `sewing.set_tolerance(0.01)`.
    pub fn set_tolerance(&mut self, toler: f64) {
        self.my_tolerance = toler;
    }
    /// `MinTolerance()`; `sewing.min_tolerance()`.
    pub fn min_tolerance(&self) -> f64 {
        self.my_min_tolerance
    }
    /// `SetMinTolerance(theMinToler)`; `sewing.set_min_tolerance(1e-7)`.
    pub fn set_min_tolerance(&mut self, min_toler: f64) {
        self.my_min_tolerance = min_toler;
    }
    /// `MaxTolerance()`; `sewing.max_tolerance()`.
    pub fn max_tolerance(&self) -> f64 {
        self.my_max_tolerance
    }
    /// `SetMaxTolerance(theMaxToler)`; `sewing.set_max_tolerance(1.0)`.
    pub fn set_max_tolerance(&mut self, max_toler: f64) {
        self.my_max_tolerance = max_toler;
    }
    /// `FaceMode()`; `sewing.face_mode()`.
    pub fn face_mode(&self) -> bool {
        self.my_face_mode
    }
    /// `SetFaceMode(theFaceMode)`; `sewing.set_face_mode(false)`.
    pub fn set_face_mode(&mut self, face_mode: bool) {
        self.my_face_mode = face_mode;
    }
    /// `FloatingEdgesMode()`; `sewing.floating_edges_mode()`.
    pub fn floating_edges_mode(&self) -> bool {
        self.my_floating_edges_mode
    }
    /// `SetFloatingEdgesMode(theFloatingEdgesMode)`; `sewing.set_floating_edges_mode(true)`.
    pub fn set_floating_edges_mode(&mut self, floating_edges_mode: bool) {
        self.my_floating_edges_mode = floating_edges_mode;
    }
    /// `LocalTolerancesMode()` (the mode is kept but no step reads it, natively);
    /// `sewing.local_tolerances_mode()`.
    pub fn local_tolerances_mode(&self) -> bool {
        self.my_local_tolerance_mode
    }
    /// `SetLocalTolerancesMode(theLocalTolerancesMode)`;
    /// `sewing.set_local_tolerances_mode(true)`.
    pub fn set_local_tolerances_mode(&mut self, local_tolerances_mode: bool) {
        self.my_local_tolerance_mode = local_tolerances_mode;
    }
    /// `SetNonManifoldMode(theNonManifoldMode)`; `sewing.set_non_manifold_mode(true)`.
    pub fn set_non_manifold_mode(&mut self, non_manifold_mode: bool) {
        self.my_nonmanifold = non_manifold_mode;
    }
    /// `NonManifoldMode()`; `sewing.non_manifold_mode()`.
    pub fn non_manifold_mode(&self) -> bool {
        self.my_nonmanifold
    }
}
