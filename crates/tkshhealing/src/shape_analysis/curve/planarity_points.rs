//! Ordered polygon normal and projection-span test from ShapeAnalysis_Curve.
use super::ShapeAnalysisCurve;
use rustycad_tkernel::{ncollection::Array1, precision, standard::Failure};
use rustycad_tkmath::gp::{Point3, Xyz};

pub(super) fn any_normal(direction: Xyz) -> Xyz {
    if direction.z().abs() < precision::confusion() {
        return Xyz::new(0.0, 0.0, 1.0);
    }
    let normal = Xyz::new(direction.z(), 0.0, -direction.x());
    let norm = normal.modulus();
    if norm < precision::confusion() {
        return Xyz::new(0.0, 0.0, 1.0);
    }
    normal / norm
}
fn point_coordinates(points: &Array1<'_, Point3>, index: i32) -> Result<Xyz, Failure> {
    points
        .value(index)
        .map(|point| point.xyz())
        .map_err(|e| e.into_failure("ShapeAnalysis_Curve::IsPlanar"))
}
fn centroid(points: &Array1<'_, Point3>) -> Result<Xyz, Failure> {
    let mut center = Xyz::default();
    for i in 1..=points.length() {
        center += point_coordinates(points, i)?;
    }
    Ok(center / f64::from(points.length()))
}
fn polygon_normal(points: &Array1<'_, Point3>, normal: &mut Xyz) -> Result<Xyz, Failure> {
    let center = centroid(points)?;
    let mut longest = point_coordinates(points, 1)? - center;
    *normal = (point_coordinates(points, points.length())? - center).crossed(longest);
    for i in 1..points.length() {
        let next = point_coordinates(points, i + 1)? - center;
        if next.square_modulus() > longest.square_modulus() {
            longest = next;
        }
        let mut contribution = (point_coordinates(points, i)? - center).crossed(next);
        if normal.dot(contribution) < 0.0 {
            contribution *= -1.0;
        }
        *normal += contribution;
    }
    Ok(longest)
}
fn projection_span(points: &Array1<'_, Point3>, normal: Xyz) -> Result<f64, Failure> {
    let (mut minimum, mut maximum) = (f64::MAX, -f64::MAX);
    for i in 1..=points.length() {
        let projection = point_coordinates(points, i)?.dot(normal);
        if projection < minimum {
            minimum = projection;
        }
        if projection > maximum {
            maximum = projection;
        }
    }
    Ok(maximum - minimum)
}
pub(super) fn line_normal(direction: Xyz, normal: &mut Xyz) -> bool {
    if normal.square_modulus() == 0.0 {
        *normal = any_normal(direction);
        return true;
    }
    direction.dot(*normal).abs() < precision::confusion()
}
impl ShapeAnalysisCurve {
    /// Detect point-array planarity while retaining native normal mutation; `ShapeAnalysisCurve::is_planar_points(&points,&mut normal,1e-7)?`.
    pub fn is_planar_points(
        points: &Array1<'_, Point3>,
        normal: &mut Xyz,
        tolerance: f64,
    ) -> Result<bool, Failure> {
        let tolerance = if tolerance > 0.0 {
            tolerance
        } else {
            precision::confusion()
        };
        if points.length() < 3 {
            return Ok(line_normal(
                point_coordinates(points, 1)? - point_coordinates(points, 2)?,
                normal,
            ));
        }
        let mut longest = Xyz::default();
        if normal.square_modulus() == 0.0 {
            longest = polygon_normal(points, normal)?;
        }
        let norm = normal.modulus();
        if norm < precision::confusion() {
            *normal = any_normal(longest);
            return Ok(true);
        }
        *normal /= norm;
        Ok(projection_span(points, *normal)? <= tolerance)
    }
}
