// Rust translation of ModelingAlgorithms/TKBool/BRepFill/BRepFill_Sweep.hxx/.cxx (the
// constructor, the settings, the accessors and the small members) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepFill_Sweep`: the topological sweep of a section law along a location law.
mod build;

pub use build::SweepBuildOptions;
mod build_edge;
mod build_face;
mod build_wire;
mod corner;
mod filling;
mod helpers;
mod iso_edge;
mod rebuild_edge;
mod same_param;
mod shell;
mod shell_edges;
mod shell_ends;
mod shell_faces;
mod topology;

use super::location_law::BRepFillLocationLaw;
use super::section_law::BRepFillSectionLaw;
use super::shape_table::{ShapeTable, to_handle};
use build_face::EdgeEdgeMap;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topods::{self, Shape, ShapeIterator, Vertex, Wire};
use rustycad_tkbrep::toptools::{MapOfShape, ShapeMapHasher};
use rustycad_tkernel::ncollection::{DataMap, HArray2, List};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkgeomalgo::geom_fill::GeomFillApproxStyle;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tktopalgo::brep_lib;

/// The tapes or rails of a sweep (`NCollection_DataMap<TopoDS_Shape,
/// occ::handle<NCollection_HArray2<TopoDS_Shape>>>`), keyed by the edges or vertices of
/// the first section.
pub type SweepTablesMap = DataMap<Shape, Handle<HArray2<Shape>>, ShapeMapHasher>;

/// The topological sweep of a section law along a location law (`BRepFill_Sweep`);
/// `let mut sweep = BRepFillSweep::new(&section, &location, true);`.
pub struct BRepFillSweep {
    is_done: bool,
    k_part: bool,
    my_tol3d: f64,
    my_bound_tol: f64,
    my_tol2d: f64,
    my_tol_angular: f64,
    my_ang_min: f64,
    my_ang_max: f64,
    my_approx_style: GeomFillApproxStyle,
    my_continuity: Continuity,
    my_degmax: i32,
    my_segmax: i32,
    my_force_approx_c1: bool,
    my_shape: Shape,
    my_loc: Handle<dyn BRepFillLocationLaw>,
    my_sec: Handle<dyn BRepFillSectionLaw>,
    my_u_edges: Option<ShapeTable>,
    my_v_edges: Option<ShapeTable>,
    my_v_edges_modified: EdgeEdgeMap,
    my_faces: Option<ShapeTable>,
    my_aux_shape: List<Shape>,
    my_tapes: Option<Vec<Shape>>,
    error: f64,
    first_shape: Wire,
    last_shape: Wire,
}

/// Standard_NullObject for a law, a table or a curve that is not set (a crash natively).
pub(super) fn null(what: &str) -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        &format!("BRepFill_Sweep: null {what}; expected a {what}"),
        "BRepFill_Sweep",
    )
}

impl BRepFillSweep {
    /// `BRepFill_Sweep(Section, Location, WithKPart)`: the default tolerances (3d 1e-4,
    /// bounds 1, 2d 1e-5, angular 1e-2), angular control (0.01, 6), the approximation
    /// by location in C2 with degree 11 and 30 segments.
    pub fn new(
        section: &Handle<dyn BRepFillSectionLaw>,
        location: &Handle<dyn BRepFillLocationLaw>,
        with_k_part: bool,
    ) -> Self {
        let mut sweep = Self {
            is_done: false,
            k_part: with_k_part,
            my_tol3d: 0.,
            my_bound_tol: 0.,
            my_tol2d: 0.,
            my_tol_angular: 0.,
            my_ang_min: 0.,
            my_ang_max: 0.,
            my_approx_style: GeomFillApproxStyle::Location,
            my_continuity: Continuity::C2,
            my_degmax: 11,
            my_segmax: 30,
            my_force_approx_c1: false,
            my_shape: Shape::default(),
            my_loc: location.clone(),
            my_sec: section.clone(),
            my_u_edges: None,
            my_v_edges: None,
            my_v_edges_modified: EdgeEdgeMap::default(),
            my_faces: None,
            my_aux_shape: List::default(),
            my_tapes: None,
            error: 0.,
            first_shape: Wire::default(),
            last_shape: Wire::default(),
        };
        sweep.set_tolerance(1.0e-4, 1.0, 1.0e-5, 1.0e-2);
        sweep.set_angular_control(0.01, 6.0);
        sweep
    }

    /// The location law.
    pub(super) fn loc(&self) -> Result<&dyn BRepFillLocationLaw, Failure> {
        self.my_loc.get().ok_or_else(|| null("location law"))
    }

    /// The section law.
    pub(super) fn sec(&self) -> Result<&dyn BRepFillSectionLaw, Failure> {
        self.my_sec.get().ok_or_else(|| null("section law"))
    }

    /// `SetBounds(FirstShape, LastShape)`: the wires bounding the sweep; their edges
    /// that are not SameRange are marked neither SameRange nor SameParameter.
    pub fn set_bounds(&mut self, first: &Wire, last: &Wire) -> Result<(), Failure> {
        self.first_shape = first.clone();
        self.last_shape = last.clone();
        for wire in [&self.first_shape, &self.last_shape] {
            if !wire.is_null() {
                unmark_not_same_range(wire)?;
            }
        }
        Ok(())
    }

    /// `SetTolerance(Tol3d, BoundTol, Tol2d, TolAngular)` (natively defaulted to 1, 1e-5
    /// and 1e-2 after the first); `sweep.set_tolerance(1e-4, 1.0, 1e-5, 1e-2)`.
    pub fn set_tolerance(&mut self, tol3d: f64, bound_tol: f64, tol2d: f64, tol_angular: f64) {
        self.my_tol3d = tol3d;
        self.my_bound_tol = bound_tol;
        self.my_tol2d = tol2d;
        self.my_tol_angular = tol_angular;
    }

    /// `SetAngularControl(AngleMin, AngleMax)` (natively defaulted to 0.01 and 6): the
    /// angles below which there is no corner and above which a right corner is round;
    /// `sweep.set_angular_control(0.01, 6.0)`.
    #[allow(clippy::approx_constant)] // The native bound is the literal 6.28, not 2*PI.
    pub fn set_angular_control(&mut self, min_angle: f64, max_angle: f64) {
        self.my_ang_min = min_angle.max(precision::angular());
        self.my_ang_max = max_angle.min(6.28);
    }

    /// `SetForceApproxC1(ForceApproxC1)`: try a C1 approximation of a C0 swept
    /// surface; `sweep.set_force_approx_c1(true)`.
    pub fn set_force_approx_c1(&mut self, force_approx_c1: bool) {
        self.my_force_approx_c1 = force_approx_c1;
    }

    /// `IsDone()`; `sweep.is_done()`.
    pub fn is_done(&self) -> bool {
        self.is_done
    }

    /// `Shape()`: the wire or shell built; `sweep.shape()`.
    pub fn shape(&self) -> Shape {
        self.my_shape.clone()
    }

    /// `ErrorOnSurface()`: the largest approximation error; `sweep.error_on_surface()`.
    pub fn error_on_surface(&self) -> f64 {
        self.error
    }

    /// `SubShape()`: the faces (sections by paths); a null handle before `build`.
    pub fn sub_shape(&self) -> Handle<HArray2<Shape>> {
        self.my_faces.as_ref().map(to_handle).unwrap_or_default()
    }

    /// `InterFaces()`: the edges between the faces along the path (u-edges); a null
    /// handle before `build`.
    pub fn inter_faces(&self) -> Handle<HArray2<Shape>> {
        self.my_u_edges.as_ref().map(to_handle).unwrap_or_default()
    }

    /// `Sections()`: the edges, faces or compounds between two sweeps (v-edges); a
    /// null handle before `build`.
    pub fn sections(&self) -> Handle<HArray2<Shape>> {
        self.my_v_edges.as_ref().map(to_handle).unwrap_or_default()
    }

    /// `Tape(Index)`: the shell of the faces swept by the edge `index` of the section;
    /// Standard_NullObject before `build` of a shell, Standard_OutOfRange outside
    /// `1..=NbLaw`.
    pub fn tape(&self, index: i32) -> Result<Shape, Failure> {
        let tapes = self.my_tapes.as_ref().ok_or_else(|| null("tape array"))?;
        usize::try_from(index - 1)
            .ok()
            .and_then(|i| tapes.get(i))
            .cloned()
            .ok_or_else(|| {
                Failure::with_type(
                    "Standard_OutOfRange",
                    &format!(
                        "BRepFill_Sweep::Tape: index {index}; expected 1..={}",
                        tapes.len()
                    ),
                    "NCollection_Array1",
                )
            })
    }

    /// `CorrectApproxParameters()` (protected): the continuity lowered to the least
    /// continuity of the path edges and the segment count raised to their largest
    /// interval count; whether either changed.
    /// `sweep.correct_approx_parameters()?`.
    pub fn correct_approx_parameters(&mut self) -> Result<bool, Failure> {
        let the_path: Wire = self.loc()?.wire().clone();
        let (mut new_cont, mut new_segmax) = (self.my_continuity, self.my_segmax);
        let mut iter = ShapeIterator::new(&the_path, true, true);
        while iter.more() {
            let a_ba_curve = BRepAdaptorCurve::with_edge(&topods::edge(iter.value()?)?)?;
            let a_continuity: Continuity = a_ba_curve.continuity()?;
            let a_nb_interv: i32 = a_ba_curve.nb_intervals(Continuity::CN)?;
            if a_continuity < new_cont {
                new_cont = a_continuity;
            }
            if a_nb_interv > new_segmax {
                new_segmax = a_nb_interv;
            }
            iter.next();
        }
        let corrected: bool = new_cont != self.my_continuity || new_segmax != self.my_segmax;
        self.my_continuity = new_cont;
        self.my_segmax = new_segmax;
        Ok(corrected)
    }

    /// `MergeVertex(V1, V2)` (protected): `v2` made `v1` when their points are within
    /// the larger of their tolerances and the 3d tolerance.
    /// `sweep.merge_vertex(&v1, &mut v2)?`.
    pub fn merge_vertex(&self, v1: &Shape, v2: &mut Shape) -> Result<bool, Failure> {
        let (vx1, vx2): (Vertex, Vertex) = (topods::vertex(v1)?, topods::vertex(v2)?);
        let mut tol: f64 = BRepTool::vertex_tolerance(&vx1)?.max(BRepTool::vertex_tolerance(&vx2)?);
        if tol < self.my_tol3d {
            tol = self.my_tol3d;
        }
        if BRepTool::pnt(&vx1)?.distance(&BRepTool::pnt(&vx2)?) <= tol {
            *v2 = v1.clone();
            return Ok(true);
        }
        Ok(false)
    }

    /// `UpdateVertex(Ipath, Isec, Error, Param, V)` (protected): the tolerance of `v`
    /// raised to cover the vertex the laws give for it.
    /// `sweep.update_vertex(1, 2, 1e-5, 0.0, &vertex)?`.
    pub fn update_vertex(
        &self,
        ipath: i32,
        isec: i32,
        err_app: f64,
        param: f64,
        v: &Shape,
    ) -> Result<(), Failure> {
        let the_v: Vertex = topods::vertex(v)?;
        let sec = self.sec()?;
        let vv: Vertex = self.loc()?.perform_vertex(
            ipath,
            &sec.vertex(isec, param)?,
            err_app + sec.vertex_tol(isec - 1, param)?,
            0,
        )?;
        let (p1, p2) = (BRepTool::pnt(&vv)?, BRepTool::pnt(&the_v)?);
        let tol: f64 = BRepTool::vertex_tolerance(&vv)? + p1.distance(&p2);
        if tol > BRepTool::vertex_tolerance(&the_v)? {
            BRepBuilder::default().update_vertex_tolerance(&the_v, tol)?;
        }
        Ok(())
    }

    /// The table of faces; Standard_NullObject before it is made.
    pub(super) fn faces_mut(&mut self) -> Result<&mut ShapeTable, Failure> {
        self.my_faces.as_mut().ok_or_else(|| null("face table"))
    }

    /// The table of u-edges; Standard_NullObject before it is made.
    pub(super) fn u_edges_mut(&mut self) -> Result<&mut ShapeTable, Failure> {
        self.my_u_edges.as_mut().ok_or_else(|| null("u-edge table"))
    }

    /// The table of v-edges; Standard_NullObject before it is made.
    pub(super) fn v_edges_mut(&mut self) -> Result<&mut ShapeTable, Failure> {
        self.my_v_edges.as_mut().ok_or_else(|| null("v-edge table"))
    }
}

/// The edges of the wire that are not SameRange marked neither SameRange nor
/// SameParameter (PRO13551).
fn unmark_not_same_range(wire: &Wire) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    let mut wexp = WireExplorer::with_wire(wire)?;
    while wexp.more() {
        let e = wexp.current().clone();
        if !brep_lib::check_same_range(&e, 1.0e-12)? {
            builder.same_range(&e, false)?;
            builder.same_parameter(&e, false)?;
        }
        wexp.next()?;
    }
    Ok(())
}

/// The reversed edges given to `build` and the tapes and rails it reads and updates.
pub struct SweepHistory<'a> {
    /// The edges of the bounds reversed by the sweep.
    pub reversed_edges: &'a mut MapOfShape,
    /// The tapes of the edges of the first section.
    pub tapes: &'a mut SweepTablesMap,
    /// The rails of the vertices of the first section.
    pub rails: &'a mut SweepTablesMap,
}
