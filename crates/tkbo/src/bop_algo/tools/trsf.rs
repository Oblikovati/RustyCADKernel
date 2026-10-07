//! `BOPAlgo_Tools::TrsfToPoint`: the translation moving far shapes next to a point.
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{Point3, Transform3, Vector3};

/// Native default criteria: the shapes must be farther than 1e5 from the point and their size
/// below 1e-5 of that distance.
pub const TRSF_TO_POINT_CRITERIA: f64 = 1.0e5;

/// Returns the translation from the union box's minimal corner to `point` when the union is
/// far from the point relative to its size, else `None`.
/// `let trsf = trsf_to_point(&b1, &b2, Point3::default(), TRSF_TO_POINT_CRITERIA)?;`.
pub fn trsf_to_point(
    first: &BoundingBox3,
    second: &BoundingBox3,
    point: Point3,
    criteria: f64,
) -> Result<Option<Transform3>, Failure> {
    let mut union = first.clone();
    union.add_box(second);
    let minimum = union.corner_min()?;
    let maximum = union.corner_max()?;
    let center = (minimum.xyz() + maximum.xyz()) / 2.;
    let distance = (point.xyz() - center).modulus();
    if distance < criteria {
        return Ok(None);
    }
    let size = union.square_extent().sqrt();
    if (size / distance) > (1. / criteria) {
        return Ok(None);
    }
    let mut transform = Transform3::default();
    transform.set_translation(Vector3::between(minimum, point));
    Ok(Some(transform))
}
