use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::{brep::BRepTool, topexp::Explorer};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point2;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeCylinder;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;

type SeamObservation = (f64, f64, Point2, Point2, Point2, Point2);

#[test]
fn rust_cylinder_seam_pcurves_match_native_brep_tool_observations() {
    let shape = make_cylinder_shape();
    let seams = seam_pcurve_observations(&shape);
    assert_eq!(seams.len(), 2);
    for (first, last, start, end, reverse_start, reverse_end) in seams {
        assert_eq!((first, last), (0.0, 5.0));
        assert_eq!((start.y(), end.y()), (0.0, 5.0));
        assert_eq!((reverse_start.y(), reverse_end.y()), (0.0, 5.0));
        assert!(is_cylinder_seam_u(start.x()));
        assert!(is_cylinder_seam_u(reverse_start.x()));
        assert!((start.x() + reverse_start.x() - std::f64::consts::TAU).abs() < 1e-12);
        assert_eq!(start.x(), end.x());
        assert_eq!(reverse_start.x(), reverse_end.x());
    }
}

fn make_cylinder_shape() -> Shape {
    let mut cylinder = BRepPrimAPIMakeCylinder::new(2.0, 5.0);
    cylinder.shape().unwrap().clone()
}

fn seam_pcurve_observations(shape: &Shape) -> Vec<SeamObservation> {
    let mut observations = Vec::new();
    for candidate in Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape).unwrap() {
        let face = rustycad_tkbrep::topods::face(&candidate).unwrap();
        observations.extend(face_seam_observations(&face));
    }
    observations
}

fn face_seam_observations(face: &rustycad_tkbrep::topods::Face) -> Vec<SeamObservation> {
    Explorer::new(
        &face.clone().into_shape(),
        ShapeEnum::Edge,
        ShapeEnum::Shape,
    )
    .unwrap()
    .filter_map(|candidate| seam_observation(&candidate, face))
    .collect()
}

fn seam_observation(
    candidate: &Shape,
    face: &rustycad_tkbrep::topods::Face,
) -> Option<SeamObservation> {
    let edge = rustycad_tkbrep::topods::edge(candidate).ok()?;
    if !BRepTool::is_closed_on_face(&edge, face).ok()? {
        return None;
    }
    edge_seam_observation(&edge, face).ok()
}

fn edge_seam_observation(
    edge: &rustycad_tkbrep::topods::Edge,
    face: &rustycad_tkbrep::topods::Face,
) -> Result<SeamObservation, rustycad_tkernel::standard::Failure> {
    let forward = BRepTool::curve_on_face(edge, face)?;
    let mut reversed = edge.clone();
    reversed.reverse();
    let backward = BRepTool::curve_on_face(&reversed, face)?;
    Ok((
        forward.first,
        forward.last,
        pcurve_value(&forward, forward.first),
        pcurve_value(&forward, forward.last),
        pcurve_value(&backward, backward.first),
        pcurve_value(&backward, backward.last),
    ))
}

fn pcurve_value(curve: &rustycad_tkbrep::brep::EdgePCurve, parameter: f64) -> Point2 {
    curve.pcurve.get().unwrap().value(parameter).unwrap()
}

fn is_cylinder_seam_u(value: f64) -> bool {
    value.abs() < 1e-12 || (value - std::f64::consts::TAU).abs() < 1e-12
}
