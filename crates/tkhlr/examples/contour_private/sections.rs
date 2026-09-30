//! Fully observable trimming results for real rectangular surface domains.
use super::super::{
    common::{Reader, attempt, end, flag, int, pnt, real},
    contap_surface, line_constructor,
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dTopolTool;
use rustycad_tkgeomalgo::int_surf::{IntSurfLineOn2S, IntSurfPntOn2S, IntSurfTypeTrans};
use rustycad_tkhlr::contap::{ContapIType, ContapLine, ContapPoint};
use rustycad_tkmath::gp::{Axis2, Circle3, Direction3, Line3, Point3};
fn walking_fixture(line: &mut ContapLine, duplicate: bool, offset: f64) {
    let points = Handle::new(IntSurfLineOn2S::new(None));
    for index in 1..=7 {
        let x = if duplicate {
            f64::from(index / 3)
        } else {
            f64::from(index)
        };
        let mut point = IntSurfPntOn2S::new();
        point.set_value_all(Point3::new(x, 0., 0.), 17., 19., offset + x * 0.1, offset);
        points.add(&point);
    }
    line.set_line_on_2s(points);
}
fn add_vertices(line: &ContapLine, scenario: i32) {
    let parameters: &[f64] = match scenario {
        0 => &[],
        1 => &[2.],
        2 => &[2., 2.],
        3 => &[1., 4., 7.],
        4 => &[1., 1.0000000005, 4.],
        _ => &[-2., 0., 2.],
    };
    for &parameter in parameters {
        let mut point = ContapPoint::with_value(Point3::new(parameter, 2., 3.), 4., 5.);
        point.set_parameter(parameter);
        if parameter == 2. {
            point.set_multiple();
            point.set_internal();
        }
        line.add_vertex(&point);
    }
}
fn report(line: &ContapLine) -> Result<(), Failure> {
    int(line.type_contour() as i32);
    int(line.transition_on_s() as i32);
    int(line.nb_vertex());
    for index in 1..=line.nb_vertex() {
        let point = line.vertex(index)?;
        pnt(point.value());
        real(point.parameter_on_line());
        let (u, v) = point.parameters();
        real(u);
        real(v);
        flag(point.is_multiple());
        flag(point.is_internal());
    }
    if line.type_contour() != ContapIType::Walking {
        return Ok(());
    }
    int(line.nb_pnts()?);
    for index in 1..=line.nb_pnts()? {
        let point = line.point(index)?;
        pnt(point.value());
        let (a, b, c, d) = point.parameters();
        real(a);
        real(b);
        real(c);
        real(d);
    }
    Ok(())
}
pub fn sections_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    attempt(|| {
        let surface = contap_surface::surface_adaptor(&mut reader)?;
        let domain = Adaptor3dTopolTool::with_surface(&surface)?;
        let (kind, scenario, duplicate, offset) = (
            reader.integer(),
            reader.integer(),
            reader.flag(),
            reader.real(),
        );
        let mut line = ContapLine::new();
        match kind {
            0 => walking_fixture(&mut line, duplicate, offset),
            1 => line.set_line(Line3::new(Point3::new(2., 0., 0.), Direction3::Z)),
            2 => line.set_circle(Circle3::new(Axis2::default(), 2.)?),
            _ => {
                domain.init();
                line.set_arc(&domain.value()?);
            }
        }
        add_vertices(&line, scenario);
        line.set_transition_on_s(IntSurfTypeTrans::Out);
        let mut output = Vec::new();
        attempt(|| line_constructor::construct_lines(&mut output, &domain, &line, &surface));
        int(output.len() as i32);
        for section in &output {
            report(section)?;
        }
        report(&line)
    });
    end();
}
