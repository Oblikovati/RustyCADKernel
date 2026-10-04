//! Runtime descriptor and handle projection for `BSplSLib_Cache`.
use rustycad_tkernel::{
    ncollection::Array1,
    standard::{Handle, RuntimeTyped, StandardTransient},
};
use rustycad_tkmath::bsplslib::Cache;

#[test]
fn surface_cache_handle_projects_to_standard_transient() {
    let knots: Array1<'static, f64> = flat_knots();
    let cache: Cache = Cache::new(1, false, &knots, 1, false, &knots, None).unwrap();
    let handle: Handle<Cache> = Handle::new(cache);

    assert_eq!(Cache::static_type().name(), "BSplSLib_Cache");
    assert_eq!(
        Cache::static_type().parent().unwrap().name(),
        "Standard_Transient"
    );
    assert!(handle.is_instance_name("BSplSLib_Cache"));
    assert!(handle.is_kind_name("Standard_Transient"));
    assert_eq!(handle.ref_count(), 1);
    assert_base_projection(&handle);
}

fn assert_base_projection(handle: &Handle<Cache>) {
    let base: Handle<StandardTransient> = handle.cast();
    assert!(base.is_instance_name("BSplSLib_Cache"));
    assert!(base.is_kind_name("Standard_Transient"));
    assert_eq!(base.object_address(), handle.object_address());
    assert_eq!(base.ref_count(), 2);
}

fn flat_knots() -> Array1<'static, f64> {
    let mut knots: Array1<'static, f64> = Array1::with_bounds(1, 4).unwrap();
    knots.as_mut_slice().copy_from_slice(&[0.0, 0.0, 1.0, 1.0]);
    knots
}
