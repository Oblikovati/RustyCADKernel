//! State and caller-owned analytic geometry, matching the native canonical probe.
use super::common::{Reader, attempt, end, flag, int, pnt, real, xyz};
use super::shapes::{TpShapes, tp_shapes};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{
    Axis2, Axis3, Circle3, Cone3, Cylinder3, Ellipse3, Line3, Plane3, Sphere3,
};
use rustycad_tkshhealing::shape_analysis::ShapeAnalysisCanonicalRecognition;

fn frame(position: Axis3) {
    pnt(position.location());
    xyz(position.direction().xyz());
    xyz(position.x_direction().xyz());
    xyz(position.y_direction().xyz());
}
fn analytic_surface(
    tool: &mut ShapeAnalysisCanonicalRecognition,
    op: i32,
    tol: f64,
    radius: f64,
    angle: f64,
) -> Result<(), Failure> {
    match op {
        2 => {
            let mut p = Plane3::default();
            flag(tool.is_plane(tol, &mut p)?);
            frame(p.position());
        }
        3 => {
            let mut p = Cylinder3::new(Axis3::default(), radius)?;
            flag(tool.is_cylinder(tol, &mut p)?);
            frame(p.position());
            real(p.radius());
        }
        4 => {
            let mut p = Cone3::new(Axis3::default(), angle, radius)?;
            flag(tool.is_cone(tol, &mut p)?);
            frame(p.position());
            real(p.semi_angle());
            real(p.ref_radius());
        }
        _ => {
            let mut p = Sphere3::new(Axis3::default(), radius)?;
            flag(tool.is_sphere(tol, &mut p)?);
            frame(p.position());
            real(p.radius());
        }
    }
    Ok(())
}
fn analytic_curve(
    tool: &mut ShapeAnalysisCanonicalRecognition,
    op: i32,
    tol: f64,
    radius: f64,
) -> Result<(), Failure> {
    match op {
        6 => {
            let mut p = Line3::default();
            flag(tool.is_line(tol, &mut p)?);
            pnt(p.location());
            xyz(p.direction().xyz());
        }
        7 => {
            let mut p = Circle3::new(Axis2::default(), radius)?;
            flag(tool.is_circle(tol, &mut p)?);
            frame(Axis3::from_axis2(p.position()));
            real(p.radius());
        }
        _ => {
            let mut p = Ellipse3::new(Axis2::default(), radius, radius / 2.0)?;
            flag(tool.is_ellipse(tol, &mut p)?);
            frame(Axis3::from_axis2(p.position()));
            real(p.major_radius());
            real(p.minor_radius());
        }
    }
    Ok(())
}
fn recognize(
    r: &mut Reader<'_>,
    tool: &mut ShapeAnalysisCanonicalRecognition,
    op: i32,
) -> Result<(), Failure> {
    let (tol, radius, angle) = (r.real(), r.real(), r.real());
    if op < 6 {
        return analytic_surface(tool, op, tol, radius, angle);
    }
    analytic_curve(tool, op, tol, radius)
}
fn state(tool: &ShapeAnalysisCanonicalRecognition, scene: &TpShapes) {
    int(tool.get_status());
    real(tool.get_gap());
    flag(tool.get_shape().is_null());
    for shape in &scene.shapes {
        flag(tool.get_shape().is_same(shape));
    }
}
pub fn canonical_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let scene = tp_shapes(&mut r)?;
        let initial = r.integer();
        let mut tool = if initial < 0 {
            ShapeAnalysisCanonicalRecognition::new()
        } else {
            ShapeAnalysisCanonicalRecognition::with_shape(&scene.shapes[initial as usize])?
        };
        state(&tool, &scene);
        for _ in 0..r.integer() {
            let op = r.integer();
            match op {
                0 => {
                    let index = r.integer();
                    attempt(|| tool.set_shape(&scene.shapes[index as usize]));
                }
                1 => tool.clear_status(),
                _ => attempt(|| recognize(&mut r, &mut tool, op)),
            }
            state(&tool, &scene);
        }
        Ok(())
    });
    end();
}
