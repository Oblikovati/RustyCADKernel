// Rust translation of BRepFeat_RibSlot::ExtremeFaces (the wire of one edge) and
// PtOnEdgeVertex of ModelingAlgorithms/TKFeat/BRepFeat/BRepFeat_RibSlot.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The extremities of the wire of a rib or slot on the basis: for each end, the edge of
//! the wire, the face of the basis it stops on, the vertex where it stops, and whether
//! it lies on the face or on an edge or vertex of the basis.
use super::RibSlot;
use super::geometry::{choice_of_faces, edge_curve};
use crate::loc_ope::CSIntersector;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::ncollection::{List, Sequence};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCurve, GeomPlane, GeomTrimmedCurve};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeomalgo::geom_api::ProjectPointOnCurve;
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_lib::BRepLibMakeVertex;

/// The in-out arguments of `ExtremeFaces`: the first and last edges, faces and vertices
/// of the wire on the basis, whether its ends lie on the faces, and the edges of the
/// basis they lie on.
#[derive(Clone, Default)]
pub struct Extremes {
    /// The edge of the wire at its first end.
    pub first_edge: Edge,
    /// The edge of the wire at its last end.
    pub last_edge: Edge,
    /// The face of the basis the first end stops on.
    pub first_face: Face,
    /// The face of the basis the last end stops on.
    pub last_face: Face,
    /// The vertex where the first end stops.
    pub first_vertex: Vertex,
    /// The vertex where the last end stops.
    pub last_vertex: Vertex,
    /// Whether the first end of the wire lies on its face.
    pub on_first_face: bool,
    /// Whether the last end of the wire lies on its face.
    pub on_last_face: bool,
    /// Whether the first end lies on an edge of the basis.
    pub pt_on_first_edge: bool,
    /// Whether the last end lies on an edge of the basis.
    pub pt_on_last_edge: bool,
    /// The edge of the basis the first end lies on.
    pub on_first_edge: Edge,
    /// The edge of the basis the last end lies on.
    pub on_last_edge: Edge,
}

/// Where a point lies on the basis (`PtOnEdgeVertex` outputs).
#[derive(Clone, Default)]
pub(super) struct PointOn {
    pub(super) pt_on_edge: bool,
    pub(super) on_edge: Edge,
    pub(super) pt_on_vertex: bool,
    pub(super) on_vertex: Vertex,
}

/// The faces of the basis holding the shape `on` (an edge or a vertex), each face once;
/// all the occurrences of `on` count when `every` (natively for the first edge), else
/// the first only.
pub(super) fn faces_holding(
    sbase: &Shape,
    on: &Shape,
    kind: ShapeEnum,
    every: bool,
) -> Result<List<Shape>, Failure> {
    let mut faces = List::default();
    let mut map = MapOfShape::default();
    for fx in Explorer::new(sbase, ShapeEnum::Face, ShapeEnum::Shape)? {
        if !map.add(fx.clone()) {
            continue;
        }
        for sub in Explorer::new(&fx, kind, ShapeEnum::Shape)? {
            if sub.is_same(on) {
                faces.append(fx.clone());
                if !every {
                    break;
                }
            }
        }
    }
    Ok(faces)
}

impl RibSlot {
    /// `PtOnEdgeVertex(RevolRib, shape, point, ...)`: the first edge of `shape` the point
    /// projects on within the edge tolerance (one projection for a rib, at least one for
    /// a revolution rib, the degenerated edges skipped for a rib), and its vertex at the
    /// point (both ends compared with the tolerance of the first vertex, as natively).
    pub(super) fn pt_on_edge_vertex(
        &self,
        revol_rib: bool,
        shape: &Shape,
        point: Point3,
        out: &mut PointOn,
    ) -> Result<(), Failure> {
        let mut map = MapOfShape::default();
        for e in Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)? {
            if !map.add(e.clone()) {
                continue;
            }
            let e = topods::edge(&e)?;
            if !revol_rib && BRepTool::degenerated(&e)? {
                continue;
            }
            let (mut ccc, fff, lll) = BRepTool::curve(&e)?;
            if !revol_rib {
                ccc = Handle::new(GeomTrimmedCurve::new(&ccc, fff, lll, true, true)?).cast();
            }
            let proj = ProjectPointOnCurve::with_curve(point, &ccc)?;
            let test_ok = if revol_rib {
                proj.nb_points() >= 1
            } else {
                proj.nb_points() == 1
            };
            if test_ok && proj.distance(1)? <= BRepTool::edge_tolerance(&e)? {
                out.pt_on_edge = true;
                out.on_edge = e.clone();
                let ev1 = topexp::first_vertex(&e, true);
                let ev2 = topexp::last_vertex(&e, true);
                let (ep1, ep2) = (BRepTool::pnt(&ev1)?, BRepTool::pnt(&ev2)?);
                let tol1 = BRepTool::vertex_tolerance(&ev1)?;
                if point.distance(&ep1) <= tol1 {
                    out.pt_on_vertex = true;
                    out.on_vertex = ev1;
                } else if point.distance(&ep2) <= tol1 {
                    out.pt_on_vertex = true;
                    out.on_vertex = ev2;
                }
                break;
            }
        }
        Ok(())
    }

    /// The face of the basis at one end of the single edge of the wire lying on an edge
    /// or a vertex of the basis: the one met first around the point at `par`.
    fn end_face(
        &self,
        on: &PointOn,
        every: bool,
        cc: &Handle<dyn GeomCurve>,
        par: f64,
        bnd: f64,
        pln: &Handle<GeomPlane>,
    ) -> Result<Option<Face>, Failure> {
        let mut faces = if on.pt_on_vertex {
            faces_holding(&self.sbase, &on.on_vertex, ShapeEnum::Vertex, false)?
        } else {
            faces_holding(&self.sbase, &on.on_edge, ShapeEnum::Edge, every)?
        };
        if faces.is_empty() {
            return Ok(None);
        }
        let fff = choice_of_faces(&mut faces, cc, par, bnd / 50., pln)?;
        Ok(if fff.is_null() { None } else { Some(fff) })
    }

    /// `ExtremeFaces(RevolRib, bnd, Pln, ...)`: the extremities of the wire on the basis
    /// (false when they are not found); `rib.extreme_faces(false, bnd, &pln, &mut ext)?`.
    pub(crate) fn extreme_faces(
        &self,
        revol_rib: bool,
        bnd: f64,
        pln: &Handle<GeomPlane>,
        x: &mut Extremes,
    ) -> Result<bool, Failure> {
        x.first_face.nullify();
        x.last_face.nullify();
        x.first_edge.nullify();
        x.last_edge.nullify();
        x.pt_on_first_edge = false;
        x.pt_on_last_edge = false;
        x.on_first_edge.nullify();
        x.on_last_edge.nullify();
        let edges: Vec<Shape> =
            Explorer::new(&self.wire, ShapeEnum::Edge, ShapeEnum::Shape)?.collect();
        if edges.len() == 1 {
            return self.extreme_faces_one_edge(revol_rib, bnd, pln, &topods::edge(&edges[0])?, x);
        }
        self.extreme_faces_wire(revol_rib, bnd, pln, x)
    }

    /// The single edge `e` of the wire: its ends on edges or vertices of the basis give
    /// their faces; else the faces where the edge enters and leaves the basis.
    fn extreme_faces_one_edge(
        &self,
        revol_rib: bool,
        bnd: f64,
        pln: &Handle<GeomPlane>,
        e: &Edge,
        x: &mut Extremes,
    ) -> Result<bool, Failure> {
        let (cc, f, l) = edge_curve(e)?;
        let p1 = BRepTool::pnt(&topexp::first_vertex(e, true))?;
        let p2 = BRepTool::pnt(&topexp::last_vertex(e, true))?;
        // ---Find if 2 points limiting the unique edge of the wire are on an edge or a
        //    vertex of the base shape
        let mut on1 = PointOn::default();
        let mut on2 = PointOn::default();
        self.pt_on_edge_vertex(revol_rib, &self.sbase, p1, &mut on1)?;
        x.pt_on_first_edge = on1.pt_on_edge;
        x.on_first_edge = on1.on_edge.clone();
        self.pt_on_edge_vertex(revol_rib, &self.sbase, p2, &mut on2)?;
        x.pt_on_last_edge = on2.pt_on_edge;
        x.on_last_edge = on2.on_edge.clone();
        if x.pt_on_first_edge {
            if let Some(fff) = self.end_face(&on1, true, &cc, f + bnd / 50., bnd, pln)? {
                x.first_face = fff;
            }
            x.first_edge = e.clone();
            x.first_vertex = BRepLibMakeVertex::new(p1)?.vertex()?;
            x.on_first_face = true;
        }
        if x.pt_on_last_edge {
            if let Some(fff) = self.end_face(&on2, false, &cc, l - bnd / 50., bnd, pln)? {
                x.last_face = fff;
            }
            x.last_edge = e.clone();
            x.last_vertex = BRepLibMakeVertex::new(p2)?.vertex()?;
            x.on_last_face = true;
        }
        if !x.first_face.is_null() && !x.last_face.is_null() {
            return Ok(true);
        }
        //--- FirstFace or LastFace was not found
        self.crossing_faces(&cc, l, e, (p1, p2), x)
    }

    /// The faces where the curve of the single edge enters (the point before the last
    /// one at most `l`) and leaves the basis, for the ends not found yet.
    fn crossing_faces(
        &self,
        cc: &Handle<dyn GeomCurve>,
        l: f64,
        e: &Edge,
        (p1, p2): (Point3, Point3),
        x: &mut Extremes,
    ) -> Result<bool, Failure> {
        let mut asi = CSIntersector::with_shape(&self.sbase);
        let mut scur = Sequence::default();
        scur.append(cc.clone());
        asi.perform_curves(&scur)?;
        if !asi.is_done() || asi.nb_points(1)? < 2 {
            return Ok(false);
        }
        let n = asi.nb_points(1)?;
        let (mut lastpar, mut lastindex) = (asi.point(1, n)?.parameter(), n);
        if lastpar > l {
            for jj in (1..n).rev() {
                let par = asi.point(1, jj)?.parameter();
                if par <= l {
                    (lastpar, lastindex) = (par, jj);
                    break;
                }
            }
        }
        let firstindex = lastindex - 1;
        let firstpar = asi.point(1, firstindex)?.parameter();
        let (mut firstpoint, mut lastpoint) = (Point3::default(), Point3::default());
        let c = cc
            .get()
            .ok_or_else(|| crate::errors::null_object("ExtremeFaces", "null curve"))?;
        if x.first_face.is_null() {
            x.first_face = asi.point(1, firstindex)?.face().clone();
            firstpoint = c.d0(firstpar)?;
            x.first_vertex = BRepLibMakeVertex::new(firstpoint)?.vertex()?;
            x.first_edge = e.clone();
        }
        if x.last_face.is_null() {
            x.last_face = asi.point(1, lastindex)?.face().clone();
            lastpoint = c.d0(lastpar)?;
            x.last_vertex = BRepLibMakeVertex::new(lastpoint)?.vertex()?;
            x.last_edge = e.clone();
        }
        if !x.on_first_face {
            x.on_first_face = p1.distance(&firstpoint) <= precision::confusion();
        }
        if !x.on_last_face {
            x.on_last_face = p2.distance(&lastpoint) <= precision::confusion();
        }
        Ok(!(x.first_face.is_null() || x.last_face.is_null()))
    }
}
