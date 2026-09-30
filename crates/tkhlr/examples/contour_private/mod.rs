mod internal;
pub use internal::internal_case;
mod attachment;
pub use attachment::attachment_case;
mod sections;
pub use sections::sections_case;
// Direct private-helper outputs and their observable surface-function state.
use super::{
    common::{Reader, attempt, end, flag, int, pnt, real, show, xyz},
    math_helpers,
};
use rustycad_tkhlr::contap::{ContapLine, ContapSurfFunction};
use rustycad_tkmath::gp::{Axis2, Circle3, Direction3, Line3, Point2, Point3, Vector3};
pub fn math_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    attempt(|| {
        let surface = super::contap_surface::surface_adaptor(&mut reader)?;
        let (mut u, mut v) = (reader.real(), reader.real());
        let uv = Point2::new(u, v);
        let tangent = Vector3::from(reader.xyz());
        show(math_helpers::recadre(&surface, &mut u, &mut v), |()| {});
        real(u);
        real(v);
        let mut function = ContapSurfFunction::new();
        function.set_surface(&surface)?;
        function.set_direction(reader.dir());
        show(
            math_helpers::transition_on_line(&mut function, u, v, tangent),
            |t| int(t as i32),
        );
        pnt(function.point());
        real(function.root());
        let geometry = Line3::new(Point3::new(0.2, -0.3, 0.4), Direction3::Z);
        show(
            math_helpers::line_transition(&mut function, geometry),
            |t| int(t as i32),
        );
        pnt(function.point());
        let circle = Circle3::new(Axis2::default(), reader.real())?;
        show(
            math_helpers::circle_transition(&mut function, circle),
            |t| int(t as i32),
        );
        pnt(function.point());
        let mut line = ContapLine::new();
        line.set_line(geometry);
        print_match(&line, &surface, uv);
        line.set_circle(circle);
        print_match(&line, &surface, uv);
        Ok(())
    });
    end();
}
fn print_match(
    line: &ContapLine,
    surface: &rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle,
    uv: Point2,
) {
    let mut found = math_helpers::LineMatch {
        parameter: 17.,
        tangent: Vector3::new(1., 2., 3.),
        ..Default::default()
    };
    show(math_helpers::find_line(line, surface, uv, &mut found), flag);
    pnt(found.point);
    real(found.parameter);
    xyz(*found.tangent.xyz());
    xyz(*found.normal.xyz());
}
