use super::{BRepMeshCircleTool, Xy, std_max};
impl BRepMeshCircleTool {
    /// Preserve native failed-output values and enlarged circumradius (OCC24923); `BRepMeshCircleTool::make_circle(&points,&mut center,&mut radius)`.
    pub fn make_circle(points: &[Xy; 3], center: &mut Xy, radius: &mut f64) -> bool {
        let links = circle_links(points);
        let precision = rustycad_tkernel::precision::p_confusion();
        if links
            .iter()
            .any(|link| link.square_modulus() < precision * precision)
        {
            return false;
        }
        let determinant = circle_determinant(points, &links);
        if determinant.abs() < rustycad_tkmath::gp::resolution() {
            return false;
        }
        *center = circle_center(points, &links, 1. / determinant);
        *radius = circle_radius(points, *center);
        true
    }
}
fn circle_links(points: &[Xy; 3]) -> [Xy; 3] {
    [
        Xy::new(points[2].x() - points[1].x(), points[1].y() - points[2].y()),
        Xy::new(points[0].x() - points[2].x(), points[2].y() - points[0].y()),
        Xy::new(points[1].x() - points[0].x(), points[0].y() - points[1].y()),
    ]
}
fn circle_center(points: &[Xy; 3], links: &[Xy; 3], inverse: f64) -> Xy {
    let a = points[0].square_modulus();
    let b = points[1].square_modulus();
    let c = points[2].square_modulus();
    Xy::new(
        (a * links[0].y() + b * links[1].y() + c * links[2].y()) * inverse,
        (a * links[0].x() + b * links[1].x() + c * links[2].x()) * inverse,
    )
}

fn circle_radius(points: &[Xy; 3], center: Xy) -> f64 {
    let a = (points[0] - center).square_modulus();
    let b = (points[1] - center).square_modulus();
    let c = (points[2] - center).square_modulus();
    std_max(std_max(a, b), c).sqrt() + 2. * f64::EPSILON
}

fn circle_determinant(points: &[Xy; 3], links: &[Xy; 3]) -> f64 {
    2. * (points[0].x() * links[0].y()
        + points[1].x() * links[1].y()
        + points[2].x() * links[2].y())
}
