//! HelixBRep probe operations, the Rust twin of tools/oracle/tkhelix_probe_helixbrep.cxx.
use super::common::{Reader, attempt, end, flag, int, pnt, real};
use super::helix_geom::{bspline, shape};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Shape, Vertex};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomBSplineCurve;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkhelix::helix_brep::HelixBRepBuilderHelix;
use rustycad_tkmath::gp::Axis3;

/// `hxb_array`: `lo n (v)...`, an array of n values from the lower bound lo.
fn array<T: Clone + Default>(r: &mut Reader<'_>, convert: fn(f64) -> T) -> Array1<'static, T> {
    let (lo, n) = (r.integer(), r.integer());
    let mut a: Array1<'static, T> = Array1::with_bounds(lo, lo + n - 1).expect("probe bounds");
    for i in lo..lo + n {
        *a.change_value(i).expect("probe index") = convert(r.real());
    }
    a
}

fn reals(r: &mut Reader<'_>) -> Array1<'static, f64> {
    array(r, |v| v)
}

fn flags(r: &mut Reader<'_>) -> Array1<'static, bool> {
    // T(r.real()) for bool: a nonzero double is true.
    array(r, |v| v != 0.0)
}

/// `hxb_vertex`: null flag, point and tolerance.
fn vertex(v: &Vertex) -> Result<(), Failure> {
    flag(v.is_null());
    if v.is_null() {
        return Ok(());
    }
    pnt(BRepTool::pnt(v)?);
    real(BRepTool::vertex_tolerance(v)?);
    Ok(())
}

/// `hxb_edge`: orientation, tolerance, curve range and B-spline, vertices.
fn edge(e: &Edge) -> Result<(), Failure> {
    int(e.orientation() as i32);
    real(BRepTool::edge_tolerance(e)?);
    let (c, f, l) = BRepTool::curve(e)?;
    real(f);
    real(l);
    let b: Handle<GeomBSplineCurve> = c.cast();
    bspline(&b)?;
    let (v1, v2) = topexp::vertices(e, false);
    vertex(&v1)?;
    vertex(&v2)
}

/// `hxb_shape`: null flag, type, orientation, edge count and edges.
fn result_shape(s: &Shape) -> Result<(), Failure> {
    flag(s.is_null());
    if s.is_null() {
        return Ok(());
    }
    int(s.shape_type()? as i32);
    int(s.orientation() as i32);
    let mut edges: Vec<Edge> = Vec::new();
    let mut x = Explorer::new(s, ShapeEnum::Edge, ShapeEnum::Shape)?;
    while x.more() {
        edges.push(topods::edge(x.current())?);
        x.next()?;
    }
    int(edges.len() as i32);
    edges.iter().try_for_each(edge)
}

/// `hxb_state`: statuses, reached tolerance and the shape.
fn state(b: &HelixBRepBuilderHelix) -> Result<(), Failure> {
    int(b.error_status());
    int(b.warning_status());
    real(b.tolerance_reached());
    result_shape(b.shape())
}

/// `hxb_set`: reads the data of one SetParameters overload, then calls it.
fn set(r: &mut Reader<'_>, b: &mut HelixBRepBuilderHelix, ax: &Axis3) -> Result<(), Failure> {
    match r.integer() {
        0 => {
            let (d, h, p, f) = (reals(r), reals(r), reals(r), flags(r));
            b.set_parameters(ax, &d, &h, &p, &f)
        }
        1 => {
            let (d, h, p, f) = (r.real(), reals(r), reals(r), flags(r));
            b.set_parameters_diam(ax, d, &h, &p, &f)
        }
        2 => {
            let (d1, d2, h, p, f) = (r.real(), r.real(), reals(r), reals(r), flags(r));
            b.set_parameters_spiral(ax, d1, d2, &h, &p, &f)
        }
        3 => {
            let (d, p, t) = (reals(r), reals(r), reals(r));
            b.set_parameters_turns(ax, &d, &p, &t)
        }
        4 => {
            let (d, p, t) = (r.real(), reals(r), reals(r));
            b.set_parameters_diam_turns(ax, d, &p, &t)
        }
        _ => {
            let (d1, d2, p, t) = (r.real(), r.real(), reals(r), reals(r));
            b.set_parameters_spiral_turns(ax, d1, d2, &p, &t)
        }
    }
}

/// `P N X indirect nsteps (step)...`: SetParameters, SetApproxParameters and Perform
/// steps on one HelixBRep_BuilderHelix.
pub fn build_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (o, n, x, indirect) = (r.pnt(), r.dir(), r.dir(), r.flag());
    // The corpus only spells valid frames; gp_Ax3 would raise for a parallel X.
    let mut ax = Axis3::new(o, n, x).expect("probe frame");
    if indirect {
        ax.y_reverse();
    }
    let mut b = HelixBRepBuilderHelix::new();
    attempt(|| state(&b));
    for _ in 0..r.integer() {
        match r.integer() {
            0 => attempt(|| set(&mut r, &mut b, &ax)),
            1 => {
                let (tol, deg, cont) = (r.real(), r.integer(), r.integer());
                b.set_approx_parameters(tol, deg, shape(cont));
            }
            _ => attempt(|| b.perform()),
        }
        attempt(|| state(&b));
    }
    end();
}
