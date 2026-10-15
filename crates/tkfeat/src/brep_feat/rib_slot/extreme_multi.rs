// Rust translation of BRepFeat_RibSlot::ExtremeFaces (the wire of several edges) of
// ModelingAlgorithms/TKFeat/BRepFeat/BRepFeat_RibSlot.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The extremities of a wire of several edges: walking the wire in order, the first
//! intersection of a slightly lengthened edge with a face of the basis (not at the end
//! of the edge before the first one is found) gives the first edge, face and vertex,
//! the next one the last ones; a point on an edge or a vertex of the basis chooses
//! among the faces around it.
use super::RibSlot;
use super::extreme::{Extremes, PointOn};
use super::geometry::{choice_of_faces, edge_curve, int_par};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCurve, GeomPlane, GeomTrimmedCurve};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tktopalgo::brep_int_curve_surface::BRepIntCurveSurfaceInter;

/// One intersection point of the current edge: the edge, face and vertex found there.
struct Found {
    edge: Edge,
    face: Face,
    vertex: Vertex,
}

/// The faces around the point: `first` and the faces of the basis other than it holding
/// the edge or the vertex the point lies on (every occurrence, as natively).
fn faces_around(sbase: &Shape, first: &Face, on: &PointOn) -> Result<List<Shape>, Failure> {
    let mut faces = List::default();
    faces.append(first.clone().into_shape());
    let (target, kind): (&Shape, ShapeEnum) = if on.pt_on_vertex {
        (&on.on_vertex, ShapeEnum::Vertex)
    } else {
        (&on.on_edge, ShapeEnum::Edge)
    };
    for fx in Explorer::new(sbase, ShapeEnum::Face, ShapeEnum::Shape)? {
        for sub in Explorer::new(&fx, kind, ShapeEnum::Shape)? {
            if sub.is_same(target) && !fx.is_same(first) {
                faces.append(fx.clone());
            }
        }
    }
    Ok(faces)
}

/// The choice of the faces around a point of the current lengthened edge: whether the
/// form is a revolution rib, its curve, the bound of the basis and the plane.
struct Choice<'a> {
    revol_rib: bool,
    curve: &'a Handle<dyn GeomCurve>,
    bnd: f64,
    pln: &'a Handle<GeomPlane>,
}

/// The state of the walk: whether the first and last ends are found.
#[derive(Default)]
struct Walk {
    first_ok: bool,
    last_ok: bool,
}

impl RibSlot {
    /// The face chosen around a point on an edge or vertex of the basis at `par`, or
    /// `face` itself.
    fn chosen_face(
        &self,
        face: &Face,
        on: &PointOn,
        ch: &Choice,
        par: f64,
    ) -> Result<Face, Failure> {
        if !on.pt_on_edge {
            return Ok(face.clone());
        }
        let mut faces = faces_around(&self.sbase, face, on)?;
        let fff = choice_of_faces(&mut faces, ch.curve, par, ch.bnd / 10., ch.pln)?;
        Ok(if fff.is_null() { face.clone() } else { fff })
    }

    /// The point found taken as the first end (true), its face chosen around it.
    fn take_first(
        &self,
        found: &mut Found,
        on: &PointOn,
        ch: &Choice,
        intpar: f64,
        x: &mut Extremes,
    ) -> Result<bool, Failure> {
        x.first_edge = found.edge.clone();
        x.first_face = found.face.clone();
        x.first_vertex = found.vertex.clone();
        x.pt_on_first_edge = on.pt_on_edge;
        x.on_first_edge = on.on_edge.clone();
        found.edge.nullify();
        found.face.nullify();
        found.vertex.nullify();
        x.first_face = self.chosen_face(&x.first_face, on, ch, intpar + ch.bnd / 10.)?;
        Ok(!x.first_edge.is_null() && !x.first_face.is_null() && !x.first_vertex.is_null())
    }

    /// The point found taken as the last end (true), its face chosen around it.
    fn take_last(
        &self,
        found: &Found,
        on: &PointOn,
        ch: &Choice,
        intpar: f64,
        x: &mut Extremes,
    ) -> Result<bool, Failure> {
        x.last_edge = found.edge.clone();
        x.last_face = found.face.clone();
        x.last_vertex = found.vertex.clone();
        x.pt_on_last_edge = on.pt_on_edge;
        x.on_last_edge = on.on_edge.clone();
        x.last_face = self.chosen_face(&x.last_face, on, ch, intpar - ch.bnd / 10.)?;
        Ok(!x.last_edge.is_null() && !x.last_face.is_null() && !x.last_vertex.is_null())
    }

    /// The intersections of the lengthened edge `e` with the face `cur_face`.
    fn edge_on_face(
        &self,
        ch: &Choice,
        e: &Edge,
        cur_face: &Face,
        on: &mut PointOn,
        walk: &mut Walk,
        x: &mut Extremes,
    ) -> Result<(), Failure> {
        let p2 = BRepTool::pnt(&topexp::last_vertex(e, true))?;
        let tol = BRepTool::face_tolerance(cur_face)?;
        let mut inter = BRepIntCurveSurfaceInter::new();
        inter.init_curve(cur_face, &GeomAdaptorCurve::with_curve(ch.curve)?, tol)?;
        let b = BRepBuilder::default();
        while inter.more() {
            let the_point = inter.pnt()?;
            if !x.first_vertex.is_null()
                && BRepTool::pnt(&x.first_vertex)?.distance(&the_point) <= tol
            {
                inter.next()?;
                continue;
            }
            let intpar = int_par(ch.curve, the_point)?;
            let mut the_vertex = Vertex::default();
            b.make_vertex_at(&mut the_vertex, the_point, precision::confusion())?;
            let mut found = Found {
                edge: e.clone(),
                face: cur_face.clone(),
                vertex: the_vertex,
            };
            if !walk.first_ok && the_point.distance(&p2) <= precision::confusion() {
                inter.next()?;
                continue;
            }
            // ---Find thepoint on an edge or a vertex of face f
            self.pt_on_edge_vertex(ch.revol_rib, cur_face, the_point, on)?;
            if x.first_edge.is_null() {
                walk.first_ok = self.take_first(&mut found, on, ch, intpar, x)?;
            }
            if x.last_edge.is_null() && !found.edge.is_null() && !x.first_edge.is_null() {
                walk.last_ok = self.take_last(&found, on, ch, intpar, x)?;
                break;
            }
            inter.next()?;
        }
        Ok(())
    }

    /// The extremities of a wire of several edges (see the module); false when either
    /// end is not found.
    pub(super) fn extreme_faces_wire(
        &self,
        revol_rib: bool,
        bnd: f64,
        pln: &Handle<GeomPlane>,
        x: &mut Extremes,
    ) -> Result<bool, Failure> {
        let mut walk = Walk::default();
        let mut ex = WireExplorer::with_wire(&self.wire)?;
        while ex.more() {
            let e = ex.current().clone();
            let (cur, f, l) = edge_curve(&e)?;
            let (f, l) = (f - bnd / 10000., l + bnd / 10000.);
            let curve: Handle<dyn GeomCurve> =
                Handle::new(GeomTrimmedCurve::new(&cur, f, l, true, true)?).cast();
            let ch = Choice {
                revol_rib,
                curve: &curve,
                bnd,
                pln,
            };
            let mut on = PointOn::default();
            for fc in Explorer::new(&self.sbase, ShapeEnum::Face, ShapeEnum::Shape)? {
                let cur_face = topods::face(&fc)?;
                self.edge_on_face(&ch, &e, &cur_face, &mut on, &mut walk, x)?;
            }
            ex.next()?;
        }
        if !(walk.first_ok && walk.last_ok) {
            return Ok(false);
        }
        let pp1 = BRepTool::pnt(&topexp::first_vertex(&x.first_edge, true))?;
        let pp2 = BRepTool::pnt(&topexp::last_vertex(&x.last_edge, true))?;
        let p1 = BRepTool::pnt(&x.first_vertex)?;
        let p2 = BRepTool::pnt(&x.last_vertex)?;
        if p1.distance(&pp1) <= BRepTool::face_tolerance(&x.first_face)? {
            x.on_first_face = true;
        }
        if p2.distance(&pp2) <= BRepTool::face_tolerance(&x.last_face)? {
            x.on_last_face = true;
        }
        Ok(true)
    }
}
