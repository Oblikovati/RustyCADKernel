//! Bit-exact native curve sampling and Graphic3d callback oracle.
use super::common::{Reader, attempt, end, flag, int, pnt, real};
use super::sensitive_display::display_driver;
#[path = "../../tests/support/stdprs_curve_geometry.rs"]
pub(super) mod geometry;
use rustycad_tkernel::{
    ncollection::Sequence,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::graphic3d::*;
use rustycad_tkv3d::{prs3d::Prs3dDrawer, std_prs::StdPrsCurve, v3d::V3dViewer};

pub fn curve_add_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    let (kind, variant, draw, arrow, count) = (
        reader.integer(),
        reader.integer(),
        reader.flag(),
        reader.flag(),
        reader.integer(),
    );
    let bounds = [reader.real(), reader.real()];
    let (driver, trace) = display_driver::SensitiveDisplayDriver::new_handle(0);
    let viewer = V3dViewer::new(&driver).unwrap();
    let prs = Graphic3dStructure::new(&viewer.structure_manager(), &Handle::default()).unwrap();
    let drawer = curve_drawer(count, arrow);
    let mut points = Sequence::default();
    points.append(Point3::new(9., 8., 7.));
    let curve = geometry::curve_geometry(kind).unwrap();
    attempt(|| {
        curve_add(
            variant,
            &prs.cast(),
            &curve,
            &drawer,
            &mut points,
            bounds,
            draw,
        )
    });
    int(points.length());
    for point in points.iter() {
        pnt(*point);
    }
    let events = trace.read();
    int(events.events.len() as i32);
    for value in &events.events {
        real(*value);
    }
    int(prs.groups().len() as i32);
    end();
}
#[allow(clippy::too_many_arguments)]
fn curve_add(
    variant: i32,
    prs: &Handle<dyn Graphic3dStructureSource>,
    curve: &dyn Adaptor3dCurve,
    drawer: &Handle<Prs3dDrawer>,
    points: &mut Sequence<Point3>,
    bounds: [f64; 2],
    draw: bool,
) -> Result<(), Failure> {
    match variant {
        0 => StdPrsCurve::add(prs, curve, drawer, draw),
        1 => StdPrsCurve::add_range(prs, curve, bounds[0], bounds[1], drawer, draw),
        2 => StdPrsCurve::add_points(prs, curve, drawer, points, draw),
        _ => StdPrsCurve::add_range_points(
            prs,
            curve,
            bounds[0],
            bounds[1],
            points,
            drawer.discretisation(),
            draw,
        ),
    }
}
pub(super) fn curve_drawer(count: i32, arrow: bool) -> Handle<Prs3dDrawer> {
    let drawer = Handle::new(Prs3dDrawer::new());
    drawer.setup_own_defaults().unwrap();
    drawer.set_discretisation(count);
    drawer.set_maximal_parameter_value(12.);
    drawer.set_maximal_chordial_deviation(0.3);
    drawer.set_line_arrow_draw(arrow);
    drawer.arrow_aspect().set_length(1.);
    drawer.arrow_aspect().set_angle(0.2).unwrap();
    drawer
}
pub fn curve_match_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    let (kind, variant, count) = (reader.integer(), reader.integer(), reader.integer());
    let point = reader.pnt();
    let distance = reader.real();
    let bounds = [reader.real(), reader.real()];
    let drawer = curve_drawer(count, false);
    let curve = geometry::curve_geometry(kind).unwrap();
    attempt(|| {
        let matched = match variant {
            0 => StdPrsCurve::matches(&point, distance, &curve, &drawer)?,
            1 => StdPrsCurve::matches_with_sampling(&point, distance, &curve, 0.3, 12., count)?,
            2 => {
                StdPrsCurve::matches_range(&point, distance, &curve, bounds[0], bounds[1], &drawer)?
            }
            _ => StdPrsCurve::matches_range_with_sampling(
                &point, distance, &curve, bounds[0], bounds[1], 0.3, count,
            )?,
        };
        flag(matched);
        Ok(())
    });
    end();
}
