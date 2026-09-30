// Rust translation of ModelingData/TKBRep/BRepLProp/BRepLProp.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepLProp::Continuity`: the continuity of two edges at their junction.
use super::BRepLPropCLProps;
use crate::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::Vector3;

/// The derivative order the properties of a curve of continuity `shape` compute: 3
/// from C3 (`>= 5`), 2 for C2 (4), 1 for C1 (2), else 0.
fn derivative_order(shape: Shape) -> i32 {
    let rank = shape as i32;
    if rank >= 5 {
        3
    } else if rank == 4 {
        2
    } else if rank == 2 {
        1
    } else {
        0
    }
}

/// `v` reversed when the edge of `curve` is REVERSED.
fn along_edge(curve: &BRepAdaptorCurve, mut v: Vector3) -> Vector3 {
    if curve.edge().orientation() == Orientation::Reversed {
        v.reverse();
    }
    v
}

/// The tolerances of the comparisons: linear (`tl`) and angular (`ta`).
#[derive(Clone, Copy)]
struct Tolerances {
    linear: f64,
    angular: f64,
}

/// `BRepLProp::Continuity(C1, C2, u1, u2, tl, ta)`: the continuity at the junction of
/// `c1` at `u1` and `c2` at `u2`, compared on the derivatives oriented by the edges:
/// C1 or G1, then C2; CN for a periodic edge joined to itself with at least G1. The
/// points must coincide within `tl`, else Standard_Failure ("Courbes non jointives");
/// `continuity_with_tolerances(&c1, &c2, 1.0, 0.0, 1e-7, 1e-12)?`.
pub fn continuity_with_tolerances(
    c1: &BRepAdaptorCurve,
    c2: &BRepAdaptorCurve,
    u1: f64,
    u2: f64,
    tl: f64,
    ta: f64,
) -> Result<Shape, Failure> {
    let (n1, n2) = (
        derivative_order(c1.continuity()?),
        derivative_order(c2.continuity()?),
    );
    let mut props1 = BRepLPropCLProps::with_parameter(c1, u1, n1, tl)?;
    let mut props2 = BRepLPropCLProps::with_parameter(c2, u2, n2, tl)?;
    if !props1.value().is_equal(&props2.value(), tl) {
        return Err(Failure::with_type(
            "Standard_Failure",
            &format!("Courbes non jointives: u1 = {u1}, u2 = {u2}; expected points within {tl}"),
            "",
        ));
    }
    let tolerances = Tolerances {
        linear: tl,
        angular: ta,
    };
    let cont = junction_continuity((&mut props1, &mut props2), (c1, c2), n1.min(n2), tolerances)?;
    if c1.edge().is_same(&c2.edge()) && c1.is_periodic()? && cont >= Shape::G1 {
        return Ok(Shape::CN);
    }
    Ok(cont)
}

/// The C0, C1, G1 or C2 decision on the oriented derivatives up to `min_order`.
fn junction_continuity(
    props: (&mut BRepLPropCLProps<'_>, &mut BRepLPropCLProps<'_>),
    curves: (&BRepAdaptorCurve, &BRepAdaptorCurve),
    min_order: i32,
    tolerances: Tolerances,
) -> Result<Shape, Failure> {
    let (props1, props2) = props;
    let mut cont = Shape::C0;
    if min_order < 1 {
        return Ok(cont);
    }
    let d1 = along_edge(curves.0, props1.d1()?);
    let d2 = along_edge(curves.1, props2.d1()?);
    if d1.is_equal(d2, tolerances.linear, tolerances.angular)? {
        cont = Shape::C1;
    } else {
        if props1.is_tangent_defined()? && props2.is_tangent_defined()? {
            let mut dir1 = props1.tangent()?;
            let mut dir2 = props2.tangent()?;
            if curves.0.edge().orientation() == Orientation::Reversed {
                dir1.reverse();
            }
            if curves.1.edge().orientation() == Orientation::Reversed {
                dir2.reverse();
            }
            if dir1.is_equal(&dir2, tolerances.angular) {
                cont = Shape::G1;
            }
        }
        // `fini`: the second derivatives are not compared.
        return Ok(cont);
    }
    if min_order >= 2 {
        let d1 = props1.d2()?;
        let d2 = props2.d2()?;
        if d1.is_equal(d2, tolerances.linear, tolerances.angular)? {
            cont = Shape::C2;
        }
    }
    Ok(cont)
}

/// `BRepLProp::Continuity(C1, C2, u1, u2)`: with `Precision::Confusion()` and
/// `Precision::Angular()`; `continuity(&c1, &c2, 1.0, 0.0)?`.
pub fn continuity(
    c1: &BRepAdaptorCurve,
    c2: &BRepAdaptorCurve,
    u1: f64,
    u2: f64,
) -> Result<Shape, Failure> {
    continuity_with_tolerances(c1, c2, u1, u2, precision::confusion(), precision::angular())
}
