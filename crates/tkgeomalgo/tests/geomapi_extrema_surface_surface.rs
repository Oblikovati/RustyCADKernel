//! Direct translation of GeomAPI_ExtremaSurfaceSurface_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkgeomalgo::geom_api::ExtremaSurfaceSurface;
use rustycad_tkmath::gp::{Direction3, Plane3, Point3};

fn plane(origin: Point3) -> Handle<dyn GeomSurface> {
    let pln = Plane3::new(origin, Direction3::new(0.0, 0.0, 1.0).unwrap()).unwrap();
    Handle::new(GeomPlane::from_pln(pln)).cast()
}

// Migrated from tests/lowalgos/extss/bug29712_44. The DRAW script contains a
// same-object typo in both extrema calls; its expected distance of 10.0 clearly
// describes the two parallel planes created immediately above, so this test
// preserves the intended regression check.
#[test]
fn GeomAPI_ExtremaSurfaceSurfaceTest__OCC29712_44_ParallelPlanes() {
    let aPlane1 = plane(Point3::new(0.0, 0.0, 0.0));
    let aPlane2 = plane(Point3::new(0.0, 0.0, 10.0));

    let anExtrema = ExtremaSurfaceSurface::with_surfaces(&aPlane1, &aPlane2).unwrap();
    assert!(anExtrema.extrema().unwrap().is_parallel().unwrap());
    assert!((anExtrema.lower_distance().unwrap() - 10.0).abs() <= 1.0e-7);
}

// Extra validation for the source typo: a plane compared with itself must have
// zero lower distance, rather than the distance between s1 and s2.
#[test]
fn GeomAPI_ExtremaSurfaceSurfaceTest__OCC29712_44_SelfComparisonHasZeroDistance() {
    let aPlane = plane(Point3::new(0.0, 0.0, 0.0));

    let anExtrema = ExtremaSurfaceSurface::with_surfaces(&aPlane, &aPlane).unwrap();
    assert!(anExtrema.extrema().unwrap().is_parallel().unwrap());
    assert!((anExtrema.lower_distance().unwrap() - 0.0).abs() <= 1.0e-7);
}
