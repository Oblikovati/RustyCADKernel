//! BRepLib edge probe operations (`BRepLib_MakeVertex`, `BRepLib_MakeEdge`,
//! `BRepLib_MakeEdge2d`), the Rust twin of tools/oracle/tktopalgo_probe_breplib_edge.cxx
//! (the request layout is documented there).
use super::brep_lib_common::{
    circ, circ2d, elips, elips2d, geom_curve, geom2d_curve, hypr, hypr2d, lin, lin2d, parab,
    parab2d, shape, vertex_ref, vertices,
};
use super::common::{Reader, attempt, end, flag, int};
use super::face::surface;
use rustycad_tkbrep::topods::{Shape, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomPlane, GeomSurface};
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tktopalgo::brep_lib::{self, BRepLibMakeEdge, BRepLibMakeEdge2d, BRepLibMakeVertex};

/// `tp_vertex_index`: the index of the input vertex `v` is the same as, -1 for none.
pub fn vertex_index(v: &Shape, vs: &[Vertex]) -> i32 {
    vs.iter()
        .position(|input| !v.is_null() && v.is_same(input))
        .map_or(-1, |i| i as i32)
}

/// `TpPrecisionScope`: sets BRepLib::Precision for one request, restored on drop.
pub struct PrecisionScope;
impl PrecisionScope {
    pub fn new(p: f64) -> Self {
        if p > 0. {
            brep_lib::set_precision(p);
        }
        Self
    }
}
impl Drop for PrecisionScope {
    fn drop(&mut self) {
        brep_lib::set_precision(precision::confusion());
    }
}

/// `TpPlaneScope`: sets BRepLib::Plane for one request (0, or 1 P N), reset on drop.
struct PlaneScope;
impl PlaneScope {
    fn new(r: &mut Reader<'_>) -> Result<Self, Failure> {
        if r.flag() {
            let p = r.pnt();
            brep_lib::set_plane(&Handle::new(GeomPlane::from_point(p, r.dir())?));
        }
        Ok(Self)
    }
}
impl Drop for PlaneScope {
    fn drop(&mut self) {
        brep_lib::set_plane(&Handle::default());
    }
}

/// The trailing arguments of a curve constructor or Init (`j` in the native header).
enum Tail<P> {
    Whole,
    Params(f64, f64),
    Points(P, P),
    Vertices(Vertex, Vertex),
    PointsParams(P, P, f64, f64),
    VerticesParams(Vertex, Vertex, f64, f64),
}
fn tail<P>(
    j: i32,
    r: &mut Reader<'_>,
    vs: &[Vertex],
    point: impl Fn(&mut Reader<'_>) -> P,
) -> Tail<P> {
    match j {
        0 => Tail::Whole,
        1 => {
            let p1 = r.real();
            Tail::Params(p1, r.real())
        }
        2 | 4 => {
            let (p, q) = (point(r), point(r));
            if j == 2 {
                return Tail::Points(p, q);
            }
            let p1 = r.real();
            Tail::PointsParams(p, q, p1, r.real())
        }
        _ => {
            let (v1, v2) = (vertex_ref(r, vs), vertex_ref(r, vs));
            if j == 3 {
                return Tail::Vertices(v1, v2);
            }
            let p1 = r.real();
            Tail::VerticesParams(v1, v2, p1, r.real())
        }
    }
}
fn point3(r: &mut Reader<'_>) -> Point3 {
    r.pnt()
}
fn point2(r: &mut Reader<'_>) -> Point2 {
    r.pnt2d()
}

/// The Init member of a Geom curve (ids 30..).
fn init_curve(
    m: &mut BRepLibMakeEdge,
    c: &Handle<dyn GeomCurve>,
    t: Tail<Point3>,
) -> Result<(), Failure> {
    match t {
        Tail::Whole => m.init_curve(c),
        Tail::Params(p1, p2) => m.init_curve_params(c, p1, p2),
        Tail::Points(a, b) => m.init_curve_points(c, a, b),
        Tail::Vertices(v1, v2) => m.init_curve_vertices(c, &v1, &v2),
        Tail::PointsParams(a, b, p1, p2) => m.init_curve_points_params(c, a, b, p1, p2),
        Tail::VerticesParams(v1, v2, p1, p2) => m.init_curve_vertices_params(c, &v1, &v2, p1, p2),
    }
}
/// The Init member of a curve on surface (ids 40..).
fn init_pcurve(
    m: &mut BRepLibMakeEdge,
    c: &Handle<dyn Geom2dCurve>,
    s: &Handle<dyn GeomSurface>,
    t: Tail<Point3>,
) -> Result<(), Failure> {
    match t {
        Tail::Whole => m.init_pcurve(c, s),
        Tail::Params(p1, p2) => m.init_pcurve_params(c, s, p1, p2),
        Tail::Points(a, b) => m.init_pcurve_points(c, s, a, b),
        Tail::Vertices(v1, v2) => m.init_pcurve_vertices(c, s, &v1, &v2),
        Tail::PointsParams(a, b, p1, p2) => m.init_pcurve_points_params(c, s, a, b, p1, p2),
        Tail::VerticesParams(v1, v2, p1, p2) => {
            m.init_pcurve_vertices_params(c, s, &v1, &v2, p1, p2)
        }
    }
}
/// A later call of `blm_edge`.
fn edge_init(
    m: &mut BRepLibMakeEdge,
    id: i32,
    r: &mut Reader<'_>,
    vs: &[Vertex],
) -> Result<(), Failure> {
    if id < 40 {
        let c = geom_curve(r)?;
        let t = tail(id - 30, r, vs, point3);
        return init_curve(m, &c, t);
    }
    let c = geom2d_curve(r)?;
    let s = surface(r)?;
    let t = tail(id - 40, r, vs, point3);
    init_pcurve(m, &c, &s, t)
}

/// The constructors of a Geom curve (ids 30..).
fn curve_ctor(c: &Handle<dyn GeomCurve>, t: Tail<Point3>) -> Result<BRepLibMakeEdge, Failure> {
    match t {
        Tail::Whole => BRepLibMakeEdge::from_curve(c),
        Tail::Params(p1, p2) => BRepLibMakeEdge::from_curve_params(c, p1, p2),
        Tail::Points(a, b) => BRepLibMakeEdge::from_curve_points(c, a, b),
        Tail::Vertices(v1, v2) => BRepLibMakeEdge::from_curve_vertices(c, &v1, &v2),
        Tail::PointsParams(a, b, p1, p2) => {
            BRepLibMakeEdge::from_curve_points_params(c, a, b, p1, p2)
        }
        Tail::VerticesParams(v1, v2, p1, p2) => {
            BRepLibMakeEdge::from_curve_vertices_params(c, &v1, &v2, p1, p2)
        }
    }
}
/// The constructors of a curve on surface (ids 40..).
fn pcurve_ctor(
    c: &Handle<dyn Geom2dCurve>,
    s: &Handle<dyn GeomSurface>,
    t: Tail<Point3>,
) -> Result<BRepLibMakeEdge, Failure> {
    match t {
        Tail::Whole => BRepLibMakeEdge::from_pcurve(c, s),
        Tail::Params(p1, p2) => BRepLibMakeEdge::from_pcurve_params(c, s, p1, p2),
        Tail::Points(a, b) => BRepLibMakeEdge::from_pcurve_points(c, s, a, b),
        Tail::Vertices(v1, v2) => BRepLibMakeEdge::from_pcurve_vertices(c, s, &v1, &v2),
        Tail::PointsParams(a, b, p1, p2) => {
            BRepLibMakeEdge::from_pcurve_points_params(c, s, a, b, p1, p2)
        }
        Tail::VerticesParams(v1, v2, p1, p2) => {
            BRepLibMakeEdge::from_pcurve_vertices_params(c, s, &v1, &v2, p1, p2)
        }
    }
}

/// The four constructors of one gp curve kind (j = 0..3).
macro_rules! gp_ctor {
    ($maker:ty, $t:expr, $g:expr, $plain:ident, $params:ident, $points:ident, $vertices:ident) => {
        match $t {
            Tail::Whole => <$maker>::$plain($g),
            Tail::Params(p1, p2) => <$maker>::$params($g, p1, p2),
            Tail::Points(a, b) => <$maker>::$points($g, a, b),
            Tail::Vertices(v1, v2) => <$maker>::$vertices($g, &v1, &v2),
            _ => unreachable!("gp constructors take j in 0..=3"),
        }
    };
}

fn edge_first(id: i32, r: &mut Reader<'_>, vs: &[Vertex]) -> Result<BRepLibMakeEdge, Failure> {
    type M = BRepLibMakeEdge;
    match id {
        0 => return Ok(M::new()),
        1 => {
            let v1 = vertex_ref(r, vs);
            return M::from_vertices(&v1, &vertex_ref(r, vs));
        }
        2 => {
            let a = r.pnt();
            return M::from_points(a, r.pnt());
        }
        40.. => {
            let (c, s) = (geom2d_curve(r)?, surface(r)?);
            return pcurve_ctor(&c, &s, tail(id - 40, r, vs, point3));
        }
        30.. => {
            let c = geom_curve(r)?;
            return curve_ctor(&c, tail(id - 30, r, vs, point3));
        }
        _ => {}
    }
    let (k, j) = ((id - 10) / 4, (id - 10) % 4);
    match k {
        0 => {
            let g = lin(r);
            gp_ctor!(
                M,
                tail(j, r, vs, point3),
                g,
                from_lin,
                from_lin_params,
                from_lin_points,
                from_lin_vertices
            )
        }
        1 => {
            let g = circ(r)?;
            gp_ctor!(
                M,
                tail(j, r, vs, point3),
                g,
                from_circ,
                from_circ_params,
                from_circ_points,
                from_circ_vertices
            )
        }
        2 => {
            let g = elips(r)?;
            gp_ctor!(
                M,
                tail(j, r, vs, point3),
                g,
                from_elips,
                from_elips_params,
                from_elips_points,
                from_elips_vertices
            )
        }
        3 => {
            let g = hypr(r)?;
            gp_ctor!(
                M,
                tail(j, r, vs, point3),
                g,
                from_hypr,
                from_hypr_params,
                from_hypr_points,
                from_hypr_vertices
            )
        }
        _ => {
            let g = parab(r)?;
            gp_ctor!(
                M,
                tail(j, r, vs, point3),
                g,
                from_parab,
                from_parab_params,
                from_parab_points,
                from_parab_vertices
            )
        }
    }
}

/// The constructors (and Init members) of a Geom2d curve for MakeEdge2d.
fn curve2d_ctor(
    c: &Handle<dyn Geom2dCurve>,
    t: Tail<Point2>,
) -> Result<BRepLibMakeEdge2d, Failure> {
    type M = BRepLibMakeEdge2d;
    match t {
        Tail::Whole => M::from_curve(c),
        Tail::Params(p1, p2) => M::from_curve_params(c, p1, p2),
        Tail::Points(a, b) => M::from_curve_points(c, a, b),
        Tail::Vertices(v1, v2) => M::from_curve_vertices(c, &v1, &v2),
        Tail::PointsParams(a, b, p1, p2) => M::from_curve_points_params(c, a, b, p1, p2),
        Tail::VerticesParams(v1, v2, p1, p2) => M::from_curve_vertices_params(c, &v1, &v2, p1, p2),
    }
}
fn init_curve2d(
    m: &mut BRepLibMakeEdge2d,
    c: &Handle<dyn Geom2dCurve>,
    t: Tail<Point2>,
) -> Result<(), Failure> {
    match t {
        Tail::Whole => m.init_curve(c),
        Tail::Params(p1, p2) => m.init_curve_params(c, p1, p2),
        Tail::Points(a, b) => m.init_curve_points(c, a, b),
        Tail::Vertices(v1, v2) => m.init_curve_vertices(c, &v1, &v2),
        Tail::PointsParams(a, b, p1, p2) => m.init_curve_points_params(c, a, b, p1, p2),
        Tail::VerticesParams(v1, v2, p1, p2) => m.init_curve_vertices_params(c, &v1, &v2, p1, p2),
    }
}
fn edge2d_first(id: i32, r: &mut Reader<'_>, vs: &[Vertex]) -> Result<BRepLibMakeEdge2d, Failure> {
    type M = BRepLibMakeEdge2d;
    match id {
        1 => {
            let v1 = vertex_ref(r, vs);
            return M::from_vertices(&v1, &vertex_ref(r, vs));
        }
        2 => {
            let a = r.pnt2d();
            return M::from_points(a, r.pnt2d());
        }
        30.. => {
            let c = geom2d_curve(r)?;
            return curve2d_ctor(&c, tail(id - 30, r, vs, point2));
        }
        _ => {}
    }
    let (k, j) = ((id - 10) / 4, (id - 10) % 4);
    match k {
        0 => {
            let g = lin2d(r);
            gp_ctor!(
                M,
                tail(j, r, vs, point2),
                g,
                from_lin2d,
                from_lin2d_params,
                from_lin2d_points,
                from_lin2d_vertices
            )
        }
        1 => {
            let g = circ2d(r)?;
            gp_ctor!(
                M,
                tail(j, r, vs, point2),
                g,
                from_circ2d,
                from_circ2d_params,
                from_circ2d_points,
                from_circ2d_vertices
            )
        }
        2 => {
            let g = elips2d(r)?;
            gp_ctor!(
                M,
                tail(j, r, vs, point2),
                g,
                from_elips2d,
                from_elips2d_params,
                from_elips2d_points,
                from_elips2d_vertices
            )
        }
        3 => {
            let g = hypr2d(r)?;
            gp_ctor!(
                M,
                tail(j, r, vs, point2),
                g,
                from_hypr2d,
                from_hypr2d_params,
                from_hypr2d_points,
                from_hypr2d_vertices
            )
        }
        _ => {
            let g = parab2d(r)?;
            gp_ctor!(
                M,
                tail(j, r, vs, point2),
                g,
                from_parab2d,
                from_parab2d_params,
                from_parab2d_points,
                from_parab2d_vertices
            )
        }
    }
}

/// `tp_edge_result` on the accessors both makers share.
macro_rules! edge_result {
    ($m:expr, $error_known:expr, $vs:expr) => {{
        flag($m.is_done());
        if $error_known {
            int($m.error() as i32);
        }
        attempt(|| shape(&$m.edge()?.into_shape()));
        attempt(|| {
            let v = $m.vertex1()?;
            shape(v)?;
            int(vertex_index(v, $vs));
            Ok(())
        });
        attempt(|| {
            let v = $m.vertex2()?;
            shape(v)?;
            int(vertex_index(v, $vs));
            Ok(())
        });
    }};
}

fn edge_body(r: &mut Reader<'_>, vs: &[Vertex]) -> Result<(), Failure> {
    let n = r.integer();
    let first = r.integer();
    let mut m = edge_first(first, r, vs)?;
    for _ in 1..n {
        let id = r.integer();
        edge_init(&mut m, id, r, vs)?;
    }
    edge_result!(m, true, vs);
    Ok(())
}
/// `blm_edge`.
pub fn edge_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let _scope = PrecisionScope::new(r.real());
    attempt(|| {
        let vs = vertices(&mut r)?;
        edge_body(&mut r, &vs)
    });
    end();
}

fn edge2d_body(r: &mut Reader<'_>, vs: &[Vertex]) -> Result<(), Failure> {
    let n = r.integer();
    let first = r.integer();
    let mut m = edge2d_first(first, r, vs)?;
    for _ in 1..n {
        let id = r.integer();
        let c = geom2d_curve(r)?;
        init_curve2d(&mut m, &c, tail(id - 30, r, vs, point2))?;
    }
    let error_known = !m.is_done();
    edge_result!(m, error_known, vs);
    Ok(())
}
/// `blm_edge2d`.
pub fn edge2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let _scope = PrecisionScope::new(r.real());
    attempt(|| {
        let _plane = PlaneScope::new(&mut r)?;
        let vs = vertices(&mut r)?;
        edge2d_body(&mut r, &vs)
    });
    end();
}

/// `blm_vertex`.
pub fn vertex_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let _scope = PrecisionScope::new(r.real());
    let p = r.pnt();
    attempt(|| {
        let m = BRepLibMakeVertex::new(p)?;
        flag(m.is_done());
        attempt(|| shape(&m.vertex()?.into_shape()));
        Ok(())
    });
    end();
}
