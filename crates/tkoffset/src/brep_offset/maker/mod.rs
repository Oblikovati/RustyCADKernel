// Rust translation of ModelingAlgorithms/TKOffset/BRepOffset/BRepOffset_MakeOffset.hxx and
// the public members of BRepOffset_MakeOffset.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepOffset_MakeOffset`: the offset shape of a shell or solid (offset faces joined by
//! arcs or by intersections, cut by their intersections, sewn into shells and solids),
//! and the thick solid between a shape and its offset.
pub(super) mod by_arc;
mod by_inter;
mod check;
mod conical;
mod finish;
mod history;
mod shells;
mod statics;
mod steps;
mod walls;

use super::{
    BRepOffsetAnalyse, BRepOffsetError, BRepOffsetInter3d, BRepOffsetMakeLoops, BRepOffsetMode,
};
use rustycad_tkbool::brep_algo::{BRepAlgoAsDes, BRepAlgoImage};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep_tools::Quilt;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Compound, Face, Shape, Solid};
use rustycad_tkbrep::toptools::{IndexedMapOfShape, MapOfShape, ShapeMapHasher};
use rustycad_tkernel::message::{ProgressRange, ProgressScope};
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{ShapeEnum, State};
use rustycad_tkmath::geom_abs::JoinType;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPISewing;
use rustycad_tktopalgo::brep_lib;

/// The operations of the progress indicator (`BRepOffset_PIOperation`).
#[derive(Clone, Copy)]
#[repr(usize)]
enum PiOperation {
    CheckInputData = 0,
    Analyse,
    BuildOffsetBy,
    Intersection,
    MakeMissingWalls,
    MakeShells,
    MakeSolid,
    Sewing,
}

/// The number of progress operations (`PIOperation_Last`).
const PI_OPERATION_LAST: usize = 8;

/// `normalizeSteps(theWhole, theSteps)`: the steps scaled to sum to the whole.
fn normalize_steps(the_whole: f64, the_steps: &mut [f64]) {
    let a_sum: f64 = the_steps.iter().sum();
    for s in the_steps.iter_mut() {
        *s = the_whole * *s / a_sum;
    }
}

/// `BRepOffset_MakeOffset`: the offset of a shape; `BRepOffsetMakeOffset::new()`.
pub struct BRepOffsetMakeOffset {
    pub(super) my_offset: f64,
    pub(super) my_tol: f64,
    pub(super) my_initial_shape: Shape,
    pub(super) my_shape: Shape,
    pub(super) my_face_comp: Shape,
    pub(super) my_mode: BRepOffsetMode,
    pub(super) my_is_linearization_allowed: bool,
    pub(super) my_inter: bool,
    pub(super) my_self_inter: bool,
    pub(super) my_join: JoinType,
    pub(super) my_thickening: bool,
    pub(super) my_remove_int_edges: bool,
    pub(super) my_face_offset: DataMap<Shape, f64, ShapeMapHasher>,
    pub(super) my_faces: IndexedMapOfShape,
    pub(super) my_original_faces: IndexedMapOfShape,
    pub(super) my_analyse: BRepOffsetAnalyse,
    pub(super) my_offset_shape: Shape,
    pub(super) my_init_offset_face: BRepAlgoImage,
    pub(super) my_init_offset_edge: BRepAlgoImage,
    pub(super) my_image_offset: BRepAlgoImage,
    pub(super) my_image_vv: BRepAlgoImage,
    pub(super) my_walls: List<Shape>,
    pub(super) my_as_des: Handle<BRepAlgoAsDes>,
    pub(super) my_edge_int_edges: DataMap<Shape, List<Shape>, ShapeMapHasher>,
    pub(super) my_done: bool,
    pub(super) my_error: BRepOffsetError,
    pub(super) my_make_loops: BRepOffsetMakeLoops,
    pub(super) my_is_perform_sewing: bool,
    pub(super) my_is_planar: bool,
    pub(super) my_bad_shape: Shape,
    pub(super) my_face_planface_map: DataMap<Shape, Shape, ShapeMapHasher>,
    pub(super) my_generated: List<Shape>,
    pub(super) my_res_map: MapOfShape,
}

impl Default for BRepOffsetMakeOffset {
    fn default() -> Self {
        Self::new()
    }
}

/// The parameters of `Initialize` past the shape, offset and tolerance, with their
/// native defaults (`Mode = BRepOffset_Skin`, no intersection, no self-intersection,
/// `Join = GeomAbs_Arc`, no thickening, internal edges kept).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MakeOffsetOptions {
    /// `Mode`.
    pub mode: BRepOffsetMode,
    /// `Intersection`: the complete intersection of the offset faces.
    pub intersection: bool,
    /// `SelfInter`.
    pub self_inter: bool,
    /// `Join`.
    pub join: JoinType,
    /// `Thickening`: the offset of a shell made a solid with its walls.
    pub thickening: bool,
    /// `RemoveIntEdges`.
    pub remove_int_edges: bool,
}

impl Default for MakeOffsetOptions {
    fn default() -> Self {
        Self {
            mode: BRepOffsetMode::Skin,
            intersection: false,
            self_inter: false,
            join: JoinType::Arc,
            thickening: false,
            remove_int_edges: false,
        }
    }
}

impl BRepOffsetMakeOffset {
    /// `BRepOffset_MakeOffset()`: an empty algorithm with a new AsDes (the native
    /// fields are left uninitialized; the defaults of `Initialize` here);
    /// `BRepOffsetMakeOffset::new()`.
    pub fn new() -> Self {
        Self {
            my_offset: 0.,
            my_tol: 0.,
            my_initial_shape: Shape::default(),
            my_shape: Shape::default(),
            my_face_comp: Shape::default(),
            my_mode: BRepOffsetMode::Skin,
            my_is_linearization_allowed: true,
            my_inter: false,
            my_self_inter: false,
            my_join: JoinType::Arc,
            my_thickening: false,
            my_remove_int_edges: false,
            my_face_offset: DataMap::default(),
            my_faces: IndexedMapOfShape::default(),
            my_original_faces: IndexedMapOfShape::default(),
            my_analyse: BRepOffsetAnalyse::new(),
            my_offset_shape: Shape::default(),
            my_init_offset_face: BRepAlgoImage::new(),
            my_init_offset_edge: BRepAlgoImage::new(),
            my_image_offset: BRepAlgoImage::new(),
            my_image_vv: BRepAlgoImage::new(),
            my_walls: List::default(),
            my_as_des: Handle::new(BRepAlgoAsDes::new()),
            my_edge_int_edges: DataMap::default(),
            my_done: false,
            my_error: BRepOffsetError::NoError,
            my_make_loops: BRepOffsetMakeLoops::new(),
            my_is_perform_sewing: false,
            my_is_planar: false,
            my_bad_shape: Shape::default(),
            my_face_planface_map: DataMap::default(),
            my_generated: List::default(),
            my_res_map: MapOfShape::default(),
        }
    }

    /// `BRepOffset_MakeOffset(S, Offset, Tol, Mode, Intersection, SelfInter, Join,
    /// Thickening, RemoveIntEdges, Range)`: the offset shape made at construction;
    /// `BRepOffsetMakeOffset::with_shape(&s, 1.0, 1e-7, MakeOffsetOptions::default(), &range)`.
    pub fn with_shape(
        s: &Shape,
        offset: f64,
        tol: f64,
        options: MakeOffsetOptions,
        the_range: &ProgressRange,
    ) -> Result<Self, Failure> {
        let mut m = Self::new();
        m.my_offset = offset;
        m.my_tol = tol;
        m.my_initial_shape = s.clone();
        m.my_shape = s.clone();
        m.set_options(options);
        m.my_done = false;
        m.my_is_linearization_allowed = true;
        m.make_offset_shape(the_range)?;
        Ok(m)
    }

    fn set_options(&mut self, o: MakeOffsetOptions) {
        self.my_mode = o.mode;
        self.my_inter = o.intersection;
        self.my_self_inter = o.self_inter;
        self.my_join = o.join;
        self.my_thickening = o.thickening;
        self.my_remove_int_edges = o.remove_int_edges;
    }

    /// `Initialize(S, Offset, Tol, Mode, Intersection, SelfInter, Join, Thickening,
    /// RemoveIntEdges)`: the parameters set and the algorithm cleared;
    /// `m.initialize(&s, 1.0, 1e-7, MakeOffsetOptions::default())`.
    pub fn initialize(&mut self, s: &Shape, offset: f64, tol: f64, options: MakeOffsetOptions) {
        self.my_offset = offset;
        self.my_initial_shape = s.clone();
        self.my_shape = s.clone();
        self.my_tol = tol;
        self.set_options(options);
        self.my_is_linearization_allowed = true;
        self.my_done = false;
        self.my_is_perform_sewing = false;
        self.my_is_planar = false;
        self.clear();
    }

    /// `Clear()`: the results and maps cleared (the error kept); `m.clear()`.
    pub fn clear(&mut self) {
        self.my_offset_shape = Shape::default();
        self.my_init_offset_face.clear();
        self.my_init_offset_edge.clear();
        self.my_image_offset.clear();
        self.my_image_vv.clear();
        self.my_faces.clear(true);
        self.my_original_faces.clear(true);
        self.my_face_offset.clear(true);
        self.my_edge_int_edges.clear(true);
        self.as_des().clear_shared();
        self.my_done = false;
        self.my_generated.clear();
        self.my_res_map.clear(true);
    }

    pub(super) fn as_des(&self) -> &BRepAlgoAsDes {
        self.my_as_des
            .get()
            .expect("BRepOffset_MakeOffset: null AsDes")
    }

    /// `AllowLinearization(theIsAllowed)`: whether nearly planar faces may be replaced
    /// by planes; `m.allow_linearization(false)`.
    pub fn allow_linearization(&mut self, the_is_allowed: bool) {
        self.my_is_linearization_allowed = the_is_allowed;
    }

    /// `AddFace(F)`: a face removed from the shape (a cap of the thick solid);
    /// `m.add_face(&face)`.
    pub fn add_face(&mut self, f: &Face) {
        self.my_original_faces.add(f.as_ref().clone());
    }

    /// `SetOffsetOnFace(F, Off)`: a specific offset for a face; `m.set_offset_on_face(&face, 2.0)`.
    pub fn set_offset_on_face(&mut self, f: &Face, off: f64) {
        self.my_face_offset.bind(f.as_ref().clone(), off);
    }

    /// `MakeOffsetShape(Range)`: the offset shape (`IsDone`, `Error`); the errors of the
    /// native algorithm are reported by `Error`, a raised exception as `Err`;
    /// `m.make_offset_shape(&range)?`.
    pub fn make_offset_shape(&mut self, the_range: &ProgressRange) -> Result<(), Failure> {
        self.my_done = false;
        // Whether the shape has only planar faces.
        self.my_is_planar = self.is_planar()?;
        self.set_faces()?;
        self.set_faces_with_offset();
        self.build_face_comp()?;
        // The shape without its caps.
        if !self.my_faces.is_empty() {
            statics::remove_corks(&mut self.my_shape, &mut self.my_original_faces)?;
            statics::remove_corks(&mut self.my_face_comp, &mut self.my_faces)?;
        }
        let a_ps = ProgressScope::with_literal(the_range, "Making offset shape", 100., false);
        let mut a_steps = [0.; PI_OPERATION_LAST];
        self.analyze_progress(100., &mut a_steps)?;
        if !self.check_input_data(&a_ps.next(a_steps[PiOperation::CheckInputData as usize]))?
            || self.my_error != BRepOffsetError::NoError
        {
            // An error in the input data, reported by Error().
            return Ok(());
        }
        self.my_error = BRepOffsetError::NoError;
        let side = if self.my_offset < 0. {
            State::Out
        } else {
            State::In
        };
        // The preanalysis.
        statics::eval_max(&self.my_shape, &mut self.my_tol)?;
        // A possible second variant: the analytical continuation of arcsin.
        let tol_angle_coeff =
            (self.my_tol / ((self.my_offset * 0.5).abs() + precision::confusion())).min(1.0);
        let tol_angle = 4. * tol_angle_coeff.asin();
        if self.my_join == JoinType::Intersection && self.my_inter && self.my_is_planar {
            self.my_analyse.set_offset_value(self.my_offset);
            self.my_analyse.set_face_offset_map(&self.my_face_offset);
        }
        let _ = a_ps.next(a_steps[PiOperation::Analyse as usize]);
        self.my_analyse.perform(&self.my_face_comp, tol_angle)?;
        for an_e in Explorer::new(&self.my_face_comp, ShapeEnum::Edge, ShapeEnum::Shape)? {
            let a_li = self
                .my_analyse
                .type_of(&rustycad_tkbrep::topods::edge(&an_e)?)?;
            let Some(last) = a_li.last() else {
                continue;
            };
            if last.get_type()? == rustycad_tkfillet::chfi_ds::TypeOfConcavity::Mixed {
                self.my_error = BRepOffsetError::MixedConnectivity;
                return Ok(());
            }
        }
        if !a_ps.more() {
            self.my_error = BRepOffsetError::UserBreak;
            return Ok(());
        }
        // The offset from the preanalysis: the update of the face offsets.
        self.update_face_offset()?;
        let build_range = a_ps.next(a_steps[PiOperation::BuildOffsetBy as usize]);
        if self.my_join == JoinType::Arc {
            self.build_offset_by_arc(&build_range)?;
        } else if self.my_join == JoinType::Intersection {
            self.build_offset_by_inter(&build_range)?;
        }
        if self.my_error != BRepOffsetError::NoError {
            return Ok(());
        }
        // The 3d intersection.
        let mut a_ps_inter = ProgressScope::unnamed(
            &a_ps.next(a_steps[PiOperation::Intersection as usize]),
            100.,
            false,
        );
        a_ps_inter.set_literal_name(if self.my_join == JoinType::Arc {
            "Connect offset faces by arc"
        } else {
            "Connect offset faces by intersection"
        });
        let mut inter = BRepOffsetInter3d::new(&self.my_as_des, side, self.my_tol);
        self.intersection3d(&mut inter, &a_ps_inter.next(90.))?;
        if self.my_error != BRepOffsetError::NoError {
            return Ok(());
        }
        // The 2d intersection.
        let mut modif = inter.touched_faces().clone();
        let new_edges = inter.new_edges().clone();
        if !modif.is_empty() {
            self.intersection2d(&modif, &new_edges, &a_ps_inter.next(4.))?;
            if self.my_error != BRepOffsetError::NoError {
                return Ok(());
            }
        }
        // The 2d unwinding and the reconstruction of the modified faces.
        self.make_loops(&mut modif, &a_ps_inter.next(4.))?;
        if self.my_error != BRepOffsetError::NoError {
            return Ok(());
        }
        // The reconstruction of the unmodified faces sharing rebuilt edges.
        if !modif.is_empty() {
            self.make_faces(&mut modif, &a_ps_inter.next(2.))?;
            if self.my_error != BRepOffsetError::NoError {
                return Ok(());
            }
        }
        a_ps_inter.close();
        if self.my_thickening {
            self.make_missing_walls(&a_ps.next(a_steps[PiOperation::MakeMissingWalls as usize]))?;
            if self.my_error != BRepOffsetError::NoError {
                return Ok(());
            }
        }
        // The shells.
        self.make_shells(&a_ps.next(a_steps[PiOperation::MakeShells as usize]))?;
        if self.my_error != BRepOffsetError::NoError {
            return Ok(());
        }
        if self.my_offset_shape.is_null() {
            // Not done.
            self.my_done = false;
            return Ok(());
        }
        // The 3d unwinding.
        self.select_shells()?;
        // The removal of the INTERNAL edges when required.
        if self.my_remove_int_edges {
            self.remove_internal_edges()?;
        }
        // The coding of the regularities.
        self.encode_regularity()?;
        // The replacement of the roots in the history maps.
        self.replace_roots()?;
        // The solids.
        self.make_solid(&a_ps.next(a_steps[PiOperation::MakeSolid as usize]))?;
        if self.my_error != BRepOffsetError::NoError {
            return Ok(());
        }
        // The update of the tolerances of the edges and vertices.
        if !self.my_offset_shape.is_null() {
            let init = if self.my_thickening {
                self.my_shape.clone()
            } else {
                Shape::default()
            };
            statics::update_tolerance(&mut self.my_offset_shape, &self.my_faces, &init)?;
            brep_lib::update_tolerances(&self.my_offset_shape, false)?;
        }
        self.correct_conical_faces()?;
        // The result solid is computed in the scope of MakeOffset.
        if self.my_thickening && self.my_is_perform_sewing {
            let mut a_sew = BRepBuilderAPISewing::new(self.my_tol, true, true, true, false);
            a_sew.add(&self.my_offset_shape)?;
            a_sew.perform(&a_ps.next(a_steps[PiOperation::Sewing as usize] / 2.))?;
            if !a_ps.more() {
                self.my_error = BRepOffsetError::UserBreak;
                return Ok(());
            }
            self.my_offset_shape = a_sew.sewed_shape().clone();
            // Rebuilt solid: the offset shape is expected to be really closed after sewing.
            self.my_offset_shape.set_closed(true)?;
            self.make_solid(&a_ps.next(a_steps[PiOperation::Sewing as usize] / 2.))?;
            if self.my_error != BRepOffsetError::NoError {
                return Ok(());
            }
        }
        self.my_done = true;
        Ok(())
    }

    /// `MakeThickSolid(Range)`: the offset shape, then the solid of the shape and its
    /// reversed offset limited by the caps (not done with fewer faces than both);
    /// `m.make_thick_solid(&range)?`.
    pub fn make_thick_solid(&mut self, the_range: &ProgressRange) -> Result<(), Failure> {
        // The shell parallel to the initial shell without caps.
        self.make_offset_shape(the_range)?;
        if !self.my_done {
            // The return code and the done state are kept.
            return Ok(());
        }
        // The solid of the initial shell and the parallel shell limited by the caps.
        if !self.my_faces.is_empty() {
            let builder = BRepBuilder::default();
            let mut res = Solid::default();
            builder.make_solid(&mut res);
            let mut nb_f = self.my_faces.extent();
            let mut glue = Quilt::new();
            for f in Explorer::new(&self.my_shape, ShapeEnum::Face, ShapeEnum::Shape)? {
                nb_f += 1;
                glue.add(&f)?;
            }
            let mut ya_result = false;
            if !self.my_offset_shape.is_null() {
                for f in Explorer::new(&self.my_offset_shape, ShapeEnum::Face, ShapeEnum::Shape)? {
                    ya_result = true;
                    glue.add(&f.reversed())?;
                }
            }
            if !ya_result {
                self.my_done = false;
                self.my_error = BRepOffsetError::UnknownError;
                return Ok(());
            }
            self.my_offset_shape = glue.shells()?;
            let mut res_shape = res.into_shape();
            for sh in Explorer::new(&self.my_offset_shape, ShapeEnum::Shell, ShapeEnum::Shape)? {
                builder.add(&mut res_shape, &sh)?;
            }
            res_shape.set_closed(true)?;
            self.my_offset_shape = res_shape;
            // The result is valid with more faces than the initial solid.
            let nb_of =
                Explorer::new(&self.my_offset_shape, ShapeEnum::Face, ShapeEnum::Shape)?.count();
            if nb_of < nb_f {
                self.my_done = false;
                self.my_error = BRepOffsetError::UnknownError;
                return Ok(());
            }
            if nb_of == nb_f {
                self.my_offset = 0.;
            }
        }
        if self.my_offset > 0. {
            self.my_offset_shape.reverse();
        }
        self.my_done = true;
        Ok(())
    }

    /// `IsDone()`; `m.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_done
    }

    /// `Error()`; `m.error()`.
    pub fn error(&self) -> BRepOffsetError {
        self.my_error
    }

    /// `Shape()`: the result; `m.shape()`.
    pub fn shape(&self) -> &Shape {
        &self.my_offset_shape
    }

    /// `InitShape()`: the initial shape; `m.init_shape()`.
    pub fn init_shape(&self) -> &Shape {
        &self.my_initial_shape
    }

    /// `GetAnalyse()`; `m.get_analyse()`.
    pub fn get_analyse(&self) -> &BRepOffsetAnalyse {
        &self.my_analyse
    }

    /// `GetJoinType()`; `m.get_join_type()`.
    pub fn get_join_type(&self) -> JoinType {
        self.my_join
    }

    /// `OffsetFacesFromShapes()`: the offset faces of the initial shapes; `m.offset_faces_from_shapes()`.
    pub fn offset_faces_from_shapes(&self) -> &BRepAlgoImage {
        &self.my_init_offset_face
    }

    /// `OffsetEdgesFromShapes()`: the offset edges of the initial shapes; `m.offset_edges_from_shapes()`.
    pub fn offset_edges_from_shapes(&self) -> &BRepAlgoImage {
        &self.my_init_offset_edge
    }

    /// `ClosingFaces()`: the caps; `m.closing_faces()`.
    pub fn closing_faces(&self) -> &IndexedMapOfShape {
        &self.my_original_faces
    }

    /// `GetBadShape()`: the shape that made the input data check fail; `m.get_bad_shape()`.
    pub fn get_bad_shape(&self) -> &Shape {
        &self.my_bad_shape
    }

    /// `analyzeProgress(theWhole, theSteps)`: the proportions of the operations.
    fn analyze_progress(
        &self,
        the_whole: f64,
        the_steps: &mut [f64; PI_OPERATION_LAST],
    ) -> Result<(), Failure> {
        the_steps.fill(0.);
        // The proportions differ with the join type; they are normalized afterwards.
        let is_arc = self.my_join == JoinType::Arc;
        let is_planar_int_case = self.my_inter
            && !is_arc
            && self.my_is_planar
            && !self.my_thickening
            && self.my_faces.is_empty()
            && statics::is_solid(&self.my_shape)?;
        the_steps[PiOperation::CheckInputData as usize] = 1.;
        the_steps[PiOperation::Analyse as usize] = 2.;
        the_steps[PiOperation::BuildOffsetBy as usize] = if is_planar_int_case {
            70.
        } else if is_arc {
            20.
        } else {
            50.
        };
        the_steps[PiOperation::Intersection as usize] = if is_planar_int_case {
            0.
        } else if is_arc {
            50.
        } else {
            20.
        };
        if self.my_thickening {
            the_steps[PiOperation::MakeMissingWalls as usize] = 5.;
        }
        the_steps[PiOperation::MakeShells as usize] = if is_planar_int_case { 25. } else { 5. };
        the_steps[PiOperation::MakeSolid as usize] = 5.;
        if self.my_is_perform_sewing && self.my_thickening {
            the_steps[PiOperation::Sewing as usize] = 10.;
        }
        normalize_steps(the_whole, the_steps);
        Ok(())
    }

    /// `SetFaces()`: the caps (their planar replacements when linearized) as their own
    /// offset images and roots.
    fn set_faces(&mut self) -> Result<(), Failure> {
        for ii in 1..=self.my_original_faces.extent() {
            let mut a_face = self.my_original_faces.find_key(ii)?.clone();
            if let Some(a_planface) = self.my_face_planface_map.seek(&a_face) {
                a_face = a_planface.clone();
            }
            self.my_faces.add(a_face.clone());
            self.my_init_offset_face.set_root(&a_face);
            self.my_init_offset_face.bind(&a_face, &a_face)?;
            self.my_image_offset.set_root(&a_face);
        }
        Ok(())
    }

    /// `SetFacesWithOffset()`: the specific offsets moved to the planar replacements.
    fn set_faces_with_offset(&mut self) {
        let pairs = by_arc::entries(&self.my_face_planface_map).unwrap_or_default();
        for (a_face, a_planface) in pairs {
            if let Some(an_offset) = self.my_face_offset.seek(&a_face).copied() {
                self.my_face_offset.unbind(&a_face);
                self.my_face_offset.bind(a_planface, an_offset);
            }
        }
    }

    /// `BuildFaceComp()`: the compound of the actual faces (the planar replacements
    /// instead of their originals).
    fn build_face_comp(&mut self) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        let mut c = Compound::default();
        builder.make_compound(&mut c);
        let mut comp = c.into_shape();
        for a_face in Explorer::new(&self.my_shape, ShapeEnum::Face, ShapeEnum::Shape)? {
            let an_or = a_face.orientation();
            let face = self
                .my_face_planface_map
                .seek(&a_face)
                .cloned()
                .unwrap_or(a_face);
            builder.add(&mut comp, &face.oriented(an_or))?;
        }
        self.my_face_comp = comp;
        Ok(())
    }
}
