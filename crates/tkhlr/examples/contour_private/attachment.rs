//! Real search results feed the original contour attachment helpers.
use super::super::{
    boundary_points,
    common::{Reader, attempt, end, flag, int, pnt, pnt2d, real, show, xy, xyz},
    contap_surface,
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::{Adaptor3dTopolTool, h_surface_tool::Adaptor3dSurfaceHandle};
use rustycad_tkgeomalgo::int_surf::IntSurfTransition;
use rustycad_tkhlr::contap::{
    ContapArcFunction, ContapLine, ContapSurfFunction, ContapTheSearch, ContapTheSearchInside,
};
use rustycad_tkmath::gp::{Axis2, Circle3, Direction3, Line3, Point3};
fn candidates() -> Result<Vec<ContapLine>, Failure> {
    let mut lines = Vec::new();
    for location in [
        Point3::new(0., 2., 0.),
        Point3::new(0., -2., 0.),
        Point3::new(2., 0., 0.),
    ] {
        let mut line = ContapLine::new();
        line.set_line(Line3::new(location, Direction3::Z));
        lines.push(line);
    }
    for direction in [Direction3::Z, Direction3::X] {
        let mut line = ContapLine::new();
        line.set_circle(Circle3::new(Axis2::new(Point3::default(), direction)?, 2.)?);
        lines.push(line);
    }
    Ok(lines)
}
fn transition(value: IntSurfTransition) {
    show(value.is_tangent(), flag);
    int(value.transition_type() as i32);
    show(value.situation(), |s| int(s as i32));
    show(value.is_opposite(), flag);
}
fn report(lines: &[ContapLine]) -> Result<(), Failure> {
    int(lines.len() as i32);
    for line in lines {
        super::sections::report(line)?;
        for index in 1..=line.nb_vertex() {
            let point = line.vertex(index)?;
            flag(point.is_on_arc());
            flag(point.is_vertex());
            real(point.parameter_on_arc()?);
            transition(point.transition_on_line()?);
            transition(point.transition_on_arc()?);
            if point.is_vertex() {
                pnt2d(point.vertex()?.value());
            }
        }
    }
    Ok(())
}
pub(super) fn functions(
    surface: &Adaptor3dSurfaceHandle,
    reader: &mut Reader,
) -> Result<(ContapArcFunction, ContapSurfFunction), Failure> {
    let mode = reader.integer();
    let direction = reader.dir();
    let mut arc = ContapArcFunction::new();
    arc.set_surface(surface)?;
    let mut function = ContapSurfFunction::new();
    function.set_surface(surface)?;
    match mode {
        0 => {
            arc.set_direction(direction);
            function.set_direction(direction);
        }
        1 => {
            let eye = Point3::new(3., 4., 5.);
            arc.set_eye(eye);
            function.set_eye(eye);
        }
        _ => {
            arc.set_direction_angle(direction, 0.2);
            function.set_direction_angle(direction, 0.2);
        }
    }
    function.set_tolerance(1e-7);
    Ok((arc, function))
}
pub fn attachment_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    attempt(|| {
        let surface = contap_surface::surface_adaptor(&mut reader)?;
        let domain = Adaptor3dTopolTool::with_surface(&surface)?;
        let (mut arc, mut function) = functions(&surface, &mut reader)?;
        let mut boundary = ContapTheSearch::new();
        boundary.perform(&mut arc, &domain, 1e-7, 1e-7, false)?;
        flag(boundary.is_done());
        int(boundary.nb_points()?);
        int(boundary.nb_segments()?);
        let lines = candidates()?;
        attempt(|| boundary_points::put_points_on_lines(&boundary, &surface, &lines));
        report(&lines)?;
        inside_points(&surface, &domain, &mut function, &boundary)
    });
    end();
}
fn inside_points(
    surface: &Adaptor3dSurfaceHandle,
    domain: &Adaptor3dTopolTool,
    function: &mut ContapSurfFunction,
    boundary: &ContapTheSearch,
) -> Result<(), Failure> {
    let inside = ContapTheSearchInside::with_domain(function, surface, domain, 1e-7)?;
    flag(inside.is_done());
    int(inside.nb_points()?);
    let mut output = Vec::new();
    attempt(|| boundary_points::keep_inside_points(&inside, boundary, function, &mut output));
    int(output.len() as i32);
    for point in output {
        pnt(point.value());
        real(point.u_parameter());
        real(point.v_parameter());
        xyz(*point.direction().xyz());
        xy(point.direction2d().xy());
    }
    Ok(())
}
