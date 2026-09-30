//! Analytic line overlaps, crossings and shared topology from the native helper.
use super::super::edge_lines::LinePair;
use super::{integer, real};
use rustycad_tkbo::int_tools::Range;
use rustycad_tkbrep::{
    brep_adaptor::BRepAdaptorCurve,
    topods::{Builder, ShapeIterator},
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::{
    adaptor3d::Adaptor3dCurve,
    geom::{GeomCurve, GeomLine},
};
use rustycad_tkmath::gp::{Direction3, Line3, Point3};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
fn direction(kind: i32) -> Result<Direction3, Failure> {
    let (x, y) = match kind {
        0 => (1., 0.),
        1 => (-1., 0.),
        2 => (0., 1.),
        3 => (1., 1.),
        4 => (1., 1e-8),
        _ => (1., 1e-14),
    };
    Direction3::new(x, y, 0.)
}
pub(super) fn case(values: &[f64]) -> Result<(), Failure> {
    let first: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_lin(Line3::new(
        Point3::default(),
        Direction3::X,
    )))
    .into_cast();
    let second: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_lin(Line3::new(
        Point3::new(values[1], values[2], values[3]),
        direction(values[0] as i32)?,
    )))
    .into_cast();
    let first_edge = BRepBuilderAPIMakeEdge::from_curve_params(&first, 0., 1.)?.edge()?;
    let mut second_edge = BRepBuilderAPIMakeEdge::from_curve_params(&second, 0., 1.)?.edge()?;
    if values[8] != 0. {
        let children = ShapeIterator::new(&first_edge, true, true);
        Builder.add(&mut second_edge, children.value()?)?;
    }
    let lines = (
        BRepAdaptorCurve::with_edge(&first_edge)?.line()?,
        BRepAdaptorCurve::with_edge(&second_edge)?.line()?,
    );
    let pair = LinePair {
        edges: (&first_edge, &second_edge),
        lines,
        ranges: (Range::new(values[4], values[5]), Range::new(-0.5, 0.75)),
        tolerances: (values[6], values[7]),
        tolerance: values[6] + values[7],
    };
    let result = pair.compute()?;
    integer(i32::from(result.is_some()));
    if let Some(part) = result {
        integer(part.shape_type() as i32);
        integer(i32::from(part.all_null_flag()));
        integer(i32::from(part.edge1().is_same(&first_edge)));
        integer(i32::from(part.edge2().is_same(&second_edge)));
        for value in [
            part.range1().first(),
            part.range1().last(),
            part.vertex_parameter1(),
            part.vertex_parameter2(),
        ] {
            real(value);
        }
        integer(part.ranges2().len() as i32);
        for range in part.ranges2() {
            real(range.first());
            real(range.last());
        }
    }
    Ok(())
}
