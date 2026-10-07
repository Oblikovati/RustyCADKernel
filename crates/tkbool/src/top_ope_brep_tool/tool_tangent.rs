// Rust translation of TopOpeBRepTool_TOOL::TggeomE (both overloads) and the static
// CheckEdgeLength of ModelingAlgorithms/TKBool/TopOpeBRepTool/TopOpeBRepTool_TOOL.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The unit geometric tangent of an edge at a parameter.
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Edge, Shape};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::IndexedMap;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Vector3;

/// `CheckEdgeLength(E)` (static): true unless the edge has a single vertex and the
/// polyline of 11 points of its curve is shorter than Precision::Confusion().
fn check_edge_length(e: &Edge) -> Result<bool, Failure> {
    let bc = BRepAdaptorCurve::with_edge(e)?;
    let mut am: IndexedMap<Shape, ShapeMapHasher> = IndexedMap::default();
    topexp::map_shapes_of_type(e.as_ref(), ShapeEnum::Vertex, &mut am)?;
    if am.len() != 1 {
        return Ok(true);
    }
    let an: i32 = 10;
    let (f, l) = (bc.first_parameter()?, bc.last_parameter()?);
    let dt = (l - f) / f64::from(an);
    let mut p1 = bc.value(f)?;
    let mut ln: f64 = 0.;
    for i in 1..=an {
        let p2 = if i == an {
            bc.value(l)?
        } else {
            bc.value(f + f64::from(i) * dt)?
        };
        ln += p1.distance(&p2);
        p1 = p2;
    }
    Ok(ln > precision::confusion())
}

/// `TopOpeBRepTool_TOOL::TggeomE(par, BC, Tg)`: the unit tangent at a parameter on the
/// curve range (or on a bound within the curve resolution of its tolerance); `None`
/// outside; `tggeom_e_on_curve(0.5, &bc)?`.
pub fn tggeom_e_on_curve(par: f64, bc: &BRepAdaptorCurve) -> Result<Option<Vector3>, Failure> {
    let (f, l) = (bc.first_parameter()?, bc.last_parameter()?);
    let tol_e = bc.tolerance()?;
    let tolp = bc.resolution(tol_e)?;
    let onf = (f - par).abs() < tolp;
    let onl = (l - par).abs() < tolp;
    let inbounds = f < par && par < l;
    if !inbounds && !onf && !onl {
        return Ok(None);
    }
    let (_, tg) = bc.d1(par)?;
    Ok(Some(tg.normalized()?))
}

/// `TopOpeBRepTool_TOOL::TggeomE(par, E, Tg)`: the unit tangent of a non-degenerated
/// edge of non-null length; `None` otherwise; `tggeom_e(0.5, &edge)?`.
pub fn tggeom_e(par: f64, e: &Edge) -> Result<Option<Vector3>, Failure> {
    if BRepTool::degenerated(e)? {
        return Ok(None);
    }
    let bc = BRepAdaptorCurve::with_edge(e)?;
    if !check_edge_length(e)? {
        return Ok(None);
    }
    tggeom_e_on_curve(par, &bc)
}
