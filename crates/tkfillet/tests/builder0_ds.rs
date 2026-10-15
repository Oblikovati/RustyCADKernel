//! The ChFi3d_Builder_0 helpers on the TopOpeBRepDS data structure: the interferences of
//! the fillet curves and points, the points of the common points, the geometry of the
//! indices and the tolerance of the points.
use rustycad_tkbool::top_ope_brep_ds::{TopOpeBRepDSDataStructure, TopOpeBRepDSKind};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::Handle;
use rustycad_tkfillet::chfi_ds::{CommonPoint, Stripe, SurfData, lock, shared};
use rustycad_tkfillet::chfi3d as b0;
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Axis3, Point3};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;

#[test]
fn the_fillet_interferences_carry_their_kinds() {
    let i = b0::fil_point_in_ds(Orientation::Forward, 2, 3, 0.5, false);
    let (gk, g, sk, s) = i.get().unwrap().gkgsks();
    assert_eq!((gk, g, sk, s), (TopOpeBRepDSKind::Point, 3, TopOpeBRepDSKind::Curve, 2));
    let v = b0::fil_vertex_in_ds(Orientation::Reversed, 4, 1, 0.0);
    assert_eq!(v.get().unwrap().geometry_type(), TopOpeBRepDSKind::Vertex);
    let c = b0::fil_curve_in_ds(5, 6, &Handle::default(), Orientation::Forward);
    let (gk, g, sk, s) = c.get().unwrap().gkgsks();
    assert_eq!((gk, g, sk, s), (TopOpeBRepDSKind::Curve, 5, TopOpeBRepDSKind::Surface, 6));
}

#[test]
fn a_common_point_off_a_vertex_becomes_a_point() {
    let mut ds = TopOpeBRepDSDataStructure::new();
    let mut cp = CommonPoint::new();
    cp.set_point(Point3::new(1., 2., 3.));
    assert_eq!(b0::index_point_in_ds(&cp, &mut ds).unwrap(), 1);
    assert_eq!(ds.point(1).unwrap().point(), Point3::new(1., 2., 3.));
    let mut bx = BoundingBox3::default();
    bx.add_point(Point3::default());
    bx.add_point(Point3::new(1., 2., 2.));
    b0::set_point_tolerance(&mut ds, &bx, 1).unwrap();
    let [a, b, c, d, e, f] = bx.get().unwrap();
    let diagonal = ((d - a).powi(2) + (e - b).powi(2) + (f - c).powi(2)).sqrt();
    assert!((ds.point(1).unwrap().tolerance() - 1.5 * diagonal).abs() < 1e-12);
}

#[test]
fn the_geometry_of_an_index_is_a_face_or_a_surface() {
    let mut ds = TopOpeBRepDSDataStructure::new();
    assert!(b0::ds_geometry(&ds, 0).unwrap().is_null());
    let solid: Shape = BRepPrimAPIMakeBox::from_sizes(1., 1., 1.).unwrap().shape().unwrap().clone();
    let face = Explorer::new(&solid, ShapeEnum::Face, ShapeEnum::Shape).unwrap().current().clone();
    let iface = ds.add_shape(&face);
    let s = b0::ds_geometry(&ds, iface).unwrap();
    assert_eq!(s.get().unwrap().get_type().unwrap(), SurfaceType::Plane);
    let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    let isurf = rustycad_tkfillet::chfikpart::index_surface_in_ds(&plane, &mut ds);
    let g = b0::ds_geometry(&ds, -isurf).unwrap();
    assert_eq!(g.get().unwrap().get_type().unwrap(), SurfaceType::Plane);
}

#[test]
fn the_surface_data_of_a_stripe_by_index() {
    let st = shared(Stripe::new());
    let sd = shared(SurfData::new());
    *lock(&st).change_set_of_surf_data() = Some(shared(vec![sd.clone()]));
    assert!(std::sync::Arc::ptr_eq(&b0::surf_data_at(&st, 1).unwrap(), &sd));
    assert!(b0::surf_data_at(&st, 2).is_err());
}

#[test]
fn a_solid_index_needs_a_spine() {
    let mut ds = TopOpeBRepDSDataStructure::new();
    let empty = Default::default();
    let err = b0::solid_index(None, &mut ds, &empty, &empty).unwrap_err();
    assert_eq!(err.message_bytes(), b"SolidIndex : Spine incomplete");
}
