// SPDX-License-Identifier: GPL-2.0-only
//! Shared authored closed-curve fixture; no production geometric special case.
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::{ncollection::Array1, standard::Handle};
use rustycad_tkg3d::{
    geom::{GeomBezierCurve, GeomCurve},
    top_abs::Orientation,
};
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
pub fn closed_analytic_cubic(scale: f64, reverse: bool) -> Shape {
    let points = [
        Point3::new(0., 0., 0.),
        Point3::new(4. * scale, 0., 0.),
        Point3::new(0., 4. * scale, 0.),
        Point3::new(0., 0., 0.),
    ];
    let poles = Array1::from_shared(&points, 1);
    let curve: Handle<dyn GeomCurve> = Handle::new(GeomBezierCurve::new(&poles).unwrap()).cast();
    let source = BRepBuilderAPIMakeEdge::from_curve(&curve)
        .unwrap()
        .edge()
        .unwrap();
    let orientation = [Orientation::Forward, Orientation::Reversed][usize::from(reverse)];
    source.oriented(orientation)
}
