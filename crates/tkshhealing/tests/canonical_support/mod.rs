//! Original canonical-recognition fixture builders from OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
pub use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::{message::ProgressRange, ncollection::Array1};
pub use rustycad_tkernel::{precision, standard::Handle};
use rustycad_tkg3d::geom::{GeomBezierCurve, GeomCircle, GeomEllipse, SurfaceBounds};
pub use rustycad_tkg3d::geom::{
    GeomConicalSurface, GeomCurve, GeomCylindricalSurface, GeomPlane, GeomSphericalSurface,
    GeomSurface, GeomSurfaceOfLinearExtrusion,
};
pub use rustycad_tkgeombase::geom_convert;
use rustycad_tkmath::convert::ParameterisationType;
pub use rustycad_tkmath::gp::{
    Axis2, Axis3, Circle3, Cone3, Cylinder3, Direction3, Ellipse3, Line3, Plane3, Point3, Sphere3,
    dx, dz,
};
pub use rustycad_tkshhealing::shape_analysis::ShapeAnalysisCanonicalRecognition;
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeWire, BRepBuilderAPINurbsConvert,
    BRepBuilderAPISewing,
};
pub use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeFace, BRepBuilderAPIMakePolygon, BRepBuilderAPIMakeShape,
};
pub use std::f64::consts::{FRAC_PI_2, PI};

pub fn canonical_near(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "actual {actual}, expected {expected} within {tolerance}"
    );
}
pub fn canonical_frame() -> Axis3 {
    Axis3::from_axis2(Axis2::new(Point3::default(), dz()).unwrap())
}
pub fn canonical_plane() -> Handle<dyn GeomSurface> {
    Handle::new(GeomPlane::from_pln(Plane3::default())).cast()
}
pub fn canonical_cylinder() -> Handle<dyn GeomSurface> {
    Handle::new(GeomCylindricalSurface::new(canonical_frame(), 1.0).unwrap()).cast()
}
pub fn canonical_cone(radius: f64) -> Handle<dyn GeomSurface> {
    Handle::new(GeomConicalSurface::new(canonical_frame(), PI / 6.0, radius).unwrap()).cast()
}
pub fn canonical_sphere() -> Handle<dyn GeomSurface> {
    Handle::new(GeomSphericalSurface::new(canonical_frame(), 1.0).unwrap()).cast()
}
pub fn canonical_face(surface: &Handle<dyn GeomSurface>, bounds: SurfaceBounds) -> Shape {
    let maker =
        BRepBuilderAPIMakeFace::from_surface_bounds(surface, bounds, precision::confusion())
            .unwrap();
    assert!(maker.is_done());
    maker.face().unwrap().into()
}
pub fn canonical_nurbs(shape: &Shape) -> Shape {
    let mut converter = BRepBuilderAPINurbsConvert::with_shape(shape, false).unwrap();
    assert!(converter.is_done());
    converter.shape().unwrap().clone()
}
pub fn canonical_recognizer(shape: &Shape) -> ShapeAnalysisCanonicalRecognition {
    let mut recognizer = ShapeAnalysisCanonicalRecognition::new();
    recognizer.set_shape(shape).unwrap();
    recognizer
}
pub fn canonical_bezier(middle_y: f64) -> Handle<dyn GeomCurve> {
    let poles = [
        Point3::default(),
        Point3::new(0.5, middle_y, 0.0),
        Point3::new(1.0, 0.0, 0.0),
    ];
    Handle::new(GeomBezierCurve::new(&Array1::from_shared(&poles, 1)).unwrap()).cast()
}
pub fn canonical_bspline_conic(ellipse: bool) -> Handle<dyn GeomCurve> {
    let frame = Axis2::with_x_direction(Point3::default(), dz(), dx()).unwrap();
    let curve: Handle<dyn GeomCurve> = if ellipse {
        Handle::new(GeomEllipse::new(frame, 1.0, 0.5).unwrap()).cast()
    } else {
        let frame = Axis2::new(Point3::default(), dz()).unwrap();
        Handle::new(GeomCircle::new(frame, 1.0).unwrap()).cast()
    };
    geom_convert::curve_to_bspline_curve(&curve, ParameterisationType::TgtThetaOver2)
        .unwrap()
        .cast()
}
pub fn canonical_edge(curve: &Handle<dyn GeomCurve>) -> Shape {
    let maker = BRepBuilderAPIMakeEdge::from_curve(curve).unwrap();
    assert!(maker.is_done());
    maker.edge().unwrap().into()
}
pub fn canonical_wire(curve: &Handle<dyn GeomCurve>, parameters: [f64; 4]) -> Shape {
    let makers = parameters
        .windows(2)
        .map(|range| BRepBuilderAPIMakeEdge::from_curve_params(curve, range[0], range[1]).unwrap())
        .collect::<Vec<_>>();
    assert!(makers.iter().all(BRepBuilderAPIMakeShape::is_done));
    let mut wire = BRepBuilderAPIMakeWire::new();
    for maker in makers {
        wire.add_edge(&maker.edge().unwrap()).unwrap();
    }
    assert!(wire.is_done());
    wire.wire().unwrap().into()
}
pub fn canonical_sew(surface: &Handle<dyn GeomSurface>, bounds: [SurfaceBounds; 4]) -> Shape {
    let faces = bounds.map(|bounds| canonical_face(surface, bounds));
    let mut sewing = BRepBuilderAPISewing::default();
    for face in faces {
        sewing.add(&face).unwrap();
    }
    sewing.perform(&ProgressRange::default()).unwrap();
    let shape = sewing.sewed_shape().clone();
    assert!(!shape.is_null());
    shape
}
