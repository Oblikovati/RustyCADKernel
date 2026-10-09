// Rust translation of BRepFill_Filling::Build and AddConstraints
// (ModelingAlgorithms/TKBool/BRepFill/BRepFill_Filling.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The plate of a filling: the constraints of the boundary edges, the free constraints
//! at the holes, the approximation and the final face.
use super::BRepFillFilling;
use super::constraints::{BRepFillCurveConstraint, BRepFillEdgeFaceAndOrder};
use super::wires::wire_from_list;
use crate::brep_fill::shape_table::table_error;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorCurve2d, BRepAdaptorSurface};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Shape, Vertex};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::ncollection::{Array1, Sequence};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dBezierCurve, Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomBSplineSurface, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeomalgo::geom_api::ProjectPointOnSurf;
use rustycad_tkgeomalgo::geom_plate::{
    GeomPlateBuildPlateSurface, GeomPlateCurveConstraint, GeomPlateMakeApprox,
    GeomPlatePlateG0Criterion,
};
use rustycad_tkgeombase::adv_app2var::{AdvApp2VarCriterionRepartition, AdvApp2VarCriterionType};
use rustycad_tkgeombase::extrema::ExtAlgo;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::{Point2, Xy, Xyz};
use rustycad_tkmath::toploc::Location;
use rustycad_tktopalgo::brep_lib::{self, BRepLibMakeEdge, BRepLibMakeFace, BRepLibMakeVertex};

/// Standard_Failure of AddConstraints for an edge without curve on surface.
fn no_surface() -> Failure {
    Failure::new("Add")
}

impl BRepFillFilling {
    /// `Build()`: the plate of the constraints (not done without boundary), its
    /// approximation (under the G0 criterion of the contour without initial surface)
    /// and the face of the new boundary edges on it.
    pub fn build(&mut self) -> Result<(), Failure> {
        self.my_builder = Some(GeomPlateBuildPlateSurface::new(
            self.my_degree,
            self.my_nb_pts_on_cur,
            self.my_nb_iter,
            self.my_tol2d,
            self.my_tol3d,
            self.my_tol_ang,
            self.my_tol_curv,
            self.my_anisotropie,
        )?);
        if self.my_boundary.is_empty() {
            self.my_is_done = false;
            return Ok(());
        }
        self.close_holes()?;
        self.load_constraints()?;
        self.builder_mut()?.perform(&ProgressRange::default())?;
        self.my_is_done = self.builder()?.is_done();
        if !self.my_is_done {
            return Ok(());
        }
        let (surface, dmax) = self.approximation()?;
        self.final_face(&surface, dmax)
    }

    /// The wires of the boundary edges, then each free constraint closing a hole
    /// between them.
    fn close_holes(&mut self) -> Result<(), Failure> {
        let edge_list: Vec<Shape> = self
            .my_boundary
            .iter()
            .map(|b| b.my_edge.as_ref().clone())
            .collect();
        let wire_list: Vec<Shape> = self.build_wires(edge_list)?;
        let mut ver_seq: Vec<Vertex> = self.find_extremities_of_holes(&wire_list)?;
        for j in 0..self.my_free_constraints.len() {
            self.close_hole(j, &mut ver_seq)?;
        }
        Ok(())
    }

    /// The initial surface, the boundary (its count of bounds), the edge constraints
    /// and the points loaded into the plate builder, in the native order.
    fn load_constraints(&mut self) -> Result<(), Failure> {
        if self.my_is_init_face_given {
            let h_surf_init = BRepAdaptorSurface::with_face(&self.my_init_face, true)?;
            let surface = BRepTool::surface(&h_surf_init.face())?;
            self.builder_mut()?.load_init_surface(&surface);
        }
        let boundary = self.my_boundary.clone();
        self.add_constraints(&boundary)?;
        let nb_bounds: i32 = self.my_boundary.len() as i32;
        self.builder_mut()?.set_nb_bounds(nb_bounds);
        let constraints = self.my_constraints.clone();
        self.add_constraints(&constraints)?;
        for point in self.my_points.clone() {
            self.builder_mut()?.add_point(&point);
        }
        Ok(())
    }

    /// The free constraint `j` closes the first hole whose two ends project on its face
    /// within Precision::Confusion(): a boundary edge on the face between them.
    fn close_hole(&mut self, j: usize, ver_seq: &mut Vec<Vertex>) -> Result<(), Failure> {
        let free = self.my_free_constraints[j].clone();
        let h_surf = BRepAdaptorSurface::with_face(&free.my_face, true)?;
        let cur_surface: Handle<dyn GeomSurface> = BRepTool::surface(&h_surf.face())?;
        let mut projector = ProjectPointOnSurf::new();
        for i in (0..ver_seq.len()).step_by(2) {
            let first_vtx: Vertex = ver_seq[i].clone();
            let last_vtx: Vertex = ver_seq
                .get(i + 1)
                .cloned()
                .ok_or_else(|| odd_extremities(ver_seq.len()))?;
            let Some((u1, v1)) = project_on(&mut projector, &first_vtx, &cur_surface)? else {
                continue;
            };
            let Some((u2, v2)) = project_on(&mut projector, &last_vtx, &cur_surface)? else {
                continue;
            };
            let e: Edge = bezier_edge(
                Point2::new(u1, v1),
                Point2::new(u2, v2),
                &cur_surface,
                &first_vtx,
                &last_vtx,
            )?;
            self.add_edge_on_face(&e, &free.my_face, free.my_order, true);
            ver_seq.drain(i..i + 2);
            break;
        }
        Ok(())
    }

    /// `AddConstraints(SeqOfConstraints)` (private): a curve constraint per edge (on its
    /// 3d curve in C0, on its first curve on surface otherwise, on its support face when
    /// it has one), with the p-curve on the initial face when it is given.
    fn add_constraints(
        &mut self,
        seq_of_constraints: &[BRepFillEdgeFaceAndOrder],
    ) -> Result<(), Failure> {
        for c in seq_of_constraints {
            let constr: Handle<GeomPlateCurveConstraint> = self.curve_constraint(c)?;
            if self.my_is_init_face_given {
                let found = BRepTool::curve_on_face(&c.my_edge, &self.my_init_face)?;
                if !found.pcurve.is_null() {
                    let trimmed: Handle<dyn Geom2dCurve> = Handle::new(Geom2dTrimmedCurve::new(
                        &found.pcurve,
                        found.first,
                        found.last,
                        true,
                        true,
                    )?)
                    .cast();
                    constr
                        .get()
                        .ok_or_else(no_surface)?
                        .set_curve2d_on_surf(&trimmed);
                }
            }
            self.builder_mut()?.add_curve(&constr);
        }
        Ok(())
    }

    /// The plate constraint of one edge.
    fn curve_constraint(
        &self,
        c: &BRepFillEdgeFaceAndOrder,
    ) -> Result<Handle<GeomPlateCurveConstraint>, Failure> {
        let tols: [f64; 3] = [self.my_tol3d, self.my_tol_ang, self.my_tol_curv];
        if !c.my_face.is_null() {
            let surf: Handle<dyn Adaptor3dSurface> =
                Handle::new(BRepAdaptorSurface::with_face(&c.my_face, true)?).cast();
            let curve2d: Handle<dyn Adaptor2dCurve2d> = Handle::new(
                BRepAdaptorCurve2d::with_edge_on_face(&c.my_edge, &c.my_face)?,
            )
            .cast();
            let h: Handle<Adaptor3dCurveOnSurface> =
                Handle::new(Adaptor3dCurveOnSurface::with_curve(&curve2d, &surf)?);
            return BRepFillCurveConstraint::on_surface(
                &h,
                c.my_order as i32,
                self.my_nb_pts_on_cur,
                tols,
            );
        }
        if c.my_order == Continuity::C0 {
            let h_curve: Handle<dyn Adaptor3dCurve> =
                Handle::new(BRepAdaptorCurve::with_edge(&c.my_edge)?).cast();
            return BRepFillCurveConstraint::with_curve(
                &h_curve,
                c.my_order as i32,
                self.my_nb_pts_on_cur,
                self.my_tol3d,
            );
        }
        let on_surface = BRepTool::curve_on_surface_by_index(&c.my_edge, 1)?;
        let surface = on_surface.surface.get().ok_or_else(no_surface)?;
        let copy: Handle<dyn GeomSurface> = surface.copy()?.cast::<dyn GeomSurface>();
        copy.get()
            .ok_or_else(no_surface)?
            .transform(on_surface.location.transformation())?;
        let surf: Handle<dyn Adaptor3dSurface> =
            Handle::new(GeomAdaptorSurface::with_surface(&copy)?).cast();
        let curve2d: Handle<dyn Adaptor2dCurve2d> =
            Handle::new(Geom2dAdaptorCurve::with_curve(&on_surface.pcurve)?).cast();
        let h: Handle<dyn Adaptor3dCurve> =
            Handle::new(Adaptor3dCurveOnSurface::with_curve(&curve2d, &surf)?).cast();
        Ok(Handle::new(GeomPlateCurveConstraint::new(
            &h,
            c.my_order as i32,
            self.my_nb_pts_on_cur,
            self.my_tol3d,
            self.my_tol_ang,
            self.my_tol_curv,
        )?))
    }

    /// The B-spline approximation of the plate (under the G0 criterion of the contour
    /// without initial surface, else of order 0) and the vertex tolerance
    /// `1.1 * G0Error`.
    fn approximation(&mut self) -> Result<(Handle<GeomBSplineSurface>, f64), Failure> {
        let g_plate = self.builder()?.surface();
        let dmax: f64 = 1.1 * self.builder()?.g0_error();
        if self.my_is_init_face_given {
            let approx = GeomPlateMakeApprox::with_order(
                &g_plate,
                self.my_tol3d,
                self.my_max_segments,
                self.my_max_deg,
                dmax,
                0,
                Continuity::C1,
                1.1,
            )?;
            return Ok((approx.surface(), dmax));
        }
        let (mut s2d, mut s3d): (Sequence<Xy>, Sequence<Xyz>) =
            (Sequence::default(), Sequence::default());
        self.builder_mut()?.disc2d_contour(4, &mut s2d)?;
        self.builder_mut()?.disc3d_contour(4, 0, &mut s3d)?;
        let seuil: f64 = self.my_tol3d.max(10. * self.builder()?.g0_error());
        let criterion = GeomPlatePlateG0Criterion::new(
            &s2d,
            &s3d,
            seuil,
            AdvApp2VarCriterionType::Absolute,
            AdvApp2VarCriterionRepartition::Regular,
        );
        let approx = GeomPlateMakeApprox::with_criterion(
            &g_plate,
            &criterion,
            self.my_tol3d,
            self.my_max_segments,
            self.my_max_deg,
            Continuity::C1,
            1.1,
        )?;
        Ok((approx.surface(), dmax))
    }

    /// The face of the approximated surface bounded by the new boundary edges (their
    /// p-curves on the plate, their vertices shared through the old/new map).
    fn final_face(
        &mut self,
        surface: &Handle<GeomBSplineSurface>,
        dmax: f64,
    ) -> Result<(), Failure> {
        let curves_on_plate = self.builder()?.curves2d()?;
        let surf: Handle<dyn GeomSurface> = surface.clone().cast();
        let mut final_edges: Vec<Shape> = Vec::new();
        for i in 1..=self.my_boundary.len() as i32 {
            let a_curve_on_plate: Handle<dyn Geom2dCurve> = curves_on_plate
                .get()
                .ok_or_else(no_surface)?
                .array1()
                .value(i)
                .map_err(table_error)?
                .clone();
            let init_edge: Edge = self.my_boundary[(i - 1) as usize].my_edge.clone();
            let new_edge: Edge = self.final_edge(&init_edge, &a_curve_on_plate, &surf, dmax)?;
            final_edges.push(new_edge.as_ref().clone());
            self.my_old_new_map.bind(
                init_edge.oriented(Orientation::Forward),
                new_edge.oriented(Orientation::Forward),
            );
        }
        let final_wire = wire_from_list(final_edges)?;
        self.my_face = BRepLibMakeFace::from_surface_wire(&surf, &final_wire, true)?.face()?;
        Ok(())
    }

    /// The new boundary edge of an old one: an empty copy of its rebuilt edge with the
    /// new vertices and the p-curve on the plate, made SameParameter.
    fn final_edge(
        &mut self,
        init_edge: &Edge,
        a_curve_on_plate: &Handle<dyn Geom2dCurve>,
        surf: &Handle<dyn GeomSurface>,
        dmax: f64,
    ) -> Result<Edge, Failure> {
        let builder = BRepBuilder::default();
        let mut an_edge = Edge::unchecked(init_edge.oriented(Orientation::Forward));
        if let Some(rebuilt) = self.my_old_new_map.seek(&an_edge) {
            an_edge = topods::edge(rebuilt)?;
        }
        let mut new_edge: Edge = topods::edge(&an_edge.empty_copied()?)?;
        let (v1, v2) = topexp::vertices(&an_edge, false);
        let mut new_v1: Vertex = self.new_vertex(&v1, dmax)?;
        let mut new_v2: Vertex = self.new_vertex(&v2, dmax)?;
        new_v1.set_orientation(Orientation::Forward);
        builder.add(&mut new_edge, &new_v1)?;
        new_v2.set_orientation(Orientation::Reversed);
        builder.add(&mut new_edge, &new_v2)?;
        builder.update_edge_pcurve(
            &new_edge,
            a_curve_on_plate,
            surf,
            &Location::default(),
            dmax,
            None,
        )?;
        brep_lib::same_parameter_shape(&new_edge, dmax, true)?;
        Ok(new_edge)
    }

    /// The new vertex of an old one: the recorded one, else a new vertex at its point
    /// with the tolerance `dmax`, recorded.
    fn new_vertex(&mut self, v: &Vertex, dmax: f64) -> Result<Vertex, Failure> {
        if let Some(new) = self.my_old_new_map.seek(v) {
            return topods::vertex(new);
        }
        let new_v: Vertex = BRepLibMakeVertex::new(BRepTool::pnt(v)?)?.vertex()?;
        BRepBuilder::default().update_vertex_tolerance(&new_v, dmax)?;
        self.my_old_new_map
            .bind(v.oriented(Orientation::Forward), new_v.as_ref().clone());
        Ok(new_v)
    }
}

/// Standard_OutOfRange of an odd count of hole extremities (never met: they come in
/// pairs).
fn odd_extremities(len: usize) -> Failure {
    Failure::with_type(
        "Standard_OutOfRange",
        &format!("BRepFill_Filling: {len} hole extremities; expected an even count"),
        "NCollection_Sequence",
    )
}

/// The parameters of the projection (gradient algorithm) of a vertex on a surface when
/// it is within Precision::Confusion() of it.
fn project_on(
    projector: &mut ProjectPointOnSurf,
    vertex: &Vertex,
    surface: &Handle<dyn GeomSurface>,
) -> Result<Option<(f64, f64)>, Failure> {
    projector.init(BRepTool::pnt(vertex)?, surface, ExtAlgo::Grad)?;
    if projector.lower_distance()? > rustycad_tkernel::precision::confusion() {
        return Ok(None);
    }
    Ok(Some(projector.lower_distance_parameters()?))
}

/// The edge on the surface of the 2d Bezier segment between two parameters, bounded by
/// the two vertices.
fn bezier_edge(
    p1: Point2,
    p2: Point2,
    surface: &Handle<dyn GeomSurface>,
    first: &Vertex,
    last: &Vertex,
) -> Result<Edge, Failure> {
    let mut points: Array1<'static, Point2> = Array1::with_bounds(1, 2).map_err(table_error)?;
    *points.change_value(1).map_err(table_error)? = p1;
    *points.change_value(2).map_err(table_error)? = p2;
    let line2d: Handle<dyn Geom2dCurve> = Handle::new(Geom2dBezierCurve::new(&points)?).cast();
    BRepLibMakeEdge::from_pcurve_vertices(&line2d, surface, first, last)?.edge()
}
