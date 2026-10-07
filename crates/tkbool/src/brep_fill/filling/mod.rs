// Rust translation of ModelingAlgorithms/TKBool/BRepFill/BRepFill_Filling.hxx/.cxx (the
// constructor, the parameters, the constraints and the accessors) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepFill_Filling`: the face of a plate surface through boundary edges (on support
//! faces or not), free constraint faces, edges and points.
mod build;
mod constraints;
mod wires;

pub use constraints::{BRepFillCurveConstraint, BRepFillEdgeFaceAndOrder, BRepFillFaceAndOrder};

use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topods::{Edge, Face, Shape};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeomalgo::geom_plate::{GeomPlateBuildPlateSurface, GeomPlatePointConstraint};
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::Point3;

/// The filling of a hole by a plate surface (`BRepFill_Filling`);
/// `let mut f = BRepFillFilling::new(3, 15, 2, false, 1e-5, 1e-4, 0.01, 0.1, 8, 9);`.
pub struct BRepFillFilling {
    my_builder: Option<GeomPlateBuildPlateSurface>,
    my_boundary: Vec<BRepFillEdgeFaceAndOrder>,
    my_constraints: Vec<BRepFillEdgeFaceAndOrder>,
    my_free_constraints: Vec<BRepFillFaceAndOrder>,
    my_points: Vec<Handle<GeomPlatePointConstraint>>,
    my_old_new_map: DataMap<Shape, Shape, ShapeMapHasher>,
    my_generated: List<Shape>,
    my_face: Face,
    my_init_face: Face,
    my_tol2d: f64,
    my_tol3d: f64,
    my_tol_ang: f64,
    my_tol_curv: f64,
    my_max_deg: i32,
    my_max_segments: i32,
    my_degree: i32,
    my_nb_pts_on_cur: i32,
    my_nb_iter: i32,
    my_anisotropie: bool,
    my_is_init_face_given: bool,
    my_is_done: bool,
}

impl Default for BRepFillFilling {
    /// The native default parameters (degree 3, 15 points, 2 iterations, no
    /// anisotropy, 1e-5, 1e-4, 0.01, 0.1, degree 8 and 9 segments).
    fn default() -> Self {
        Self::new(3, 15, 2, false, 1.0e-5, 1.0e-4, 0.01, 0.1, 8, 9)
    }
}

impl BRepFillFilling {
    /// `BRepFill_Filling(Degree, NbPtsOnCur, NbIter, Anisotropie, Tol2d, Tol3d, TolAng,
    /// TolCurv, MaxDeg, MaxSegments)`: the resolution, constraint and approximation
    /// parameters.
    #[allow(clippy::too_many_arguments)] // The native ten-argument constructor.
    pub fn new(
        degree: i32,
        nb_pts_on_cur: i32,
        nb_iter: i32,
        anisotropie: bool,
        tol2d: f64,
        tol3d: f64,
        tol_ang: f64,
        tol_curv: f64,
        max_deg: i32,
        max_segments: i32,
    ) -> Self {
        Self {
            my_builder: None,
            my_boundary: Vec::new(),
            my_constraints: Vec::new(),
            my_free_constraints: Vec::new(),
            my_points: Vec::new(),
            my_old_new_map: DataMap::default(),
            my_generated: List::default(),
            my_face: Face::default(),
            my_init_face: Face::default(),
            my_tol2d: tol2d,
            my_tol3d: tol3d,
            my_tol_ang: tol_ang,
            my_tol_curv: tol_curv,
            my_max_deg: max_deg,
            my_max_segments: max_segments,
            my_degree: degree,
            my_nb_pts_on_cur: nb_pts_on_cur,
            my_nb_iter: nb_iter,
            my_anisotropie: anisotropie,
            my_is_init_face_given: false,
            my_is_done: false,
        }
    }

    /// `SetConstrParam(Tol2d, Tol3d, TolAng, TolCurv)` (natively defaulted to 1e-5,
    /// 1e-4, 0.01 and 0.1); `f.set_constr_param(1e-5, 1e-4, 0.01, 0.1)`.
    pub fn set_constr_param(&mut self, tol2d: f64, tol3d: f64, tol_ang: f64, tol_curv: f64) {
        (
            self.my_tol2d,
            self.my_tol3d,
            self.my_tol_ang,
            self.my_tol_curv,
        ) = (tol2d, tol3d, tol_ang, tol_curv);
    }

    /// `SetResolParam(Degree, NbPtsOnCur, NbIter, Anisotropie)` (natively defaulted to
    /// 3, 15, 2 and false); `f.set_resol_param(3, 15, 2, false)`.
    pub fn set_resol_param(
        &mut self,
        degree: i32,
        nb_pts_on_cur: i32,
        nb_iter: i32,
        anisotropie: bool,
    ) {
        (
            self.my_degree,
            self.my_nb_pts_on_cur,
            self.my_nb_iter,
            self.my_anisotropie,
        ) = (degree, nb_pts_on_cur, nb_iter, anisotropie);
    }

    /// `SetApproxParam(MaxDeg, MaxSegments)` (natively defaulted to 8 and 9);
    /// `f.set_approx_param(8, 9)`.
    pub fn set_approx_param(&mut self, max_deg: i32, max_segments: i32) {
        (self.my_max_deg, self.my_max_segments) = (max_deg, max_segments);
    }

    /// `LoadInitSurface(aFace)`: the initial surface of the plate;
    /// `f.load_init_surface(&face)`.
    pub fn load_init_surface(&mut self, a_face: &Face) {
        self.my_init_face = a_face.clone();
        self.my_is_init_face_given = true;
    }

    /// The count of the constraints before the points.
    fn nb_before_points(&self) -> i32 {
        (self.my_boundary.len() + self.my_free_constraints.len() + self.my_constraints.len()) as i32
    }

    /// `Add(anEdge, Order, IsBound)` (natively bound by default): a boundary edge, or
    /// an edge constraint; its index among the constraints;
    /// `f.add_edge(&edge, Continuity::C0, true)`.
    pub fn add_edge(&mut self, an_edge: &Edge, order: Continuity, is_bound: bool) -> i32 {
        self.add_edge_on_face(an_edge, &Face::default(), order, is_bound)
    }

    /// `Add(anEdge, Support, Order, IsBound)` (natively bound by default): an edge on a
    /// support face; its index among the constraints;
    /// `f.add_edge_on_face(&edge, &face, Continuity::G1, true)`.
    pub fn add_edge_on_face(
        &mut self,
        an_edge: &Edge,
        support: &Face,
        order: Continuity,
        is_bound: bool,
    ) -> i32 {
        let edge_face_and_order = BRepFillEdgeFaceAndOrder::with_edge(an_edge, support, order);
        if is_bound {
            self.my_boundary.push(edge_face_and_order);
            return self.my_boundary.len() as i32;
        }
        self.my_constraints.push(edge_face_and_order);
        self.nb_before_points()
    }

    /// `Add(Support, Order)`: a free constraint (a face without edge); its index;
    /// `f.add_face(&face, Continuity::G1)`.
    pub fn add_face(&mut self, support: &Face, order: Continuity) -> i32 {
        self.my_free_constraints
            .push(BRepFillFaceAndOrder::with_face(support, order));
        (self.my_boundary.len() + self.my_free_constraints.len()) as i32
    }

    /// `Add(Point)`: a point constraint in G0 with the 3d tolerance; its index;
    /// `f.add_point(Point3::new(0.5, 0.5, 0.2))?`.
    pub fn add_point(&mut self, point: Point3) -> Result<i32, Failure> {
        let a_pc =
            GeomPlatePointConstraint::with_point(point, Continuity::C0 as i32, self.my_tol3d)?;
        self.my_points.push(Handle::new(a_pc));
        Ok(self.nb_before_points() + self.my_points.len() as i32)
    }

    /// `Add(U, V, Support, Order)`: a point constraint on a support face (the order is
    /// passed as its enumeration value, as natively); its index;
    /// `f.add_point_on_face(0.5, 0.5, &face, Continuity::G1)?`.
    pub fn add_point_on_face(
        &mut self,
        u: f64,
        v: f64,
        support: &Face,
        order: Continuity,
    ) -> Result<i32, Failure> {
        let h_surf = BRepAdaptorSurface::with_face(support, true)?;
        let surface = BRepTool::surface(&h_surf.face())?;
        let a_pc = GeomPlatePointConstraint::on_surface(
            u,
            v,
            &surface,
            order as i32,
            self.my_tol3d,
            self.my_tol_ang,
            self.my_tol_curv,
        )?;
        self.my_points.push(Handle::new(a_pc));
        Ok(self.nb_before_points() + self.my_points.len() as i32)
    }

    /// `IsDone()`; `f.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_is_done
    }

    /// `Face()`: the filling face; `f.face()`.
    pub fn face(&self) -> Face {
        self.my_face.clone()
    }

    /// `Generated(S)`: the new edge of an old boundary edge (or the new vertex of an old
    /// vertex); `let new_edges = f.generated(&edge);`.
    pub fn generated(&mut self, s: &Shape) -> &List<Shape> {
        self.my_generated.clear();
        if let Some(new) = self.my_old_new_map.seek(s) {
            self.my_generated.append(new.clone());
        }
        &self.my_generated
    }

    /// The plate builder; Standard_NullObject before `build` (a crash natively).
    fn builder(&self) -> Result<&GeomPlateBuildPlateSurface, Failure> {
        self.my_builder.as_ref().ok_or_else(|| {
            Failure::with_type(
                "Standard_NullObject",
                "BRepFill_Filling: the plate is not built; expected Build first",
                "BRepFill_Filling",
            )
        })
    }

    /// The plate builder, for writing.
    fn builder_mut(&mut self) -> Result<&mut GeomPlateBuildPlateSurface, Failure> {
        self.my_builder.as_mut().ok_or_else(|| {
            Failure::with_type(
                "Standard_NullObject",
                "BRepFill_Filling: the plate is not built; expected Build first",
                "BRepFill_Filling",
            )
        })
    }

    /// `G0Error()`: the greatest distance from the boundary to the surface;
    /// `f.g0_error()?`.
    pub fn g0_error(&self) -> Result<f64, Failure> {
        Ok(self.builder()?.g0_error())
    }

    /// `G1Error()`: the greatest angle to the constraint surfaces; `f.g1_error()?`.
    pub fn g1_error(&self) -> Result<f64, Failure> {
        Ok(self.builder()?.g1_error())
    }

    /// `G2Error()`: the greatest curvature gap; `f.g2_error()?`.
    pub fn g2_error(&self) -> Result<f64, Failure> {
        Ok(self.builder()?.g2_error())
    }

    /// `G0Error(Index)`: the greatest distance to the constraint `index`;
    /// `f.g0_error_of(1)?`.
    pub fn g0_error_of(&mut self, index: i32) -> Result<f64, Failure> {
        self.builder_mut()?.g0_error_of(index)
    }

    /// `G1Error(Index)`; `f.g1_error_of(1)?`.
    pub fn g1_error_of(&mut self, index: i32) -> Result<f64, Failure> {
        self.builder_mut()?.g1_error_of(index)
    }

    /// `G2Error(Index)`; `f.g2_error_of(1)?`.
    pub fn g2_error_of(&mut self, index: i32) -> Result<f64, Failure> {
        self.builder_mut()?.g2_error_of(index)
    }
}
