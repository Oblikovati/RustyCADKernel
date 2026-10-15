//! `PrsDim::ComputeGeometry` for two edges: their curves and ends, on a plane or with the
//! ends of an infinite one taken from the other.
use super::PrsDim;
use super::tool_geometry::{PrsDimEnds, adaptor_curve, conic_ends, null_curve, untrimmed};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve, GeomLine, GeomPlane};
use rustycad_tkgeomalgo::geom_api::ProjectPointOnCurve;
use rustycad_tkgeombase::geom_proj_lib::project_on_plane;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::Point3;

/// The curves and ends of two edges.
#[derive(Clone, Default)]
pub struct PrsDimTwoEdges {
    /// The first curve.
    pub first_curve: Handle<dyn GeomCurve>,
    /// The second curve.
    pub second_curve: Handle<dyn GeomCurve>,
    /// The ends of the first curve.
    pub first_ends: PrsDimEnds,
    /// The ends of the second curve.
    pub second_ends: PrsDimEnds,
    /// Whether the first curve is infinite.
    pub first_infinite: bool,
    /// Whether the second curve is infinite.
    pub second_infinite: bool,
    /// The edge (1 or 2) whose curve is off the plane or infinite, 0 when none.
    pub ext_index: i32,
    /// The original curve of the edge off the plane.
    pub ext_curve: Handle<dyn GeomCurve>,
}

fn line_or_circle_ends(
    curve: &Handle<dyn GeomCurve>,
    first: f64,
    last: f64,
) -> Result<Option<PrsDimEnds>, Failure> {
    if curve.cast::<GeomLine>().get().is_some() || curve.cast::<GeomCircle>().get().is_some() {
        return conic_ends(curve, first, last);
    }
    Ok(None)
}

fn located_curve(edge: &Edge) -> Result<(Handle<dyn GeomCurve>, f64, f64), Failure> {
    let found = BRepTool::curve_with_location(edge)?;
    let Some(curve) = found.curve.get() else {
        return Ok((Handle::default(), found.first, found.last));
    };
    if found.location.is_identity() {
        return Ok((found.curve.clone(), found.first, found.last));
    }
    Ok((
        curve.transformed(found.location.transformation())?.cast(),
        found.first,
        found.last,
    ))
}

/// The ends of an infinite line taken from the projections of the other line's ends.
fn line_ends_from(
    target: &Handle<dyn GeomCurve>,
    source: &Handle<dyn GeomCurve>,
    ends: &PrsDimEnds,
) -> Option<PrsDimEnds> {
    let target = target.cast::<GeomLine>().get()?.lin();
    let source = source.cast::<GeomLine>().get()?.lin();
    let at = |p: Point3| -> Option<Point3> {
        Some(elclib::value(elclib::parameter(&source, p).ok()?, &target))
    };
    Some((at(ends.0)?, at(ends.1)?))
}

/// The ends of an infinite curve projected from the other curve's ends.
fn projected_ends(curve: &Handle<dyn GeomCurve>, ends: &PrsDimEnds) -> Result<PrsDimEnds, Failure> {
    let value = curve.get().ok_or_else(null_curve)?;
    let mut projector = ProjectPointOnCurve::with_curve(ends.0, curve)?;
    let first = value.value(projector.lower_distance_parameter()?)?;
    projector.init(ends.1, curve)?;
    Ok((first, value.value(projector.lower_distance_parameter()?)?))
}

fn same_line_type(first: &Handle<dyn GeomCurve>, second: &Handle<dyn GeomCurve>) -> bool {
    match (first.get(), second.get()) {
        (Some(a), Some(b)) => {
            std::ptr::eq(a.dynamic_type(), b.dynamic_type())
                && first.is_instance(GeomLine::static_type())
        }
        _ => false,
    }
}

impl PrsDim {
    /// `ComputeGeometry(edge1, edge2, ..., plane)`: the curves of two edges projected on a
    /// plane (lines or circles only) with their ends.
    /// Example: `PrsDim::compute_two_edges_on_plane(&e1, &e2, &plane)?`.
    pub fn compute_two_edges_on_plane(
        first: &Edge,
        second: &Edge,
        plane: &Handle<GeomPlane>,
    ) -> Result<Option<PrsDimTwoEdges>, Failure> {
        let Some(geom_plane) = plane.get() else {
            return Ok(None);
        };
        let (curve1, first1, last1) = located_curve(first)?;
        let (curve2, first2, last2) = located_curve(second)?;
        if curve1.is_null() || curve2.is_null() {
            return Ok(None);
        }
        let normal = geom_plane.pln().axis().direction();
        let curve1 = untrimmed(project_on_plane(&curve1, plane, normal, false)?);
        let curve2 = untrimmed(project_on_plane(&curve2, plane, normal, false)?);
        let Some(first_ends) = line_or_circle_ends(&curve1, first1, last1)? else {
            return Ok(None);
        };
        let Some(second_ends) = line_or_circle_ends(&curve2, first2, last2)? else {
            return Ok(None);
        };
        Ok(Some(PrsDimTwoEdges {
            first_curve: curve1,
            second_curve: curve2,
            first_ends,
            second_ends,
            ..PrsDimTwoEdges::default()
        }))
    }

    /// `ComputeGeometry(edge1, edge2, ..., isInfinite1, isInfinite2)`: the curves of two
    /// edges, an infinite one getting the ends of its projection from the other.
    /// Example: `PrsDim::compute_two_edges(&e1, &e2)?`.
    pub fn compute_two_edges(
        first: &Edge,
        second: &Edge,
    ) -> Result<Option<PrsDimTwoEdges>, Failure> {
        let Some((curve1, ends1, infinite1)) = Self::compute_edge_geometry_infinite(first)? else {
            return Ok(None);
        };
        let Some((curve2, ends2, infinite2)) = Self::compute_edge_geometry_infinite(second)? else {
            return Ok(None);
        };
        let mut result = PrsDimTwoEdges {
            first_curve: curve1,
            second_curve: curve2,
            first_ends: ends1,
            second_ends: ends2,
            first_infinite: infinite1,
            second_infinite: infinite2,
            ..PrsDimTwoEdges::default()
        };
        if !infinite1 && !infinite2 {
            return Ok(Some(result));
        }
        if same_line_type(&result.first_curve, &result.second_curve) {
            if infinite1 {
                result.first_ends = line_ends_from(
                    &result.first_curve,
                    &result.second_curve,
                    &result.second_ends,
                )
                .ok_or_else(null_curve)?;
            } else {
                result.second_ends = line_ends_from(
                    &result.second_curve,
                    &result.first_curve,
                    &result.first_ends,
                )
                .ok_or_else(null_curve)?;
            }
            return Ok(Some(result));
        }
        if infinite1 && !infinite2 {
            result.first_ends = projected_ends(&result.first_curve, &result.second_ends)?;
        } else if !infinite1 && infinite2 {
            result.second_ends = projected_ends(&result.second_curve, &result.first_ends)?;
        } else {
            return Ok(None);
        }
        Ok(Some(result))
    }

    /// `ComputeGeometry(edge1, edge2, extIndex, ..., extCurve, ..., plane)`: the curves of
    /// two edges on a plane, with the original curve of the one off the plane. `None` for a
    /// null plane (the outputs untouched), else the native result with the outputs as the
    /// native code leaves them (the extension index is set even when it fails).
    /// Example: `PrsDim::compute_two_edges_ext(&e1, &e2, &plane)?`.
    pub fn compute_two_edges_ext(
        first: &Edge,
        second: &Edge,
        plane: &Handle<GeomPlane>,
    ) -> Result<Option<(bool, PrsDimTwoEdges)>, Failure> {
        if plane.is_null() {
            return Ok(None);
        }
        let adaptor1 = BRepAdaptorCurve::with_edge(first)?;
        let adaptor2 = BRepAdaptorCurve::with_edge(second)?;
        let mut curve1 = untrimmed(adaptor_curve(&adaptor1)?);
        let mut curve2 = untrimmed(adaptor_curve(&adaptor2)?);
        let (first1, last1) = (adaptor1.first_parameter()?, adaptor1.last_parameter()?);
        let (first2, last2) = (adaptor2.first_parameter()?, adaptor2.last_parameter()?);
        let mut result = PrsDimTwoEdges::default();
        if curve1.is_null() || curve2.is_null() {
            return Ok(Some((false, result)));
        }
        let (saved1, saved2) = (curve1.clone(), curve2.clone());
        let on_plane1 =
            Self::compute_geom_curve(&mut curve1, first1, last1, &mut result.first_ends, plane)?;
        let on_plane2 =
            Self::compute_geom_curve(&mut curve2, first2, last2, &mut result.second_ends, plane)?;
        result.first_curve = curve1;
        result.second_curve = curve2;
        if precision::is_infinite(first1) || precision::is_infinite(last1) {
            result.first_infinite = true;
            result.ext_index = 1;
        }
        if precision::is_infinite(first2) || precision::is_infinite(last2) {
            result.second_infinite = true;
            result.ext_index = 2;
        }
        if result.first_infinite && result.second_infinite {
            result.ext_index = 0;
        }
        if (result.first_infinite || result.second_infinite)
            && same_line_type(&result.first_curve, &result.second_curve)
        {
            if result.ext_index == 1 {
                result.first_ends = line_ends_from(
                    &result.first_curve,
                    &result.second_curve,
                    &result.second_ends,
                )
                .ok_or_else(null_curve)?;
            } else if result.ext_index == 2 {
                result.second_ends = line_ends_from(
                    &result.second_curve,
                    &result.first_curve,
                    &result.first_ends,
                )
                .ok_or_else(null_curve)?;
            }
        }
        match (on_plane1, on_plane2) {
            (true, true) => {}
            (false, true) => {
                result.ext_index = 1;
                result.ext_curve = saved1;
            }
            (true, false) => {
                result.ext_index = 2;
                result.ext_curve = saved2;
            }
            (false, false) => return Ok(Some((false, result))),
        }
        Ok(Some((true, result)))
    }
}
