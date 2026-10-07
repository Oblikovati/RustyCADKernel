//! HLRBRep fixtures, the Rust twin of tools/oracle/tkhlr_probe_hlrbrep_fixtures.cxx:
//! projectors and located edges read from a request, and the 2D conic printers.
use super::common::{Reader, flag, pnt2d, real, xy};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBezierCurve, GeomCircle, GeomCurve, GeomEllipse, GeomLine,
};
use rustycad_tkgeomalgo::int_res2d::Domain;
use rustycad_tkhlr::hlr_algo::Projector;
use rustycad_tkmath::gp::{Axis2, Circle2, Ellipse2, Line2, Point3};

/// `pk [loc dir xdir [focus]]`: see `hb_projector`.
pub fn projector(r: &mut Reader) -> Result<Projector, Failure> {
    let kind = r.integer();
    if kind == 2 {
        return Projector::new();
    }
    let (loc, dir, xdir) = (r.pnt(), r.dir(), r.dir());
    let frame = Axis2::with_x_direction(loc, dir, xdir)?;
    if kind == 0 {
        return Projector::with_frame(frame);
    }
    Projector::with_frame_focus(frame, r.real())
}

fn poles(r: &mut Reader) -> Array1<'static, Point3> {
    let n = r.integer();
    let mut poles = Array1::with_bounds(1, n).expect("probe poles");
    for i in 1..=n {
        poles[i] = r.pnt();
    }
    poles
}

fn bspline(r: &mut Reader) -> Result<Handle<dyn GeomCurve>, Failure> {
    let poles = poles(r);
    let nk = poles.len() as i32 - 1;
    let mut knots = Array1::with_bounds(1, nk).expect("probe knots");
    let mut mults = Array1::with_bounds(1, nk).expect("probe knots");
    for i in 1..=nk {
        knots[i] = f64::from(i - 1);
        mults[i] = if i == 1 || i == nk { 3 } else { 1 };
    }
    Ok(Handle::new(GeomBSplineCurve::new(&poles, &knots, &mults, 2, false)?).cast())
}

/// `ck ...`: see `hb_geometry`.
pub fn geometry(r: &mut Reader) -> Result<Handle<dyn GeomCurve>, Failure> {
    let kind = r.integer();
    match kind {
        0 => {
            let p = r.pnt();
            return Ok(Handle::new(GeomLine::from_point(p, r.dir())).cast());
        }
        3 => return Ok(Handle::new(GeomBezierCurve::new(&poles(r))?).cast()),
        4 => return bspline(r),
        _ => {}
    }
    let (p, n, x) = (r.pnt(), r.dir(), r.dir());
    let frame = Axis2::with_x_direction(p, n, x)?;
    if kind == 1 {
        return Ok(Handle::new(GeomCircle::new(frame, r.real())?).cast());
    }
    let major = r.real();
    Ok(Handle::new(GeomEllipse::new(frame, major, r.real())?).cast())
}

/// `geometry u1 u2`: see `hb_edge`.
pub fn edge(r: &mut Reader) -> Result<Edge, Failure> {
    let curve = geometry(r)?;
    let (u1, u2) = (r.real(), r.real());
    let mut edge = Edge::default();
    let builder = BRepBuilder::default();
    builder.make_edge_with_curve(&mut edge, &curve, 1e-7)?;
    builder.range(&edge, u1, u2, false)?;
    Ok(edge)
}

pub fn lin2d(l: Line2) {
    pnt2d(l.location());
    xy(l.direction().xy());
}
pub fn circ2d(c: Circle2) {
    pnt2d(c.location());
    xy(c.x_axis().direction().xy());
    xy(c.y_axis().direction().xy());
    real(c.radius());
}
pub fn elips2d(e: Ellipse2) {
    pnt2d(e.location());
    xy(e.x_axis().direction().xy());
    xy(e.y_axis().direction().xy());
    real(e.major_radius());
    real(e.minor_radius());
}

/// `hb_domain`: the bounds of an IntRes2d domain.
pub fn domain(d: &Domain) -> Result<(), Failure> {
    flag(d.has_first_point());
    if d.has_first_point() {
        real(d.first_parameter()?);
        pnt2d(d.first_point()?);
        real(d.first_tolerance()?);
    }
    flag(d.has_last_point());
    if d.has_last_point() {
        real(d.last_parameter()?);
        pnt2d(d.last_point()?);
        real(d.last_tolerance()?);
    }
    flag(d.is_closed());
    if d.is_closed() {
        let (a, b) = d.equivalent_parameters();
        real(a);
        real(b);
    }
    Ok(())
}
