use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Edge, Face},
};
use rustycad_tkernel::{
    impl_transient,
    ncollection::Array1,
    standard::{Failure, Handle},
};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomCurve, GeomCylindricalSurface, GeomLine, GeomPlane, GeomSurface};
use rustycad_tkmath::gp::{Axis2d, Axis3, Direction3, Point2, Point3};
use rustycad_tkshhealing::{
    shape_extend::{ShapeExtend, ShapeExtendStatus},
    shape_upgrade::{
        ShapeUpgradeSplitCurve2d, ShapeUpgradeSplitCurve3d, SplitCurve, SplitCurveData,
    },
};

// Named native-style subclasses inject splitter outcomes without mocking geometry.
macro_rules! criterion {
    ($name:ident, $base:ty, $rtti:literal) => {
        pub struct $name { parent: $base, mode: i32 }
        impl_transient!($name, $rtti, $base, parent, interfaces: [dyn SplitCurve]);
        impl $name {
            pub fn new(mode: i32) -> Self { Self { parent: <$base>::new(), mode } }
        }
        impl SplitCurve for $name {
            fn split_curve_data(&self) -> &SplitCurveData { self.parent.split_curve_data() }
            fn compute(&self) -> Result<(), Failure> {
                if self.mode == 3 {
                    self.split_values().change_sequence(|s|s.insert_before(2,0.5).map(|_|()))?;
                }
                self.parent.compute()
            }
            fn build(&self, segment: bool) -> Result<(), Failure> {
                if self.mode == 4 { return Err(Failure::new("fixture splitter failure")); }
                self.parent.build(segment)?;
                let flag = match self.mode { 1 => ShapeExtendStatus::Fail1, 2 => ShapeExtendStatus::Done2, _ => return Ok(()) };
                self.split_curve_data().with_mut(|f|f.status=ShapeExtend::encode_status(flag));
                Ok(())
            }
        }
    };
}
criterion!(
    SpatialCriterion,
    ShapeUpgradeSplitCurve3d,
    "SmallBezierSpatialCriterion"
);
criterion!(
    PlanarCriterion,
    ShapeUpgradeSplitCurve2d,
    "SmallBezierPlanarCriterion"
);

pub fn edge(spatial: bool) -> Edge {
    let builder = BRepBuilder::default();
    let mut edge = Edge::default();
    builder.make_edge(&mut edge).unwrap();
    if spatial {
        let curve: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_point(
            Point3::default(),
            Direction3::new(1., 0., 0.).unwrap(),
        ))
        .cast();
        builder.update_edge_curve(&edge, &curve, 1e-7).unwrap();
    }
    builder.range(&edge, 0., 1., false).unwrap();
    edge
}
fn pcurve(y: f64) -> Handle<dyn Geom2dCurve> {
    let poles = [
        Point2::new(0., y),
        Point2::new(0.25, y),
        Point2::new(0.75, y),
        Point2::new(1., y),
    ];
    let poles = Array1::from_shared(&poles, 1);
    let knots = Array1::from_shared(&[0., 0.5, 1.], 1);
    let mults = Array1::from_shared(&[3, 1, 3], 1);
    Handle::new(Geom2dBSplineCurve::new(&poles, &knots, &mults, 2, false).unwrap()).cast()
}
pub fn face(edge: &Edge, seam: bool, geometry: u8) -> Face {
    let builder = BRepBuilder::default();
    let mut face = Face::default();
    let surface: Handle<dyn GeomSurface> = if seam {
        Handle::new(GeomCylindricalSurface::new(Axis3::default(), 1.).unwrap()).cast()
    } else {
        Handle::new(GeomPlane::new(Axis3::default())).cast()
    };
    builder
        .make_face_on_surface(&mut face, &surface, 1e-7)
        .unwrap();
    let forward: Handle<dyn Geom2dCurve> = if geometry == 1 {
        Handle::new(Geom2dLine::new(Axis2d::default())).cast()
    } else {
        pcurve(0.)
    };
    let reverse = if geometry == 2 {
        Handle::new(Geom2dLine::new(Axis2d::default())).cast()
    } else {
        pcurve(1.)
    };
    if seam {
        builder
            .update_edge_pcurves_on_face(edge, (&forward, &reverse), &face, 1e-7)
            .unwrap();
    } else {
        builder
            .update_edge_pcurve_on_face(edge, &forward, &face, 1e-7)
            .unwrap();
    }
    builder.range_on_face(edge, &face, 0., 1.).unwrap();
    face
}
