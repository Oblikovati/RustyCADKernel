//! Adaptive point streams and side effects at the real Graphic3d recording seam.
#[allow(dead_code)]
#[path = "support/sensitive_display_driver.rs"]
mod display_driver;
#[path = "support/stdprs_curve_geometry.rs"]
mod geometry;
use rustycad_tkernel::{ncollection::Sequence, standard::Handle};
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::graphic3d::*;
use rustycad_tkv3d::{prs3d::Prs3dDrawer, std_prs::StdPrsDeflectionCurve as Curve, v3d::V3dViewer};
fn fixture() -> (Handle<dyn Graphic3dStructureSource>, Handle<Prs3dDrawer>) {
    let (driver, _) = display_driver::SensitiveDisplayDriver::new_handle(0);
    let viewer = V3dViewer::new(&driver).unwrap();
    let prs = Graphic3dStructure::new(&viewer.structure_manager(), &Handle::default())
        .unwrap()
        .cast();
    let drawer = Handle::new(Prs3dDrawer::new());
    drawer.setup_own_defaults().unwrap();
    drawer.set_maximal_parameter_value(12.);
    (prs, drawer)
}
#[test]
fn draw_false_preserves_groups_even_with_arrow_and_appends_requested_points() {
    let (prs, drawer) = fixture();
    drawer.set_line_arrow_draw(true);
    let curve = geometry::curve_geometry(1).unwrap();
    Curve::add(&prs, &curve, &drawer, false).unwrap();
    Curve::add_range(&prs, &curve, 0.25, 0.75, &drawer, false).unwrap();
    Curve::add_with_deflection(&prs, &curve, 0.05, 12., 0.2, false).unwrap();
    let mut points = Sequence::default();
    points.append(Point3::new(9., 8., 7.));
    Curve::add_points(&prs, &curve, 0.05, &drawer, &mut points, false).unwrap();
    assert!(prs.groups().is_empty());
    assert!(points.len() > 3);
    assert_eq!(*points.iter().next().unwrap(), Point3::new(9., 8., 7.));
}
#[test]
fn c1_intervals_keep_duplicate_endpoints_and_empty_raw_range_adds_nothing() {
    let (prs, drawer) = fixture();
    let curve = geometry::curve_geometry(4).unwrap();
    let mut points = Sequence::default();
    Curve::add_points(&prs, &curve, 0.05, &drawer, &mut points, false).unwrap();
    let sampled: Vec<Point3> = points.iter().copied().collect();
    assert_eq!(sampled.len(), 18);
    assert_eq!(sampled[1], sampled[2]);
    Curve::add_range_points(&prs, &curve, 1., 0., 0.05, &mut points, 0.2, false).unwrap();
    assert_eq!(points.len(), 18);
}
#[test]
fn all_matching_overloads_include_endpoints_and_reject_far_queries() {
    let (_, drawer) = fixture();
    let curve = geometry::curve_geometry(0).unwrap();
    let near = Point3::new(-3., -2., 0.);
    let far = Point3::new(100., 100., 100.);
    assert!(Curve::matches(&near, 0.01, &curve, &drawer).unwrap());
    assert!(Curve::matches_with_deflection(&near, 0.01, &curve, 0.1, 12., 0.2).unwrap());
    assert!(Curve::matches_range(&near, 0.01, &curve, 0., 1., &drawer).unwrap());
    assert!(Curve::matches_range_with_deflection(&near, 0.01, &curve, 0., 1., 0.1, 0.2).unwrap());
    assert!(!Curve::matches(&far, 0.01, &curve, &drawer).unwrap());
}

#[derive(Default)]
struct ConstantInfiniteCurve {
    transient: rustycad_tkernel::standard::StandardTransient,
    evaluations: std::sync::atomic::AtomicUsize,
}
rustycad_tkernel::impl_transient!(ConstantInfiniteCurve,"Martensite_TestConstantInfiniteCurve",dyn rustycad_tkg3d::adaptor3d::Adaptor3dCurve,transient,
    interfaces:[dyn rustycad_tkg3d::adaptor3d::Adaptor3dCurve]);
impl rustycad_tkg3d::adaptor3d::Adaptor3dCurve for ConstantInfiniteCurve {
    fn first_parameter(&self) -> Result<f64, rustycad_tkernel::standard::Failure> {
        Ok(-rustycad_tkernel::precision::infinite())
    }
    fn last_parameter(&self) -> Result<f64, rustycad_tkernel::standard::Failure> {
        Ok(rustycad_tkernel::precision::infinite())
    }
    fn d0(&self, _: f64) -> Result<Point3, rustycad_tkernel::standard::Failure> {
        self.evaluations
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(Point3::default())
    }
}
#[test]
fn unbounded_degenerate_curve_stops_after_native_iteration_budget() {
    let (prs, drawer) = fixture();
    let curve = ConstantInfiniteCurve::default();
    Curve::add(&prs, &curve, &drawer, false).unwrap();
    assert_eq!(
        curve.evaluations.load(std::sync::atomic::Ordering::Relaxed),
        200000
    );
    assert!(prs.groups().is_empty());
    assert!(!Curve::matches(&Point3::default(), 1., &curve, &drawer).unwrap());
    assert_eq!(
        curve.evaluations.load(std::sync::atomic::Ordering::Relaxed),
        400000
    );
}
