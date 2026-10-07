// Rust translation of BRepFill_LocationLaw::PerformVertex, CurvilinearBounds, Parameter,
// D0 and Abscissa (ModelingAlgorithms/TKBool/BRepFill/BRepFill_LocationLaw.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The positions along a location law: vertices of the sweep, abscissas and sections.
use super::BRepFillLocationLawBase;
use super::continuity::is_g1;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{Shape, Vertex};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkgeomalgo::geom_fill::{GeomFillLocationLaw, LocationD0};
use rustycad_tkgeombase::gcpnts::AbscissaPoint;
use rustycad_tkmath::gp::{Point3, Transform3, Xyz};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeShape, BRepBuilderAPITransform};

/// The placement of law `index` at the start (`at_end` false) or end of its domain,
/// written over `placement`.
fn placement_at(
    base: &BRepFillLocationLawBase,
    index: i32,
    at_end: bool,
    placement: &mut LocationD0,
) -> Result<(), Failure> {
    let law = base.law_at(index)?;
    let (first, last) = law.get_domain()?;
    law.d0(if at_end { last } else { first }, placement)?;
    Ok(())
}

/// The placements around the vertex `index` of the path for `PerformVertex`, and
/// whether the vertex is the middle of both.
fn vertex_placements(
    base: &BRepFillLocationLawBase,
    index: i32,
    tol_min: f64,
    iloc: i32,
) -> Result<(LocationD0, LocationD0, bool), Failure> {
    let n: i32 = base.count()?;
    let (mut m1, mut m2) = (LocationD0::default(), LocationD0::default());
    let mut is_bary: bool = iloc == 0;
    let (before, after) = if index > 0 && index < n {
        (index, index + 1)
    } else {
        (n, 1)
    };
    let interior: bool = index > 0 && index < n;
    let end: bool = index == 0 || index == n;
    if end && (!base.path.closed()? || is_g1(base, index, tol_min, 1.0e-4)? != 1) {
        is_bary = false;
        if index == 0 {
            placement_at(base, 1, false, &mut m1)?;
        } else {
            placement_at(base, n, true, &mut m1)?;
        }
        return Ok((m1, m2, is_bary));
    }
    if interior || end {
        if iloc <= 0 {
            placement_at(base, before, true, &mut m1)?;
        }
        if iloc >= 0 {
            let target: &mut LocationD0 = if iloc == 0 { &mut m2 } else { &mut m1 };
            placement_at(base, after, false, target)?;
        }
    }
    Ok((m1, m2, is_bary))
}

/// `M * P + V`.
fn placed(xyz: Xyz, placement: &LocationD0) -> Xyz {
    let mut p: Xyz = xyz;
    p *= placement.m;
    p += *placement.v.xyz();
    p
}

/// `PerformVertex(Index, Input, TolMin, Output, ILoc)`.
pub(super) fn perform_vertex(
    base: &BRepFillLocationLawBase,
    index: i32,
    input: &Vertex,
    tol_min: f64,
    iloc: i32,
) -> Result<Vertex, Failure> {
    let (m1, m2, is_bary) = vertex_placements(base, index, tol_min, iloc)?;
    let p: Point3 = BRepTool::pnt(input)?;
    let builder = BRepBuilder::default();
    let mut output = Vertex::default();
    if is_bary {
        let mut p1: Xyz = placed(p.xyz(), &m1);
        let p2: Xyz = placed(p.xyz(), &m2);
        let mut middle = Xyz::default();
        middle.set_linear_form(0.5, p1, 0.5, p2);
        p1 -= p2;
        let tol: f64 = p1.modulus() / 2. + tol_min;
        builder.make_vertex_at(&mut output, Point3::from(middle), tol)?;
    } else {
        builder.make_vertex_at(&mut output, Point3::from(placed(p.xyz(), &m1)), tol_min)?;
    }
    Ok(output)
}

/// The cumulated lengths of the laws (`myLength(ii + 1)`), computed once.
fn compute_lengths(base: &BRepFillLocationLawBase) -> Result<(), Failure> {
    let mut length: f64 = 0.;
    for ii in 1..=base.edges.len() as i32 {
        let law = base.law_at(ii)?;
        law.get_domain()?;
        length += AbscissaPoint::length_with_tolerance(adaptor(&live_curve(law)?)?, base.tol)?;
        base.length.write()[ii as usize] = length;
    }
    Ok(())
}

/// The curve of a law (`GetCurve()`); Standard_NullObject without one.
fn live_curve(law: &dyn GeomFillLocationLaw) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
    let curve = law.get_curve();
    if curve.is_null() {
        return Err(Failure::with_type(
            "Standard_NullObject",
            "BRepFill_LocationLaw: a law without curve; expected a curve",
            "BRepFill_LocationLaw",
        ));
    }
    Ok(curve)
}

/// The adaptor of a live curve handle.
fn adaptor(curve: &Handle<dyn Adaptor3dCurve>) -> Result<&dyn Adaptor3dCurve, Failure> {
    curve.get().ok_or_else(|| {
        Failure::with_type(
            "Standard_NullObject",
            "BRepFill_LocationLaw: null curve; expected a curve",
            "BRepFill_LocationLaw",
        )
    })
}

/// `myLength(i)` (1-based).
fn length_at(base: &BRepFillLocationLawBase, i: i32) -> Result<f64, Failure> {
    let lengths = base.length.read();
    usize::try_from(i - 1)
        .ok()
        .and_then(|k| lengths.get(k))
        .copied()
        .ok_or_else(|| super::index_error("BRepFill_LocationLaw myLength", i, lengths.len()))
}

/// `CurvilinearBounds(Index, First, Last)`.
pub(super) fn curvilinear_bounds(
    base: &BRepFillLocationLawBase,
    index: i32,
) -> Result<(f64, f64), Failure> {
    if length_at(base, index + 1)? < 0. {
        // The calculation is required.
        compute_lengths(base)?;
    }
    Ok((length_at(base, index)?, length_at(base, index + 1)?))
}

/// `Parameter(Abscissa, Index, U)`: `(0, 0)` outside the path.
pub(super) fn parameter(
    base: &BRepFillLocationLawBase,
    abscissa: f64,
) -> Result<(i32, f64), Failure> {
    let nb_e: i32 = base.edges.len() as i32;
    if length_at(base, nb_e + 1)? < 0. {
        curvilinear_bounds(base, nb_e)?;
    }
    let mut iedge: i32 = 1;
    while iedge <= nb_e && length_at(base, iedge + 1)? < abscissa {
        iedge += 1;
    }
    if iedge > nb_e {
        return Ok((0, 0.));
    }
    let law = base.law_at(iedge)?;
    let (f, l) = law.get_domain()?;
    if abscissa == length_at(base, iedge + 1)? {
        return Ok((iedge, l));
    }
    if abscissa == length_at(base, iedge)? {
        return Ok((iedge, f));
    }
    let curve = live_curve(law)?;
    let point = AbscissaPoint::with_tolerance(
        base.tol,
        adaptor(&curve)?,
        abscissa - length_at(base, iedge)?,
        f,
    )?;
    Ok((iedge, point.parameter()?))
}

/// `D0(Abscissa, W)`: the section transformed (copied) by the placement at the
/// abscissa; a null shape outside the path.
pub(super) fn d0(
    base: &BRepFillLocationLawBase,
    abscissa: f64,
    section: &Shape,
) -> Result<Shape, Failure> {
    let (ind, u) = parameter(base, abscissa)?;
    if ind == 0 {
        return Ok(Shape::default());
    }
    let mut placement = LocationD0::default();
    base.law_at(ind)?.d0(u, &mut placement)?;
    let (m, v) = (placement.m, placement.v);
    let mut fila = Transform3::default();
    fila.set_values([
        [m.value(1, 1)?, m.value(1, 2)?, m.value(1, 3)?, v.x()],
        [m.value(2, 1)?, m.value(2, 2)?, m.value(2, 3)?, v.y()],
        [m.value(3, 1)?, m.value(3, 2)?, m.value(3, 3)?, v.z()],
    ])?;
    let mut transform = BRepBuilderAPITransform::with_shape(section, fila, true, false)?;
    Ok(transform.shape()?.clone())
}

/// `Abscissa(Index, Param)`: from `myLength(Index)`, or, when it is not computed yet,
/// from the end of edge `index` (CurvilinearBounds writes its Last there natively),
/// plus the length from the start of the curve to `param`.
pub(super) fn abscissa(
    base: &BRepFillLocationLawBase,
    index: i32,
    param: f64,
) -> Result<f64, Failure> {
    let mut length: f64 = length_at(base, index)?;
    if length < 0. {
        length = curvilinear_bounds(base, index)?.1;
    }
    let law = base.law_at(index)?;
    let curve = live_curve(law)?;
    let first: f64 = adaptor(&curve)?.first_parameter()?;
    Ok(length
        + AbscissaPoint::length_between_with_tolerance(adaptor(&curve)?, first, param, base.tol)?)
}
