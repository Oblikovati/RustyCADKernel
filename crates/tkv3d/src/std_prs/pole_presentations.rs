//! The control polygon presentations (`StdPrs_PoleCurve`, `StdPrs_WFPoleSurface`), the
//! coarse shading of an adaptor surface (`StdPrs_ShadedSurface`) and `StdPrs_ToolVertex`.
use super::curve_draw::draw_arrow;
use crate::prs3d::Prs3dDrawer;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::Vertex;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::real::real_last;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkmath::geom_abs::{CurveType, Shape as Continuity, SurfaceType};
use rustycad_tkmath::gp::{Direction3, Point3, Vector3};
use rustycad_tkservice::graphic3d::*;

/// The poles of a Bezier or BSpline curve adaptor, None for another curve type.
fn curve_poles(curve: &dyn Adaptor3dCurve) -> Result<Option<Vec<Point3>>, Failure> {
    Ok(match curve.get_type()? {
        CurveType::BezierCurve => {
            let bezier = curve.bezier()?;
            Some(
                (1..=bezier.nb_poles())
                    .map(|i| bezier.pole(i))
                    .collect::<Result<_, _>>()?,
            )
        }
        CurveType::BSplineCurve => {
            let bspline = curve.bspline()?;
            Some(
                (1..=bspline.nb_poles())
                    .map(|i| bspline.pole(i))
                    .collect::<Result<_, _>>()?,
            )
        }
        _ => None,
    })
}

/// The Manhattan distance of a pole to a point (the native pick metric).
fn manhattan(point: &Point3, pole: &Point3) -> f64 {
    (point.x() - pole.x()).abs() + (point.y() - pole.y()).abs() + (point.z() - pole.z()).abs()
}

/// `StdPrs_PoleCurve`: the control polygon of a Bezier or BSpline curve.
/// Example: `StdPrsPoleCurve::add(&prs, &curve, &drawer)?`.
pub struct StdPrsPoleCurve;

impl StdPrsPoleCurve {
    /// `Add`: the polyline of the poles with the line aspect, then the arrow at the last
    /// parameter when the drawer asks for it (for any curve type).
    /// Example: `StdPrsPoleCurve::add(&prs, &curve, &drawer)?`.
    pub fn add(
        prs: &Handle<dyn Graphic3dStructureSource>,
        curve: &dyn Adaptor3dCurve,
        drawer: &Handle<Prs3dDrawer>,
    ) -> Result<(), Failure> {
        let group = prs.current_group()?;
        group.set_primitives_aspect(&drawer.line_aspect().aspect().cast());
        if let Some(poles) = curve_poles(curve)? {
            let array = Handle::new(Graphic3dArrayOfPrimitives::new(
                Graphic3dTypeOfPrimitiveArray::Polylines,
                poles.len() as i32,
                0,
                0,
                0,
            )?);
            for pole in &poles {
                array.add_vertex_pnt(pole)?;
            }
            group.add_primitive_array(&array, true);
        }
        draw_arrow(curve, &prs.current_group()?, drawer)
    }

    /// `Match`: whether a pole lies within `distance` of the point (Manhattan metric).
    /// Example: `StdPrsPoleCurve::match_point(x, y, z, 0.1, &curve, &drawer)?`.
    pub fn match_point(
        x: f64,
        y: f64,
        z: f64,
        distance: f64,
        curve: &dyn Adaptor3dCurve,
        _drawer: &Handle<Prs3dDrawer>,
    ) -> Result<bool, Failure> {
        let point = Point3::new(x, y, z);
        Ok(curve_poles(curve)?
            .is_some_and(|poles| poles.iter().any(|pole| manhattan(&point, pole) <= distance)))
    }

    /// `Pick`: the one-based index of the nearest pole within `distance`, 0 without.
    /// Example: `StdPrsPoleCurve::pick(x, y, z, 0.1, &curve, &drawer)?`.
    pub fn pick(
        x: f64,
        y: f64,
        z: f64,
        distance: f64,
        curve: &dyn Adaptor3dCurve,
        _drawer: &Handle<Prs3dDrawer>,
    ) -> Result<i32, Failure> {
        let point = Point3::new(x, y, z);
        let (mut nearest, mut index) = (real_last(), 0);
        for (i, pole) in curve_poles(curve)?.unwrap_or_default().iter().enumerate() {
            let dist = manhattan(&point, pole);
            if dist <= distance && dist < nearest {
                (nearest, index) = (dist, i as i32 + 1);
            }
        }
        Ok(index)
    }
}

/// `StdPrs_WFPoleSurface`: the control net of a Bezier or BSpline surface.
/// Example: `StdPrsWfPoleSurface::add(&prs, &surface, &drawer)?`.
pub struct StdPrsWfPoleSurface;

impl StdPrsWfPoleSurface {
    /// `Add`: the pole rows with the U isoline aspect, then the pole columns with the V one;
    /// nothing for another surface type. Example: `StdPrsWfPoleSurface::add(&prs, &surface, &drawer)?`.
    pub fn add(
        prs: &Handle<dyn Graphic3dStructureSource>,
        surface: &dyn Adaptor3dSurface,
        drawer: &Handle<Prs3dDrawer>,
    ) -> Result<(), Failure> {
        let poles = match surface.get_type()? {
            SurfaceType::BezierSurface => surface.bezier()?.poles(),
            SurfaceType::BSplineSurface => surface.bspline()?.poles(),
            _ => return Ok(()),
        };
        add_poles(prs, &poles, drawer)
    }
}

/// `AddPoles`: the rows, then the columns of the net, each array in its own aspect.
fn add_poles(
    prs: &Handle<dyn Graphic3dStructureSource>,
    poles: &Array2<'static, Point3>,
    drawer: &Handle<Prs3dDrawer>,
) -> Result<(), Failure> {
    let (n, m) = (poles.col_length(), poles.row_length());
    let (row0, col0) = (poles.lower_row(), poles.lower_col());
    let pole = |i: i32, j: i32| -> Result<Point3, Failure> {
        poles
            .value(row0 + i - 1, col0 + j - 1)
            .copied()
            .map_err(|error| Failure::with_message("Standard_OutOfRange", &format!("{error:?}")))
    };
    let group = prs.current_group()?;
    group.set_primitives_aspect(&drawer.u_iso_aspect().aspect().cast());
    let rows = Handle::new(Graphic3dArrayOfPrimitives::new(
        Graphic3dTypeOfPrimitiveArray::Polylines,
        n * m,
        n,
        0,
        0,
    )?);
    for i in 1..=n {
        rows.add_bound(m)?;
        for j in 1..=m {
            rows.add_vertex_pnt(&pole(i, j)?)?;
        }
    }
    group.add_primitive_array(&rows, true);
    let group = prs.current_group()?;
    group.set_primitives_aspect(&drawer.v_iso_aspect().aspect().cast());
    let columns = Handle::new(Graphic3dArrayOfPrimitives::new(
        Graphic3dTypeOfPrimitiveArray::Polylines,
        n * m,
        m,
        0,
        0,
    )?);
    for j in 1..=m {
        columns.add_bound(n)?;
        for i in 1..=n {
            columns.add_vertex_pnt(&pole(i, j)?)?;
        }
    }
    group.add_primitive_array(&columns, true);
    Ok(())
}

/// `StdPrs_ShadedSurface`: a coarse shading of an adaptor surface, one triangle strip
/// array per C1 interval patch. Example: `StdPrsShadedSurface::add(&prs, &surface, &drawer)?`.
pub struct StdPrsShadedSurface;

impl StdPrsShadedSurface {
    /// `Add`: at least 3 x 3 cells per patch (the isoline numbers), infinite bounds clamped
    /// to the maximal parameter value. Native quirks kept: each strip bound counts `N2 + 1`
    /// vertices although `2 (N2 + 1)` are added, and the first vertex of every pair is the
    /// origin (the native `P1` is never assigned) with the normal of the current row.
    /// Example: `StdPrsShadedSurface::add(&prs, &surface, &drawer)?`.
    pub fn add(
        prs: &Handle<dyn Graphic3dStructureSource>,
        surface: &dyn Adaptor3dSurface,
        drawer: &Handle<Prs3dDrawer>,
    ) -> Result<(), Failure> {
        let n1 = drawer.u_iso_aspect().number().max(3);
        let n2 = drawer.v_iso_aspect().number().max(3);
        let group = prs.current_group()?;
        group.set_group_primitives_aspect(&drawer.shading_aspect().aspect().cast());
        group.set_closed(surface.is_u_closed()? && surface.is_v_closed()?);
        let u_count = surface.nb_u_intervals(Continuity::C1)?;
        let v_count = surface.nb_v_intervals(Continuity::C1)?;
        let mut intervals_u = Array1::with_bounds(1, u_count + 1)
            .map_err(|e| e.into_failure("NCollection_Array1"))?;
        let mut intervals_v = Array1::with_bounds(1, v_count + 1)
            .map_err(|e| e.into_failure("NCollection_Array1"))?;
        surface.u_intervals(&mut intervals_u, Continuity::C1)?;
        surface.v_intervals(&mut intervals_v, Continuity::C1)?;
        let inter_u: Vec<f64> = (1..=u_count + 1).map(|i| intervals_u[i]).collect();
        let inter_v: Vec<f64> = (1..=v_count + 1).map(|i| intervals_v[i]).collect();
        let limit = drawer.maximal_parameter_value();
        let p1 = Point3::default();
        let normal = |d1u: Vector3, d1v: Vector3| -> Result<Direction3, Failure> {
            Direction3::from_vector(d1u.crossed(d1v).normalized()?)
        };
        for nu in 0..u_count as usize {
            for nv in 0..v_count as usize {
                let clamp_low = |value: f64| {
                    if precision::is_negative_infinite(value) {
                        -limit
                    } else {
                        value
                    }
                };
                let clamp_high = |value: f64| {
                    if precision::is_positive_infinite(value) {
                        limit
                    } else {
                        value
                    }
                };
                let (u1, u2) = (clamp_low(inter_u[nu]), clamp_high(inter_u[nu + 1]));
                let (v1, v2) = (clamp_low(inter_v[nv]), clamp_high(inter_v[nv + 1]));
                let du = (u2 - u1) / f64::from(n1);
                let dv = (v2 - v1) / f64::from(n2);
                let array = Handle::new(Graphic3dArrayOfPrimitives::new(
                    Graphic3dTypeOfPrimitiveArray::Trianglestrips,
                    2 * (n1 + 1) * (n2 + 1),
                    n1 + 1,
                    0,
                    GRAPHIC3D_ARRAY_FLAGS_VERTEX_NORMAL,
                )?);
                for i in 1..=n1 + 1 {
                    array.add_bound(n2 + 1)?;
                    for j in 1..=n2 + 1 {
                        let v = v1 + dv * f64::from(j - 1);
                        let first = surface.d1(u1 + du * f64::from(i - 1), v)?;
                        let d1 = normal(first.d1u, first.d1v)?;
                        let second = surface.d1(u1 + du * f64::from(i), v)?;
                        let d2 = normal(second.d1u, second.d1v)?;
                        array.add_vertex_pnt_normal(&p1, &d1)?;
                        array.add_vertex_pnt_normal(&second.point, &d2)?;
                    }
                }
                group.add_primitive_array(&array, true);
            }
        }
        Ok(())
    }
}

/// `StdPrs_ToolVertex`: the coordinates of a vertex.
/// Example: `let (x, y, z) = StdPrsToolVertex::coord(&vertex)?;`.
pub struct StdPrsToolVertex;

impl StdPrsToolVertex {
    /// `Coord`: the point of the vertex; `StdPrsToolVertex::coord(&vertex)?`.
    pub fn coord(vertex: &Vertex) -> Result<(f64, f64, f64), Failure> {
        let point = BRepTool::pnt(vertex)?;
        Ok((point.x(), point.y(), point.z()))
    }
}
