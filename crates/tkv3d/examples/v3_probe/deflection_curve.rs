//! Native adaptive curve sampling, overload effects and match decisions.
use super::{
    common::{Reader, attempt, end, flag, int, pnt, real},
    sensitive_display::display_driver,
    stdprs_curve::{curve_drawer, geometry},
};
use rustycad_tkernel::{
    ncollection::Sequence,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::{aspect::AspectTypeOfDeflection, graphic3d::*};
use rustycad_tkv3d::{prs3d::Prs3dDrawer, std_prs::StdPrsDeflectionCurve as Curve, v3d::V3dViewer};
fn drawer(relative: bool, arrow: bool) -> Handle<Prs3dDrawer> {
    let drawer = curve_drawer(4, arrow);
    drawer.set_type_of_deflection(if relative {
        AspectTypeOfDeflection::Relative
    } else {
        AspectTypeOfDeflection::Absolute
    });
    drawer.set_deviation_coefficient(0.025);
    drawer.set_deviation_angle(0.2);
    drawer
}
pub fn add_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (kind, variant, draw, arrow, relative) =
        (r.integer(), r.integer(), r.flag(), r.flag(), r.flag());
    let bounds = [r.real(), r.real()];
    let (driver, trace) = display_driver::SensitiveDisplayDriver::new_handle(0);
    let viewer = V3dViewer::new(&driver).unwrap();
    let prs = Graphic3dStructure::new(&viewer.structure_manager(), &Handle::default()).unwrap();
    let drawer = drawer(relative, arrow);
    let curve = geometry::curve_geometry(kind).unwrap();
    let mut points = Sequence::default();
    points.append(Point3::new(9., 8., 7.));
    attempt(|| {
        add(
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
fn add(
    variant: i32,
    prs: &Handle<dyn Graphic3dStructureSource>,
    curve: &dyn Adaptor3dCurve,
    drawer: &Handle<Prs3dDrawer>,
    points: &mut Sequence<Point3>,
    bounds: [f64; 2],
    draw: bool,
) -> Result<(), Failure> {
    match variant {
        0 => Curve::add(prs, curve, drawer, draw),
        1 => Curve::add_range(prs, curve, bounds[0], bounds[1], drawer, draw),
        2 => Curve::add_with_deflection(prs, curve, 0.3, 12., 0.2, draw),
        3 => Curve::add_points(prs, curve, 0.3, drawer, points, draw),
        _ => Curve::add_range_points(prs, curve, bounds[0], bounds[1], 0.3, points, 0.2, draw),
    }
}
pub fn match_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (kind, variant, relative) = (r.integer(), r.integer(), r.flag());
    let (point, distance, first, last) = (r.pnt(), r.real(), r.real(), r.real());
    let curve = geometry::curve_geometry(kind).unwrap();
    let drawer = drawer(relative, false);
    attempt(|| {
        let matched = match variant {
            0 => Curve::matches(&point, distance, &curve, &drawer)?,
            1 => Curve::matches_range(&point, distance, &curve, first, last, &drawer)?,
            2 => Curve::matches_with_deflection(&point, distance, &curve, 0.3, 12., 0.2)?,
            _ => Curve::matches_range_with_deflection(
                &point, distance, &curve, first, last, 0.3, 0.2,
            )?,
        };
        flag(matched);
        Ok(())
    });
    end();
}
