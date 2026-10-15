// Rust translation of BRepOffset_Offset::Init(Path, Edge1, Edge2, Offset, FirstEdge,
// LastEdge, Polynomial, Tol, Conti) and Init(Edge, Offset) of
// ModelingAlgorithms/TKOffset/BRepOffset/BRepOffset_Offset.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The pipe face of an edge between two rail edges (the rails get their p-curves on
//! the pipe, the connectors at the ends are built or completed), and the tube face
//! of a free edge.
use super::curve3d::compute_curve3d;
use super::{
    BRepOffsetOffset, PipeOptions, pipe_failure, update_edge_curve, update_edge_pcurve,
    update_edge_pcurves,
};
use crate::brep_offset::BRepOffsetStatus;
use crate::brep_offset::tool::live;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::update_face;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Face, Wire};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine, Geom2dTrimmedCurve};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface, GeomTrimmedCurve};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeomalgo::geom_fill::GeomFillPipe;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Direction2, Point2};
use rustycad_tkmath::toploc::Location;
use rustycad_tktopalgo::brep_lib::{self, BRepLibMakeFace};

/// The rail of a pipe: its adaptor, its 3d curve (when it has one) and whether that
/// curve is a circle of null radius.
struct Rail {
    adaptor: Handle<dyn Adaptor3dCurve>,
    curve: Option<Handle<dyn GeomCurve>>,
    degenerated: bool,
}

/// The located 3d curve of an edge trimmed to its range and moved by its location.
fn trimmed_located(
    c: &Handle<dyn GeomCurve>,
    f: f64,
    l: f64,
    loc: &Location,
) -> Result<Handle<dyn GeomCurve>, Failure> {
    let t: Handle<dyn GeomCurve> = Handle::new(GeomTrimmedCurve::new(c, f, l, true, true)?).cast();
    live(&t, "curve")?.transform(loc.transformation())?;
    Ok(t)
}

/// The rail of an edge: its 3d curve, or the curve on its first surface.
fn rail(edge: &Edge) -> Result<Rail, Failure> {
    let found = BRepTool::curve_with_location(edge)?;
    if found.curve.is_null() {
        let cos = BRepTool::first_curve_on_surface(edge)?;
        let s1 = live(&cos.surface, "surface")?
            .transformed(cos.location.transformation())?
            .cast::<dyn GeomSurface>();
        let c12d: Handle<dyn Geom2dCurve> = Handle::new(Geom2dTrimmedCurve::new(
            &cos.pcurve,
            cos.first,
            cos.last,
            true,
            true,
        )?)
        .cast();
        let hs1: Handle<dyn Adaptor3dSurface> =
            Handle::new(GeomAdaptorSurface::with_surface(&s1)?).cast();
        let hc1 = Handle::new(Geom2dAdaptorCurve::with_curve(&c12d)?).cast();
        let cons = Adaptor3dCurveOnSurface::with_curve(&hc1, &hs1)?;
        return Ok(Rail {
            adaptor: Handle::new(cons).cast(),
            curve: None,
            degenerated: false,
        });
    }
    let c = trimmed_located(&found.curve, found.first, found.last, &found.location)?;
    let ac = GeomAdaptorCurve::with_curve(&c)?;
    let degenerated =
        ac.get_type()? == CurveType::Circle && ac.circle()?.radius() < precision::confusion();
    Ok(Rail {
        adaptor: Handle::new(ac).cast(),
        curve: Some(c),
        degenerated,
    })
}

/// A 2d line through a point along X or Y.
fn line2d(x: f64, y: f64, along_x: bool) -> Handle<Geom2dLine> {
    let d = if along_x {
        Direction2::X
    } else {
        Direction2::Y
    };
    Handle::new(Geom2dLine::from_point(Point2::new(x, y), d))
}

/// The pipe surface and its parameter box, with the exchange flag and the error.
struct PipeSurface {
    s: Handle<dyn GeomSurface>,
    exch_uv: bool,
    error: f64,
    bounds: (f64, f64, f64, f64),
}

impl BRepOffsetOffset {
    /// `Init(Path, Edge1, Edge2, Offset, FirstEdge, LastEdge, Polynomial, Tol, Conti)`:
    /// the pipe of radius |Offset| along the path between the rails (which get their
    /// p-curves, and a 3d curve when they lack one); the connectors are the given edges
    /// when they share the rail vertices, new edges otherwise; Standard_ConstructionError
    /// when the pipe fails;
    /// `o.init_edges_with_connectors(&path, &e1, &e2, 1.0, &first, &last, PipeOptions::default())?`.
    #[allow(clippy::too_many_arguments)]
    pub fn init_edges_with_connectors(
        &mut self,
        path: &Edge,
        edge1: &Edge,
        edge2: &Edge,
        offset: f64,
        first_edge: &Edge,
        last_edge: &Edge,
        options: PipeOptions,
    ) -> Result<(), Failure> {
        let PipeOptions {
            polynomial,
            tol,
            conti,
        } = options;
        self.my_status = BRepOffsetStatus::Good;
        self.my_shape = path.as_ref().clone();
        let found = BRepTool::curve_with_location(path)?;
        let cp = trimmed_located(&found.curve, found.first, found.last, &found.location)?;
        let hcp: Handle<dyn Adaptor3dCurve> =
            Handle::new(GeomAdaptorCurve::with_curve(&cp)?).cast();
        let r1 = rail(edge1)?;
        let r2 = rail(edge2)?;

        // The pipe.
        let mut pipe =
            GeomFillPipe::with_adaptor_rails(&hcp, &r1.adaptor, &r2.adaptor, offset.abs());
        pipe.perform(tol, polynomial, conti, 11, 30)?;
        if !pipe.is_done() {
            return Err(pipe_failure());
        }
        let s = pipe.surface();
        let ps = PipeSurface {
            bounds: live(&s, "surface")?.bounds(),
            s,
            exch_uv: pipe.exchange_uv(),
            error: pipe.error_on_surf(),
        };
        let (f1, l1, f2, l2) = ps.bounds;

        // The face.
        let path_tol = BRepTool::edge_tolerance(path)?;
        let builder = BRepBuilder::default();
        let mut my_face = Face::default();
        builder.make_face(&mut my_face);
        let id = Location::default();
        builder.update_face(&my_face, &ps.s, &id, path_tol)?;

        // The rails (which already have a 3d curve unless built from the pipe).
        let rail1 = if ps.exch_uv {
            (line2d(0., f2, true), (f1, l1), false, f2)
        } else {
            (line2d(f1, 0., false), (f2, l2), true, f1)
        };
        update_rail(edge1, &r1, &ps, &my_face, rail1, tol, path_tol, true)?;
        let rail2 = if ps.exch_uv {
            (line2d(0., l2, true), (f1, l1), false, l2)
        } else {
            (line2d(l1, 0., false), (f2, l2), true, l1)
        };
        update_rail(edge2, &r2, &ps, &my_face, rail2, tol, path_tol, false)?;

        // The connectors.
        let (v1f_path, v1l_path) = topexp::vertices(path, false);
        let is_closed = v1f_path.is_same(&v1l_path);
        let (v1f, v1l) = topexp::vertices(edge1, false);
        let (v2f, v2l) = topexp::vertices(edge2, false);
        let start_degenerated = v1f.is_same(&v2f);
        let end_degenerated = v1l.is_same(&v2l);
        let (edge3, e3rev) = connector(first_edge, &v1f, &v2f)?;
        let tol_app = precision::approximation();
        let edge4;
        let flip = |u1: f64, u2: f64, rev: bool| if rev { (-u2, -u1) } else { (u1, u2) };
        if is_closed {
            edge4 = edge3.clone();
            // With exchanged parameters the wire is reversed, so the forward edge in the
            // face is E4: hence L1 and L2.
            let (l1c, l2c, range) = if ps.exch_uv {
                (line2d(l1, 0., false), line2d(f1, 0., false), (f2, l2))
            } else {
                (line2d(0., f2, true), line2d(0., l2, true), (f1, l1))
            };
            if e3rev {
                live(&l1c, "line")?.reverse()?;
                live(&l2c, "line")?.reverse()?;
            }
            let (u1, u2) = flip(range.0, range.1, e3rev);
            let (c1, c2): (Handle<dyn Geom2dCurve>, Handle<dyn Geom2dCurve>) =
                (l1c.clone().cast(), l2c.cast());
            update_edge_pcurves(&edge3, &c1, &c2, &my_face, path_tol)?;
            builder.range_on_face(&edge3, &my_face, u1, u2)?;
            if start_degenerated {
                builder.degenerated(&edge3, true)?;
            } else if first_edge.is_null() {
                // The 3d curve is not computed yet.
                compute_curve3d(&edge3, &c1, &ps.s, &id, tol_app)?;
            }
        } else {
            let e4rev;
            (edge4, e4rev) = connector(last_edge, &v1l, &v2l)?;
            let (l1c, range) = if ps.exch_uv {
                (line2d(f1, 0., false), (f2, l2))
            } else {
                (line2d(0., f2, true), (f1, l1))
            };
            if e3rev {
                live(&l1c, "line")?.reverse()?;
            }
            let (u1, u2) = flip(range.0, range.1, e3rev);
            let c1: Handle<dyn Geom2dCurve> = l1c.cast();
            update_edge_pcurve(&edge3, &c1, &my_face, path_tol)?;
            builder.range_on_face(&edge3, &my_face, u1, u2)?;
            if start_degenerated {
                builder.degenerated(&edge3, true)?;
            } else if first_edge.is_null() {
                compute_curve3d(&edge3, &c1, &ps.s, &id, tol_app)?;
            }
            let (l2c, range) = if ps.exch_uv {
                (line2d(l1, 0., false), (f2, l2))
            } else {
                (line2d(0., l2, true), (f1, l1))
            };
            if e4rev {
                live(&l2c, "line")?.reverse()?;
            }
            let (u1, u2) = flip(range.0, range.1, e4rev);
            let c2: Handle<dyn Geom2dCurve> = l2c.cast();
            update_edge_pcurve(&edge4, &c2, &my_face, path_tol)?;
            builder.range_on_face(&edge4, &my_face, u1, u2)?;
            if end_degenerated {
                builder.degenerated(&edge4, true)?;
            } else if last_edge.is_null() {
                compute_curve3d(&edge4, &c2, &ps.s, &id, tol_app)?;
            }
        }

        // SameParameter of the given connectors.
        if !first_edge.is_null() && !start_degenerated {
            same_parameter_connector(&edge3, path_tol, tol)?;
        }
        if !last_edge.is_null() && !end_degenerated {
            same_parameter_connector(&edge4, path_tol, tol)?;
        }

        let mut w = Wire::default();
        builder.make_wire(&mut w);
        let mut w_shape = w.into_shape();
        builder.add(&mut w_shape, &edge1.oriented(Orientation::Reversed))?;
        builder.add(&mut w_shape, &edge2.oriented(Orientation::Forward))?;
        builder.add(&mut w_shape, &edge4.reversed())?;
        builder.add(&mut w_shape, edge3.as_ref())?;
        if ps.exch_uv {
            w_shape.reverse();
        }
        let mut face_shape = my_face.as_ref().clone();
        builder.add(&mut face_shape, &w_shape)?;
        if ps.exch_uv {
            my_face.reverse();
        }
        update_face(&my_face)?;
        if edge1.orientation() == Orientation::Reversed {
            my_face.reverse();
        }
        self.my_face = my_face;
        Ok(())
    }

    /// `Init(Edge, Offset)`: the tube of radius |Offset| around the edge, reversed for
    /// a negative offset (the status is kept); Standard_ConstructionError when the
    /// pipe fails; `o.init_edge(&edge, 1.0)?`.
    pub fn init_edge(&mut self, edge: &Edge, offset: f64) -> Result<(), Failure> {
        self.my_shape = edge.as_ref().clone();
        let my_offset = offset.abs();
        let found = BRepTool::curve_with_location(edge)?;
        let cp = trimmed_located(&found.curve, found.first, found.last, &found.location)?;
        let mut pipe = GeomFillPipe::with_radius(&cp, my_offset)?;
        pipe.perform_with_parameters(false, false)?;
        if !pipe.is_done() {
            return Err(pipe_failure());
        }
        let mf = BRepLibMakeFace::from_surface(&pipe.surface(), precision::confusion())?;
        self.my_face = mf.face()?;
        if offset < 0. {
            self.my_face.reverse();
        }
        Ok(())
    }
}

/// The p-curve of a rail on the pipe face: `(PC, (U1, U2), u_iso, iso)` with the
/// iso of the pipe surface that is the 3d curve of a rail without one.
type RailCurve = (Handle<Geom2dLine>, (f64, f64), bool, f64);

/// The update of a rail edge (`Edge1` when `first`, `Edge2` otherwise): its 3d curve
/// when it has none (the pipe iso) or a null one when degenerated, its p-curve on the
/// face, its ranges and, for exact pipes, same parameter.
#[allow(clippy::too_many_arguments)]
fn update_rail(
    edge: &Edge,
    r: &Rail,
    ps: &PipeSurface,
    my_face: &Face,
    (pc, (u1, u2), u_iso, iso): RailCurve,
    tol: f64,
    path_tol: f64,
    first: bool,
) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    let id = Location::default();
    let is_3d = r.curve.is_some();
    if !is_3d {
        let s = live(&ps.s, "surface")?;
        let c = if u_iso { s.u_iso(iso)? } else { s.v_iso(iso)? };
        update_edge_curve(edge, &c, &id, BRepTool::edge_tolerance(edge)?)?;
    } else if r.degenerated {
        update_edge_curve(
            edge,
            &Handle::default(),
            &id,
            BRepTool::edge_tolerance(edge)?,
        )?;
        builder.degenerated(edge, true)?;
    }
    let mut the_tol = path_tol.max(BRepTool::edge_tolerance(edge)? + ps.error);
    let pc: Handle<dyn Geom2dCurve> = pc.cast();
    update_edge_pcurve(edge, &pc, my_face, the_tol)?;
    // The new p-curve at the same range.
    if !first {
        builder.same_range(edge, false)?;
    }
    if !is_3d && !r.degenerated {
        if first {
            builder.same_range(edge, false)?;
        }
        builder.range(edge, u1, u2, true)?;
    }
    builder.range_on_face(edge, my_face, u1, u2)?;
    brep_lib::same_range(edge, 1.0e-5)?;
    // Same parameter for the particular cases.
    if ps.error == 0. {
        the_tol = the_tol.max(tol);
        builder.same_parameter(edge, false)?;
        brep_lib::same_parameter_edge(edge, the_tol)?;
    }
    Ok(())
}

/// The connector at an end of the rails: the given edge (forward, reversed when it
/// starts at the second rail; `(edge, reversed)`) when it shares a rail vertex, a new
/// edge from the first rail vertex to the second otherwise.
fn connector(
    given: &Edge,
    v1: &topods::Vertex,
    v2: &topods::Vertex,
) -> Result<(Edge, bool), Failure> {
    let builder = BRepBuilder::default();
    let new_edge = || -> Result<Edge, Failure> {
        let mut e = Edge::default();
        builder.make_edge(&mut e)?;
        let mut shape = e.as_ref().clone();
        builder.add(&mut shape, &v1.oriented(Orientation::Forward))?;
        builder.add(&mut shape, &v2.oriented(Orientation::Reversed))?;
        Ok(e)
    };
    if given.is_null() {
        return Ok((new_edge()?, false));
    }
    let mut e = topods::edge(&given.oriented(Orientation::Forward))?;
    let (vvf, _) = topexp::vertices(&e, false);
    if !vvf.is_same(v1) && !vvf.is_same(v2) {
        // Most likely an inconsistent input: another edge, sewn afterwards.
        return Ok((new_edge()?, false));
    }
    if !vvf.is_same(v1) {
        e.reverse();
        return Ok((e, true));
    }
    Ok((e, false))
}

/// The 3d curve and same parameter of a given connector.
fn same_parameter_connector(e: &Edge, path_tol: f64, tol: f64) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    let (continuity, max_degree, max_segment) = brep_lib::BUILD_CURVE3D_DEFAULTS;
    brep_lib::build_curve3d(e, path_tol, continuity, max_degree, max_segment)?;
    builder.same_range(e, false)?;
    builder.same_parameter(e, false)?;
    brep_lib::same_parameter_edge(e, tol)
}
