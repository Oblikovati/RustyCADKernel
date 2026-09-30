//! Public overloads and failed initialization retain the same OCCT object state.
use super::super::{
    common::{Reader, attempt, end, flag, int, real, show},
    contap_surface,
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::{Adaptor3dTopolTool, h_surface_tool::Adaptor3dSurfaceHandle};
use rustycad_tkhlr::contap::ContapContour;
use rustycad_tkmath::gp::{Point3, Vector3};
fn report(contour: &mut ContapContour) {
    flag(contour.is_done());
    show(contour.is_empty(), flag);
    show(contour.nb_lines(), int);
    show(contour.line(0).map(|_| ()), |()| {});
    show(
        contour
            .surface_function()
            .map(|f| (f.function_type(), f.direction(), f.eye())),
        |(kind, dir, eye)| {
            int(kind as i32);
            for v in [dir.x(), dir.y(), dir.z(), eye.x(), eye.y(), eye.z()] {
                real(v);
            }
        },
    );
}
fn constructors(
    surface: &Adaptor3dSurfaceHandle,
    domain: &Adaptor3dTopolTool,
    dir: Vector3,
) -> Result<(), Failure> {
    let eye = Point3::new(3., 4., 5.);
    let mut candidates = [
        ContapContour::with_direction(dir)?,
        ContapContour::with_direction_angle(dir, 0.2)?,
        ContapContour::with_eye(eye),
        ContapContour::with_surface_direction(surface, domain, dir)?,
        ContapContour::with_surface_direction_angle(surface, domain, dir, 0.2)?,
        ContapContour::with_surface_eye(surface, domain, eye)?,
    ];
    for contour in &mut candidates {
        report(contour);
        show(contour.perform(surface, domain), |()| {});
        report(contour);
    }
    Ok(())
}
fn mutate_views(
    contour: &mut ContapContour,
    surface: &Adaptor3dSurfaceHandle,
    domain: &Adaptor3dTopolTool,
    dir: Vector3,
) {
    let eye = Point3::new(3., 4., 5.);
    let zero = Vector3::new(0., 0., 0.);
    show(contour.perform_direction(surface, domain, dir), |()| {});
    report(contour);
    show(contour.init_direction(zero), |()| {});
    report(contour);
    show(contour.perform(surface, domain), |()| {});
    report(contour);
    show(contour.init_direction_angle(zero, 0.2), |()| {});
    report(contour);
    show(
        contour.perform_direction_angle(surface, domain, dir, 0.2),
        |()| {},
    );
    report(contour);
    show(contour.perform_eye(surface, domain, eye), |()| {});
    report(contour);
    show(
        contour
            .surface_function()
            .map(|f| f.set_eye(Point3::new(0., 0., 0.))),
        |()| {},
    );
    show(contour.perform(surface, domain), |()| {});
    report(contour);
}
pub fn state_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    attempt(|| {
        let surface = contap_surface::surface_adaptor(&mut reader)?;
        let domain = Adaptor3dTopolTool::with_surface(&surface)?;
        let dir = Vector3::from(reader.dir());
        let mut contour = ContapContour::new();
        report(&mut contour);
        show(contour.perform(&surface, &domain), |()| {});
        report(&mut contour);
        constructors(&surface, &domain, dir)?;
        mutate_views(&mut contour, &surface, &domain, dir);
        Ok(())
    });
    end();
}
