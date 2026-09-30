//! Regression tests for the virtual members of Adaptor3d_HVertex and Adaptor3d_TopolTool
//! (docs/topalgo-design.md section 8): the trait-backed vertex dispatches to its
//! implementation (as the native BRepTopAdaptor_HVertex overrides), the base instance
//! keeps the native base behaviour, and the SamplePnts body runs the counts of the
//! `ComputeSamplePoints` the caller passes.
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkernel::{impl_transient, standard::RuntimeTyped};
use rustycad_tkg2d::adaptor2d::{Adaptor2dCurve2d, Adaptor2dLine2d};
use rustycad_tkg3d::adaptor3d::{
    Adaptor3dHVertex, Adaptor3dHVertexBase, Adaptor3dSurface, Adaptor3dTopolTool,
};
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Axis3, Direction2, Direction3, Point2, Point3};

/// A subclass of the vertex whose members all differ from the base ones.
struct FakeOverridingVertex {
    parent: StandardTransient,
}
impl_transient!(
    FakeOverridingVertex,
    "FakeOverridingVertex",
    dyn Adaptor3dHVertex,
    parent,
    interfaces: [dyn Adaptor3dHVertex]
);
impl Adaptor3dHVertex for FakeOverridingVertex {
    fn value(&self) -> Point2 {
        Point2::new(7.0, 8.0)
    }
    fn parameter(&self, _curve: &Handle<dyn Adaptor2dCurve2d>) -> Result<f64, Failure> {
        Ok(42.0)
    }
    fn resolution(&self, _curve: &Handle<dyn Adaptor2dCurve2d>) -> Result<f64, Failure> {
        Err(Failure::with_message("Standard_NullObject", "fake"))
    }
    fn orientation(&self) -> Orientation {
        Orientation::Internal
    }
    fn is_same(&self, _other: &Handle<dyn Adaptor3dHVertex>) -> Result<bool, Failure> {
        Ok(false)
    }
}

fn line() -> Handle<dyn Adaptor2dCurve2d> {
    let d = Direction2::new(1.0, 0.0).unwrap();
    Handle::new(Adaptor2dLine2d::new(Point2::new(0.0, 0.0), d, 0.0, 10.0)).cast()
}

fn plane_adaptor() -> Handle<dyn Adaptor3dSurface> {
    let z = Direction3::new(0.0, 0.0, 1.0).unwrap();
    let frame = Axis3::from_normal(Point3::new(0.0, 0.0, 0.0), z).unwrap();
    let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(frame)).cast();
    Handle::new(GeomAdaptorSurface::with_surface(&plane).unwrap()).cast()
}

#[test]
fn base_vertex_keeps_the_native_base_members() {
    let base = Adaptor3dHVertexBase::new(Point2::new(3.0, 0.0), Orientation::Reversed, 0.25);
    let vertex: Handle<dyn Adaptor3dHVertex> = Handle::new(base).cast();
    assert_eq!(vertex.value(), Point2::new(3.0, 0.0));
    assert_eq!(vertex.parameter(&line()).unwrap(), 3.0);
    assert_eq!(vertex.resolution(&line()).unwrap(), 0.25);
    assert_eq!(vertex.orientation(), Orientation::Reversed);
    assert!(vertex.is_same(&vertex).unwrap());
    assert!(vertex.is_kind_name("Adaptor3d_HVertex"));
}

#[test]
fn trait_handles_dispatch_to_the_overrides() {
    let fake: Handle<dyn Adaptor3dHVertex> = Handle::new(FakeOverridingVertex {
        parent: StandardTransient::default(),
    })
    .cast();
    assert_eq!(fake.value(), Point2::new(7.0, 8.0));
    assert_eq!(fake.parameter(&line()).unwrap(), 42.0);
    assert!(fake.resolution(&line()).is_err());
    assert!(fake.is_kind_name("Adaptor3d_HVertex"));
    assert!(fake.is_instance(FakeOverridingVertex::static_type()));
    // The base IsSame reads the other vertex through its own (overridden) Value.
    let base: Handle<dyn Adaptor3dHVertex> = Handle::new(Adaptor3dHVertexBase::new(
        Point2::new(7.0, 8.0),
        Orientation::Forward,
        0.0,
    ))
    .cast();
    assert!(base.is_same(&fake).unwrap());
    // Adaptor3d_TopolTool::Orientation(V) and Identical call the vertex members.
    let tool = Adaptor3dTopolTool::default();
    assert_eq!(
        tool.vertex_orientation(&fake).unwrap(),
        Orientation::Internal
    );
    assert!(!tool.identical(&fake, &base).unwrap());
    assert!(tool.identical(&base, &fake).unwrap());
}

#[test]
fn sample_pnts_body_spreads_the_counts_of_the_given_computation() {
    let surface = plane_adaptor();
    let (u, v) =
        Adaptor3dTopolTool::sample_pnts_parameters(&*surface, (0.01, 2, 2), || Ok((3, 5))).unwrap();
    assert_eq!((u.length(), v.length()), (3, 5));
    // The plane adaptor spans +-Precision::Infinite(), which is not RealFirst/RealLast,
    // so the box keeps it (natively too): the parameters spread over [-2e100, 2e100].
    assert_eq!(
        (*u.value(1).unwrap(), *u.value(3).unwrap()),
        (-2.0e100, 2.0e100)
    );
    assert_eq!(*v.value(2).unwrap(), -2.0e100 + 1.0e100);
    let failing = Adaptor3dTopolTool::sample_pnts_parameters(&*surface, (0.01, 2, 2), || {
        Err(Failure::with_message("Standard_NullObject", "no counts"))
    });
    assert!(failing.is_err());
    assert!(Adaptor3dTopolTool::uniform_sampling_of(&*surface).unwrap());
    // The base tool passes its own counts: 6 x 6 for a plane.
    let tool = Adaptor3dTopolTool::with_surface(&surface).unwrap();
    tool.sample_pnts(0.01, 2, 2).unwrap();
    assert_eq!(tool.u_parameters().unwrap().length(), 6);
    assert_eq!(tool.nb_samples_v().unwrap(), 6);
}
