// Rust translation of the static BRepFill_Confusion, EdgeVertices, IsVertical, IsPlanar, Side,
// IsInversed, ConcaveSide, Bubble, Compare, Relative, PosOnFace, DoubleOrNotInFace,
// DistanceToOZ, Altitud and SimpleExpression of
// ModelingAlgorithms/TKBool/BRepFill/BRepFill_Evolved.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The geometric predicates of the evolved shape: the position of the profile parts
//! relative to the axis Oz, the tangents at the spine and the bisector curves.
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex, Wire};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::Vector3;
use rustycad_tktopalgo::bisector::{Bisec, BisecAna};
use rustycad_tktopalgo::brep_lib::BRepLibMakeWire;

/// `BRepFill_Confusion()` (static): 1e-6.
pub fn confusion() -> f64 {
    1.0e-6
}

/// `EdgeVertices(E, V1, V2)` (static): the vertices of the edge in its orientation.
pub fn edge_vertices(e: &Edge) -> (Vertex, Vertex) {
    let (v1, v2) = topexp::vertices(e, false);
    if e.as_ref().orientation() == Orientation::Reversed {
        (v2, v1)
    } else {
        (v1, v2)
    }
}

/// The edge is a line with both ends at the same coordinate `coord` (Y or Z).
fn is_line_at_constant(
    e: &Edge,
    coord: fn(&rustycad_tkmath::gp::Point3) -> f64,
) -> Result<bool, Failure> {
    let (v1, v2) = topexp::vertices(e, false);
    let (p1, p2) = (BRepTool::pnt(&v1)?, BRepTool::pnt(&v2)?);
    if (coord(&p1) - coord(&p2)).abs() >= confusion() {
        return Ok(false);
    }
    Ok(BRepTool::curve_with_location(e)?
        .curve
        .is_instance_name("Geom_Line"))
}

/// `IsVertical(E)` (static): a line at constant Y.
pub fn is_vertical(e: &Edge) -> Result<bool, Failure> {
    is_line_at_constant(e, |p| p.y())
}

/// `IsPlanar(E)` (static): a line at constant Z.
pub fn is_planar(e: &Edge) -> Result<bool, Failure> {
    is_line_at_constant(e, |p| p.z())
}

/// `Side(Profil, Tol)` (static): 1 left, 4 right (a first vertex below -Tol in Y),
/// plus 1 when planar or 2 when vertical, from the first edge of the profile.
pub fn side(profil: &Wire, tol: f64) -> Result<i32, Failure> {
    let first = Explorer::new(profil.as_ref(), ShapeEnum::Edge, ShapeEnum::Shape)?
        .take(1)
        .collect::<Vec<Shape>>()
        .pop()
        .ok_or_else(|| {
            Failure::with_type(
                "Standard_NoSuchObject",
                "BRepFill_Evolved Side: the profile has no edge; expected one",
                "BRepFill_Evolved",
            )
        })?;
    let e = topods::edge(&first)?;
    let (v1, v2) = topexp::vertices(&e, false);
    let (p1, p2) = (BRepTool::pnt(&v1)?, BRepTool::pnt(&v2)?);
    let mut the_side = if p1.y() < -tol || p2.y() < -tol { 4 } else { 1 };
    if is_vertical(&e)? {
        the_side += 2;
    } else if is_planar(&e)? {
        the_side += 1;
    }
    Ok(the_side)
}

/// The tangent of an edge at its start in its orientation.
fn start_tangent(e: &Edge) -> Result<Vector3, Failure> {
    let c = BRepAdaptorCurve::with_edge(e)?;
    if e.as_ref().orientation() == Orientation::Forward {
        return Ok(c.d1(c.first_parameter()?)?.1);
    }
    Ok(c.d1(c.last_parameter()?)?.1.reversed())
}

/// `IsInversed(S, E1, E2, Inverse)` (static): for an edge `s`, whether `e1` and `e2`
/// start against its start tangent (a degenerated one always does).
pub fn is_inversed(s: &Shape, e1: &Edge, e2: &Edge) -> Result<[bool; 2], Failure> {
    if s.shape_type()? != ShapeEnum::Edge {
        return Ok([false, false]);
    }
    let ds = start_tangent(&topods::edge(s)?)?;
    let inv = |e: &Edge| -> Result<bool, Failure> {
        if BRepTool::degenerated(e)? {
            return Ok(true);
        }
        Ok(ds.dot(start_tangent(e)?) < 0.)
    };
    Ok([inv(e1)?, inv(e2)?])
}

/// `ConcaveSide(S, F)` (static): an edge on a circle of the face turning counter
/// clockwise in its orientation (done only for circles, as natively).
pub fn concave_side(s: &Shape, f: &Face) -> Result<bool, Failure> {
    if s.shape_type()? != ShapeEnum::Edge {
        return Ok(false);
    }
    let pc = BRepTool::curve_on_face(&topods::edge(s)?, f)?;
    let ac = Geom2dAdaptorCurve::with_range(&pc.pcurve, pc.first, pc.last)?;
    if ac.get_type()? != CurveType::Circle {
        return Ok(false);
    }
    let direct = ac.circle()?.is_direct();
    Ok(if s.orientation() == Orientation::Reversed {
        !direct
    } else {
        direct
    })
}

/// `Bubble(Seq)` (static): the reals bubble sorted.
pub fn bubble_reals(seq: &mut [f64]) {
    let mut invert = true;
    while invert {
        invert = false;
        for i in 0..seq.len().saturating_sub(1) {
            if seq[i + 1] < seq[i] {
                seq.swap(i, i + 1);
                invert = true;
            }
        }
    }
}

/// `Compare(E1, E2)` (static): REVERSED when the first point of `e1` is nearer the last
/// point of `e2` than its first.
pub fn compare(e1: &Edge, e2: &Edge) -> Result<Orientation, Failure> {
    let (v10, _) = topexp::vertices(e1, false);
    let (v20, v21) = topexp::vertices(e2, false);
    let (p1, p2, p3) = (
        BRepTool::pnt(&v10)?,
        BRepTool::pnt(&v20)?,
        BRepTool::pnt(&v21)?,
    );
    Ok(if p1.distance(&p3) < p1.distance(&p2) {
        Orientation::Reversed
    } else {
        Orientation::Forward
    })
}

/// The first edge of `w` with `v` as a vertex.
fn edge_at(w: &Wire, v: &Vertex) -> Result<Option<Edge>, Failure> {
    for s in Explorer::new(w.as_ref(), ShapeEnum::Edge, ShapeEnum::Shape)? {
        let e = topods::edge(&s)?;
        let (v1, v2) = topexp::vertices(&e, false);
        if v1.as_ref().is_same(v.as_ref()) || v2.as_ref().is_same(v.as_ref()) {
            return Ok(Some(e));
        }
    }
    Ok(None)
}

/// `Relative(W1, W2, V, Commun)` (static): whether both wires have an edge at `v`
/// (`commun`), and FORWARD when those edges are on the same side of Oz (both strictly
/// left, or both beyond the right side code 4), else REVERSED.
pub fn relative(w1: &Wire, w2: &Wire, v: &Vertex) -> Result<(Orientation, bool), Failure> {
    let (Some(e1), Some(e2)) = (edge_at(w1, v)?, edge_at(w2, v)?) else {
        return Ok((Orientation::Forward, false));
    };
    let ww1 = BRepLibMakeWire::from_edge(&e1)?.wire()?;
    let ww2 = BRepLibMakeWire::from_edge(&e2)?.wire()?;
    let tol = confusion();
    let (s1, s2) = (side(&ww1, tol)?, side(&ww2, tol)?);
    if (s1 < 4 && s2 < 4) || (s1 > 4 && s2 > 4) {
        return Ok((Orientation::Forward, true));
    }
    Ok((Orientation::Reversed, true))
}

/// `PosOnFace(d1, d2, d3)` (static): 1 on the d2 edge, 3 on the d3 edge, 2 strictly
/// inside, 0 out (within the confusion).
pub fn pos_on_face(d1: f64, d2: f64, d3: f64) -> i32 {
    let c = confusion();
    if (d1 - d2).abs() <= c {
        return 1;
    }
    if (d1 - d3).abs() <= c {
        return 3;
    }
    let (lo, hi) = if d2 < d3 { (d2, d3) } else { (d3, d2) };
    if d1 > lo + c && d1 < hi - c { 2 } else { 0 }
}

/// `DoubleOrNotInFace(EC, V)` (static): `v` is a vertex of no edge of `ec`, or of two
/// ends.
pub fn double_or_not_in_face(ec: &[Shape], v: &Vertex) -> Result<bool, Failure> {
    let mut vu = false;
    for s in ec {
        let (v1, v2) = topexp::vertices(&topods::edge(s)?, false);
        for end in [v1, v2] {
            if end.as_ref().is_same(v.as_ref()) {
                if vu {
                    return Ok(true);
                }
                vu = true;
            }
        }
    }
    Ok(!vu)
}

/// `DistanceToOZ(V)` (static): |Y| of the vertex point.
pub fn distance_to_oz(v: &Vertex) -> Result<f64, Failure> {
    Ok(BRepTool::pnt(v)?.y().abs())
}

/// `Altitud(V)` (static): Z of the vertex point.
pub fn altitud(v: &Vertex) -> Result<f64, Failure> {
    Ok(BRepTool::pnt(v)?.z())
}

/// `SimpleExpression(B, Bis)` (static): the bisector curve, its analytic basis
/// trimmed in place of a trimmed Bisector_BisecAna.
pub fn simple_expression(b: &Bisec) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let value = b.value();
    let bis: Handle<dyn Geom2dCurve> = value.clone().cast();
    let Some(tr_bis) = value.get() else {
        return Ok(bis);
    };
    let bas_bis = tr_bis.basis_curve();
    let ana: Handle<BisecAna> = bas_bis.cast();
    let Some(ana) = ana.get() else {
        return Ok(bis);
    };
    let curve = ana.geom2d_curve();
    Ok(Handle::new(Geom2dTrimmedCurve::new(
        &curve,
        tr_bis.first_parameter(),
        tr_bis.last_parameter(),
        true,
        true,
    )?)
    .cast())
}
