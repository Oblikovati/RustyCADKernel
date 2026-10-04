//! Protected cache transitions matched against a derived native surface adaptor.
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorSurface, GeomAdaptorTransformedSurface};
use rustycad_tkmath::gp::{self, Axis3, Plane3, Transform3, Vector3};

type CacheAdaptor = GeomAdaptorTransformedSurface;

fn empty_receipt() -> String {
    let adaptor = CacheAdaptor::default();
    adaptor.ensure_transformed_cache().unwrap();
    let absent = adaptor.cached_transformed_surface().is_some();
    adaptor.init_transformed_cache().unwrap();
    let cached = adaptor.cached_transformed_surface().unwrap();
    format!(
        "empty {} 1 {}\n",
        i32::from(absent),
        i32::from(cached.is_null())
    )
}

fn refresh_receipt(adaptor: &CacheAdaptor, surface: &Handle<dyn GeomSurface>) -> String {
    adaptor.ensure_transformed_cache().unwrap();
    let cached = adaptor.cached_transformed_surface().unwrap();
    adaptor.ensure_transformed_cache().unwrap();
    let same = cached == adaptor.cached_transformed_surface().unwrap();
    surface.translate(Vector3::new(1.0, 0.0, 0.0)).unwrap();
    let stale = plane_x(adaptor);
    adaptor.init_transformed_cache().unwrap();
    let changed = cached != adaptor.cached_transformed_surface().unwrap();
    let rebuilt = plane_x(adaptor);
    let receipt = format!(
        "repeat {}\nstale {stale}\nrebuilt {} {rebuilt}\n",
        i32::from(same),
        i32::from(changed)
    );
    format!("{receipt}{}", invalidation_receipt(adaptor))
}

fn plane_x(adaptor: &CacheAdaptor) -> f64 {
    adaptor
        .transformed_adaptor()
        .unwrap()
        .plane()
        .unwrap()
        .location()
        .x()
}

fn invalidation_receipt(adaptor: &CacheAdaptor) -> String {
    adaptor.invalidate_transformed_cache();
    let present = adaptor.cached_transformed_surface().is_some();
    adaptor.ensure_transformed_cache().unwrap();
    format!(
        "invalid {}\nensured {}\n",
        i32::from(present),
        plane_x(adaptor)
    )
}

fn identity_receipt(adaptor: &CacheAdaptor, surface: &Handle<dyn GeomSurface>) -> String {
    adaptor.set_trsf(Transform3::default());
    adaptor.ensure_transformed_cache().unwrap();
    let absent = adaptor.cached_transformed_surface().is_some();
    adaptor.init_transformed_cache().unwrap();
    let cached = adaptor.cached_transformed_surface().unwrap();
    let original = adaptor.transformed_adaptor().unwrap();
    let same_adaptor = original == adaptor.adaptor_surface_original();
    let flags = format!(
        "identity {} 1 {} {}\n",
        i32::from(absent),
        i32::from(cached == *surface),
        i32::from(same_adaptor)
    );
    format!("{flags}{}", bounds_receipt(&original))
}

fn bounds_receipt(original: &GeomAdaptorSurface) -> String {
    let (u1, u2, v1, v2) = original.bounds().unwrap();
    format!(
        "range {u1} {u2} {v1} {v2} {} {}\n",
        original.tolerance_u(),
        original.tolerance_v()
    )
}

fn configured_adaptor() -> (Handle<dyn GeomSurface>, CacheAdaptor) {
    let plane = Plane3::from_position(Axis3::from_axis2(gp::xoy()));
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::from_pln(plane)).cast();
    let mut shift = Transform3::default();
    shift.set_translation(Vector3::new(10.0, 0.0, 0.0));
    let adaptor =
        CacheAdaptor::with_range(&surface, ((1.0, 3.0, 2.0, 4.0), (0.01, 0.02)), shift).unwrap();
    (surface, adaptor)
}

#[test]
fn protected_cache_operations_match_native_identity_and_refresh_semantics() {
    let (surface, adaptor) = configured_adaptor();
    let observed = format!(
        "{}{}{}",
        empty_receipt(),
        refresh_receipt(&adaptor, &surface),
        identity_receipt(&adaptor, &surface)
    );
    assert_eq!(
        observed,
        include_str!("../../../tools/oracle/transformed_surface_cache.expected")
    );
}
