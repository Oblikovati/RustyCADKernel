//! Native sampler conventions differ before and after SamplePnts, and survive reinitialization.
use rustycad_tkbo::int_tools::TopolTool;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::{
    adaptor3d::Adaptor3dSurface,
    geom::{GeomPlane, GeomSurface},
    geom_adaptor::GeomAdaptorSurface,
};
use rustycad_tkmath::gp::Axis3;
fn plane() -> Handle<dyn Adaptor3dSurface> {
    let geometry: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    Handle::new(GeomAdaptorSurface::with_range(&geometry, ((0., 11., 0., 22.), (0., 0.))).unwrap())
        .cast()
}
#[test]
fn plane_counts_and_native_zero_based_interior_samples() {
    let tool = TopolTool::with_surface(&plane()).unwrap();
    assert_eq!(tool.nb_samples_u().unwrap(), 10);
    assert_eq!(tool.nb_samples_v().unwrap(), 10);
    assert_eq!(tool.nb_samples().unwrap(), 100);
    tool.compute_sample_points().unwrap();
    let (uv, p) = tool.sample_point(0).unwrap();
    assert_eq!((uv.x(), uv.y()), (1., 2.));
    assert_eq!((p.x(), p.y(), p.z()), (1., 2., 0.));
    assert!(tool.more());
    assert_eq!(
        tool.initialize().unwrap_err().exception_type(),
        "Standard_NotImplemented"
    );
}
#[test]
fn explicit_parameter_arrays_are_one_based_and_survive_reinitialize() {
    let tool = TopolTool::default();
    assert!(tool.u_parameters().is_err());
    assert!(tool.v_parameters().is_err());
    tool.initialize_surface(&plane()).unwrap();
    tool.sample_pnts(0.1, 2, 3).unwrap();
    assert_eq!(tool.u_parameters().unwrap().length(), 10);
    assert_eq!(tool.v_parameters().unwrap().length(), 10);
    assert_eq!(tool.sample_point(1).unwrap().0.x(), 0.);
    assert!(tool.sample_point(0).is_err());
    tool.initialize_surface(&plane()).unwrap();
    assert_eq!(tool.sample_point(1).unwrap().0.x(), 0.);
    assert_eq!(*tool.u_parameters().unwrap().last().unwrap(), 11.);
    assert_eq!(tool.nb_samples().unwrap(), 100);
    assert_eq!(tool.sample_point(100).unwrap().0.y(), 22.);
}
#[test]
fn sampler_handles_retain_native_rtti_and_inherited_topology() {
    let tool = Handle::new(TopolTool::with_surface(&plane()).unwrap());
    assert_eq!(tool.dynamic_type().unwrap().name(), "IntTools_TopolTool");
    tool.init();
    let mut count = 0;
    while tool.more() {
        tool.value().unwrap();
        tool.next();
        count += 1;
    }
    assert_eq!(count, 4);
}
#[test]
fn intersection_domain_dispatch_preserves_inttools_sampler() {
    let domain: std::rc::Rc<dyn rustycad_tkgeomalgo::int_start::TheTopolTool> =
        std::rc::Rc::new(TopolTool::with_surface(&plane()).unwrap());
    assert_eq!(domain.nb_samples_u().unwrap(), 10);
    assert_eq!(domain.nb_samples_v().unwrap(), 10);
    assert_eq!(domain.nb_samples().unwrap(), 100);
    assert_eq!(
        domain.sample_point(0).unwrap().0,
        rustycad_tkmath::gp::Point2::new(1., 2.)
    );
    domain.sample_pnts(0.1, 2, 3).unwrap();
    assert_eq!(domain.u_parameters().unwrap().length(), 10);
    assert_eq!(domain.v_parameters().unwrap().length(), 10);
    assert_eq!(
        domain.sample_point(1).unwrap().0,
        rustycad_tkmath::gp::Point2::default()
    );
    assert!(domain.is_uniform_sampling().unwrap());
}
