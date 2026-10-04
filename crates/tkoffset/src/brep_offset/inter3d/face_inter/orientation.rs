//! Native OrientSection vector rule for section-edge occurrences.
use super::not_implemented;
use rustycad_tkbo::int_tools::Curve;
use rustycad_tkbrep::{brep::BRepTool, topods::Face};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::{
    geom_adaptor::GeomAdaptorCurve,
    top_abs::{Orientation, State},
};
use rustycad_tkgeombase::gcpnts::AbscissaPoint;
use rustycad_tkmath::gp::{self, Vector3};

pub(super) fn orient_section(
    curve: &Curve,
    first: &Face,
    second: &Face,
    range: (f64, f64),
) -> Result<(Orientation, Orientation), Failure> {
    let parameter = middle_length_parameter(curve, range)?;
    let tangent = section_tangent(curve, parameter)?;
    let first_normal = face_normal(first, curve.first_curve2d(), parameter)?;
    let second_normal = face_normal(second, curve.second_curve2d(), parameter)?;
    Ok(oriented_faces(
        first_normal,
        second_normal,
        tangent,
        first,
        second,
    ))
}

fn middle_length_parameter(curve: &Curve, range: (f64, f64)) -> Result<f64, Failure> {
    let adaptor = GeomAdaptorCurve::with_curve(curve.curve())?;
    let half_length = AbscissaPoint::length(&adaptor)? / 2.0;
    Ok(AbscissaPoint::from_abscissa(&adaptor, half_length, range.0)
        .and_then(|point| point.parameter())
        .unwrap_or(range.0 + (range.1 - range.0) / 2.0))
}

fn section_tangent(curve: &Curve, parameter: f64) -> Result<Vector3, Failure> {
    let curve = curve
        .curve()
        .get()
        .ok_or_else(|| not_implemented("null spatial section curve; expected a 3d curve"))?;
    let tangent = curve.eval_dn(parameter, 1)?;
    if tangent.square_magnitude() <= gp::resolution() {
        return Ok(tangent);
    }
    tangent.normalized()
}

fn face_normal(
    face: &Face,
    pcurve: &Handle<dyn Geom2dCurve>,
    parameter: f64,
) -> Result<Vector3, Failure> {
    let pcurve = pcurve
        .get()
        .ok_or_else(|| not_implemented("null face pcurve; expected a pcurve on each face"))?;
    let uv = pcurve.eval_d0(parameter)?;
    let (surface, location) = BRepTool::surface_with_location(face)?;
    let derivatives = surface
        .get()
        .ok_or_else(|| not_implemented("null face surface; expected Geom_Surface"))?
        .eval_d1(uv.x(), uv.y())?;
    let mut normal = derivatives
        .d1u
        .crossed(derivatives.d1v)
        .transformed(location.transformation());
    if face.orientation() == Orientation::Reversed {
        normal.reverse();
    }
    Ok(normal)
}

fn oriented_faces(
    first_normal: Vector3,
    second_normal: Vector3,
    tangent: Vector3,
    first: &Face,
    second: &Face,
) -> (Orientation, Orientation) {
    let first_product = first_normal.dot(second_normal.crossed(tangent));
    let second_product = second_normal.dot(first_normal.crossed(tangent));
    (
        orient_from_product(first_product, first),
        orient_from_product(second_product, second),
    )
}

fn orient_from_product(product: f64, face: &Face) -> Orientation {
    let forward = product < 0.0;
    let reverse = face.orientation() == Orientation::Reversed;
    if forward ^ reverse {
        Orientation::Forward
    } else {
        Orientation::Reversed
    }
}

pub(super) fn apply_side(orientation: Orientation, side: State) -> Orientation {
    if side != State::Out {
        return orientation;
    }
    match orientation {
        Orientation::Forward => Orientation::Reversed,
        Orientation::Reversed => Orientation::Forward,
        other => other,
    }
}
