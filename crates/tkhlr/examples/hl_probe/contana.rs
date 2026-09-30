//! Analytic contour construction, compared with the unchanged OCCT class.
use super::common::{Reader, attempt, dir, end, flag, int, pnt, real};
use rustycad_tkernel::standard::Failure;
use rustycad_tkhlr::contap::ContapContAna;
use rustycad_tkmath::gp::{Axis2, Axis3, Cone3, Cylinder3, Direction3, Point3, Sphere3, Xyz};

fn contour_snapshot(contour: &ContapContAna) {
    flag(contour.is_done());
    attempt(|| {
        int(contour.nb_contours()?);
        Ok(())
    });
    attempt(|| {
        int(contour.type_contour()? as i32);
        Ok(())
    });
    attempt(|| contour_circle(contour));
    for index in [-1, 0, 1, 2, 3, 4, 5] {
        attempt(|| {
            let line = contour.line(index)?;
            pnt(line.location());
            dir(line.direction());
            Ok(())
        });
    }
}
fn contour_circle(contour: &ContapContAna) -> Result<(), Failure> {
    let circle = contour.circle()?;
    pnt(circle.location());
    dir(circle.position().direction());
    dir(circle.position().x_direction());
    real(circle.radius());
    Ok(())
}
struct AnalyticInput {
    frame: Axis3,
    radius: f64,
    semi_angle: f64,
    view: Xyz,
    draft: f64,
}
fn read_analytic(r: &mut Reader) -> Result<AnalyticInput, Failure> {
    let mut frame = Axis3::from_axis2(Axis2::with_x_direction(r.pnt(), r.dir(), r.dir())?);
    if r.integer() != 0 {
        frame.y_reverse();
    }
    Ok(AnalyticInput {
        frame,
        radius: r.real(),
        semi_angle: r.real(),
        view: r.xyz(),
        draft: r.real(),
    })
}
fn perform_analytic(c: &mut ContapContAna, mode: i32, a: &AnalyticInput) -> Result<(), Failure> {
    if mode % 3 == 2 {
        let eye = Point3::new(a.view.x(), a.view.y(), a.view.z());
        return match mode / 3 {
            0 => c.perform_sphere_eye(&Sphere3::new(a.frame, a.radius)?, eye),
            1 => c.perform_cylinder_eye(&Cylinder3::new(a.frame, a.radius)?, eye),
            _ => c.perform_cone_eye(&Cone3::new(a.frame, a.semi_angle, a.radius)?, eye),
        };
    }
    perform_direction(c, mode, a, Direction3::from_xyz(a.view)?)
}
fn perform_direction(
    c: &mut ContapContAna,
    mode: i32,
    a: &AnalyticInput,
    d: Direction3,
) -> Result<(), Failure> {
    match mode {
        0 => c.perform_sphere(&Sphere3::new(a.frame, a.radius)?, d),
        1 => c.perform_sphere_draft(&Sphere3::new(a.frame, a.radius)?, d, a.draft),
        3 => c.perform_cylinder(&Cylinder3::new(a.frame, a.radius)?, d),
        4 => c.perform_cylinder_draft(&Cylinder3::new(a.frame, a.radius)?, d, a.draft),
        6 => c.perform_cone(&Cone3::new(a.frame, a.semi_angle, a.radius)?, d),
        _ => c.perform_cone_draft(&Cone3::new(a.frame, a.semi_angle, a.radius)?, d, a.draft),
    }
}
/// Every Perform overload, fresh or reused state, and all accessor error domains.
pub fn analytic_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mode = r.integer();
    let warm = r.integer();
    let mut contour = ContapContAna::new();
    attempt(|| {
        let input = read_analytic(&mut r)?;
        if warm != 0 {
            contour.perform_sphere(&Sphere3::new(Axis3::default(), 2.0)?, Direction3::Z)?;
        }
        contour_snapshot(&contour);
        perform_analytic(&mut contour, mode, &input)
    });
    contour_snapshot(&contour);
    end();
}
