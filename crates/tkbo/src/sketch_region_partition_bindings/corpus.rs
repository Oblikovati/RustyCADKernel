// SPDX-License-Identifier: GPL-2.0-only
//! Authored inputs share the native corpus; no conditioning changes the mapped pipeline.
use super::fixtures::*;
use crate::sketch_region_candidates::tests::rectangle;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::{gp::Transform3, toploc::Location};

pub(crate) fn partition_binding_sources(mode: usize) -> Vec<Shape> {
    if mode == 0 {
        return two_root_edges();
    }
    if mode == 1 {
        return single_root_edges();
    }
    rectangle_binding_sources(mode)
}
fn rectangle_binding_sources(mode: usize) -> Vec<Shape> {
    let mut sources = rectangle(0., 0., 2., 2.);
    match mode {
        2 => {}
        3 => {
            sources = sources
                .iter()
                .map(|source| source.oriented(Orientation::Reversed))
                .collect()
        }
        4 => sources = located_sources(&sources),
        5 => sources.extend(rectangle(0., 0., 2., 2.)),
        6 => sources.push(sources[0].oriented(Orientation::Reversed)),
        _ => panic!("partition corpus mode={mode}; expected 0..=6"),
    }
    sources
}
fn located_sources(sources: &[Shape]) -> Vec<Shape> {
    let location = Location::new(Transform3::translation([2., 3., 0.]));
    sources
        .iter()
        .map(|source| source.located(&location, false).unwrap())
        .collect()
}
pub(super) fn circle_source() -> Shape {
    use rustycad_tkernel::standard::Handle;
    use rustycad_tkg3d::geom::{GeomCircle, GeomCurve};
    use rustycad_tkmath::gp::Axis2;
    use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
    let curve: Handle<dyn GeomCurve> =
        Handle::new(GeomCircle::new(Axis2::default(), 1.).unwrap()).cast();
    BRepBuilderAPIMakeEdge::from_curve(&curve)
        .unwrap()
        .edge()
        .unwrap()
        .into_shape()
}
