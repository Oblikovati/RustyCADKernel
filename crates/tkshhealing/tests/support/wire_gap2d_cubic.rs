use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Edge, Face},
};
use rustycad_tkernel::{ncollection::Array1, standard::Handle};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve, Geom2dLine};
use rustycad_tkmath::gp::{Axis2d, Direction2, Point2};

pub(super) fn set_cubic_spline_and_line(face: &Face, first: &Edge, second: &Edge) {
    let poles = [
        Point2::new(0.0, 0.0),
        Point2::new(1.0 / 3.0, 0.5),
        Point2::new(2.0 / 3.0, -0.5),
        Point2::new(1.0, 0.0),
    ];
    let knots = [0.0, 1.0];
    let multiplicities = [4, 4];
    let spline: Handle<dyn Geom2dCurve> = Handle::new(
        Geom2dBSplineCurve::new(
            &Array1::from_shared(&poles, 1),
            &Array1::from_shared(&knots, 1),
            &Array1::from_shared(&multiplicities, 1),
            3,
            false,
        )
        .unwrap(),
    )
    .cast();
    let line: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(Axis2d::new(
        Point2::new(0.0, 0.1),
        Direction2::new(1.0, 0.0).unwrap(),
    )))
    .cast();
    let builder = BRepBuilder::default();
    super::support::set_pcurve_range(&builder, first, face, &spline, 1.0);
    super::support::set_pcurve_range(&builder, second, face, &line, 1.0);
}
