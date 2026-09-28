// Copyright (c) 2025 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! Direct translation of GeomAdaptor_TransformedSurface_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::EXPECT_NEAR;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{
    GeomBSplineSurface, GeomCurve, GeomLine, GeomOffsetSurface, GeomPlane, GeomSphericalSurface,
    GeomSurface, GeomSurfaceOfLinearExtrusion, GeomSurfaceOfRevolution,
};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorSurface, GeomAdaptorTransformedSurface};
use rustycad_tkg3d::geom_grid_eval::GeomGridEvalSurface;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::{self, Axis1, Axis3, Direction3, Plane3, Point3, Transform3, Vector3};
use std::f64::consts::PI;

const THE_TOLERANCE: f64 = 1.0e-10;

fn createTranslation(theDx: f64, theDy: f64, theDz: f64) -> Transform3 {
    let mut aTrsf = Transform3::default();
    aTrsf.set_translation(Vector3::new(theDx, theDy, theDz));
    aTrsf
}

fn createRotationY(theAngle: f64) -> Transform3 {
    let mut aTrsf = Transform3::default();
    let anAxis = Axis1::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 1.0, 0.0).unwrap(),
    );
    aTrsf.set_rotation(anAxis, theAngle).unwrap();
    aTrsf
}

fn createBSplineSurface() -> Handle<GeomBSplineSurface> {
    let mut aPoles: Array2<'static, Point3> = Array2::with_bounds(1, 2, 1, 2).unwrap();
    aPoles[(1, 1)] = Point3::new(0.0, 0.0, 0.0);
    aPoles[(2, 1)] = Point3::new(1.0, 0.0, 0.0);
    aPoles[(1, 2)] = Point3::new(0.0, 1.0, 0.0);
    aPoles[(2, 2)] = Point3::new(1.0, 1.0, 1.0);

    let mut aUKnots: Array1<'static, f64> = Array1::with_bounds(1, 2).unwrap();
    let mut aVKnots: Array1<'static, f64> = Array1::with_bounds(1, 2).unwrap();
    let mut aUMults: Array1<'static, i32> = Array1::with_bounds(1, 2).unwrap();
    let mut aVMults: Array1<'static, i32> = Array1::with_bounds(1, 2).unwrap();
    aUKnots[1] = 0.0;
    aUKnots[2] = 1.0;
    aVKnots[1] = 0.0;
    aVKnots[2] = 1.0;
    aUMults[1] = 2;
    aUMults[2] = 2;
    aVMults[1] = 2;
    aVMults[2] = 2;

    Handle::new(
        GeomBSplineSurface::new(
            &aPoles,
            (&aUKnots, &aUMults),
            (&aVKnots, &aVMults),
            (1, 1),
            (false, false),
        )
        .unwrap(),
    )
}

// `new Geom_Plane(gp_Pln(gp::XOY()))`, as a `Geom_Surface` handle.
fn PlaneXOY() -> Handle<dyn GeomSurface> {
    let aPln = Plane3::from_position(Axis3::from_axis2(gp::xoy()));
    Handle::new(GeomPlane::from_pln(aPln)).cast()
}

fn Transformed(
    theSurface: &Handle<dyn GeomSurface>,
    theTrsf: Transform3,
) -> GeomAdaptorTransformedSurface {
    GeomAdaptorTransformedSurface::with_surface(theSurface, theTrsf).unwrap()
}

// `occ::down_cast<GeomAdaptor_TransformedSurface>(anAdaptor.ShallowCopy())`.
fn TransformedCopy(
    theAdaptor: &GeomAdaptorTransformedSurface,
) -> Handle<GeomAdaptorTransformedSurface> {
    theAdaptor.shallow_copy().unwrap().cast()
}

//=================================================================================================

#[test]
fn GeomAdaptor_TransformedSurfaceTest__IdentityTransformUsesOriginalSurface() {
    let aPlane = PlaneXOY();
    let anAdaptor = Transformed(&aPlane, Transform3::default());

    assert!(anAdaptor.geom_surface_original() == aPlane);
    assert!(anAdaptor.geom_surface_transformed().unwrap() == aPlane);
}

//=================================================================================================

#[test]
fn GeomAdaptor_TransformedSurfaceTest__PlaneCachesTransformedSurface() {
    let aPlane = PlaneXOY();
    let anAdaptor = Transformed(&aPlane, createTranslation(0.0, 0.0, 5.0));

    let aFirst = anAdaptor.geom_surface_transformed().unwrap();
    let aSecond = anAdaptor.geom_surface_transformed().unwrap();
    assert!(aFirst == aSecond);

    let aCachedPlane: Plane3 = anAdaptor.plane().unwrap();
    EXPECT_NEAR(aCachedPlane.location().z(), 5.0, THE_TOLERANCE);
}

//=================================================================================================

#[test]
fn GeomAdaptor_TransformedSurfaceTest__BSplineCachePreservedByShallowCopy() {
    let anAdaptor = Transformed(
        &createBSplineSurface().cast(),
        createTranslation(4.0, 0.0, 0.0),
    );

    let aSurface = anAdaptor.geom_surface_transformed().unwrap();
    let aBSpline: Handle<GeomBSplineSurface> = anAdaptor.bspline().unwrap();

    let aCopy = TransformedCopy(&anAdaptor);
    assert!(!aCopy.is_null());

    assert!(aCopy.geom_surface_transformed().unwrap() == aSurface);
    assert!(aCopy.bspline().unwrap() == aBSpline);
}

//=================================================================================================

#[test]
fn GeomAdaptor_TransformedSurfaceTest__ShallowCopyFromUnbuiltStateBuildsCacheLazily() {
    let aPlane = PlaneXOY();
    let anAdaptor = Transformed(&aPlane, createTranslation(0.0, 0.0, 4.0));

    let aCopy = TransformedCopy(&anAdaptor);
    assert!(!aCopy.is_null());

    let aFirst = aCopy.geom_surface_transformed().unwrap();
    let aSecond = aCopy.geom_surface_transformed().unwrap();
    assert!(aFirst == aSecond);
    EXPECT_NEAR(aCopy.plane().unwrap().location().z(), 4.0, THE_TOLERANCE);
}

//=================================================================================================

#[test]
fn GeomAdaptor_TransformedSurfaceTest__SetTrsfRebuildsCache() {
    let aPlane = PlaneXOY();
    let anAdaptor = Transformed(&aPlane, createTranslation(0.0, 0.0, 1.0));

    let aFirst = anAdaptor.geom_surface_transformed().unwrap();
    anAdaptor.set_trsf(createTranslation(0.0, 0.0, 3.0));
    let aSecond = anAdaptor.geom_surface_transformed().unwrap();

    assert!(aFirst != aSecond);
    EXPECT_NEAR(
        anAdaptor.plane().unwrap().location().z(),
        3.0,
        THE_TOLERANCE,
    );
}

//=================================================================================================

#[test]
fn GeomAdaptor_TransformedSurfaceTest__LoadRebuildsCache() {
    let aPlane = PlaneXOY();
    let anAdaptor = Transformed(&aPlane, createTranslation(0.0, 0.0, 2.0));

    let aFirst = anAdaptor.geom_surface_transformed().unwrap();

    let aSphereAx = Axis3::from_normal(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let aSphere: Handle<dyn GeomSurface> =
        Handle::new(GeomSphericalSurface::new(aSphereAx, 3.0).unwrap()).cast();
    anAdaptor.load(&aSphere, Transform3::default()).unwrap();

    let aSecond = anAdaptor.geom_surface_transformed().unwrap();
    assert!(aFirst != aSecond);
    assert_eq!(anAdaptor.get_type().unwrap(), SurfaceType::Sphere);
    EXPECT_NEAR(anAdaptor.sphere().unwrap().radius(), 3.0, THE_TOLERANCE);
}

//=================================================================================================

#[test]
fn GeomAdaptor_TransformedSurfaceTest__ExtrusionCachesDirectionAndBasisCurve() {
    let aLine: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_point(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(1.0, 0.0, 0.0).unwrap(),
    ))
    .cast();
    let anExt: Handle<dyn GeomSurface> = Handle::new(
        GeomSurfaceOfLinearExtrusion::new(&aLine, Direction3::new(0.0, 0.0, 1.0).unwrap()).unwrap(),
    )
    .cast();
    let anAdaptor = Transformed(&anExt, createRotationY(PI / 2.0));

    let aDirection: Direction3 = anAdaptor.direction().unwrap();
    EXPECT_NEAR(aDirection.x(), 1.0, THE_TOLERANCE);
    EXPECT_NEAR(aDirection.y(), 0.0, THE_TOLERANCE);
    EXPECT_NEAR(aDirection.z(), 0.0, THE_TOLERANCE);

    let aCurve: Handle<dyn Adaptor3dCurve> = anAdaptor.basis_curve().unwrap();
    assert!(!aCurve.is_null());
    assert_eq!(aCurve.get_type().unwrap(), CurveType::Line);
}

//=================================================================================================

#[test]
fn GeomAdaptor_TransformedSurfaceTest__RevolutionCachesAxisAndBasisCurve() {
    let aLine: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_point(
        Point3::new(2.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    ))
    .cast();
    let anAxis = Axis1::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    );
    let aRev: Handle<dyn GeomSurface> =
        Handle::new(GeomSurfaceOfRevolution::new(&aLine, anAxis).unwrap()).cast();
    let anAdaptor = Transformed(&aRev, createRotationY(PI / 2.0));

    let aAxis: Axis1 = anAdaptor.axe_of_revolution().unwrap();
    EXPECT_NEAR(aAxis.direction().x(), 1.0, THE_TOLERANCE);
    EXPECT_NEAR(aAxis.direction().y(), 0.0, THE_TOLERANCE);
    EXPECT_NEAR(aAxis.direction().z(), 0.0, THE_TOLERANCE);

    let aCurve: Handle<dyn Adaptor3dCurve> = anAdaptor.basis_curve().unwrap();
    assert!(!aCurve.is_null());
    assert_eq!(aCurve.get_type().unwrap(), CurveType::Line);
}

//=================================================================================================

#[test]
fn GeomAdaptor_TransformedSurfaceTest__OffsetCachesBasisSurfaceAndOffsetValue() {
    let aPlane = PlaneXOY();
    let anOffset: Handle<dyn GeomSurface> =
        Handle::new(GeomOffsetSurface::new(&aPlane, 2.5, false).unwrap()).cast();
    let anAdaptor = Transformed(&anOffset, createTranslation(0.0, 0.0, 3.0));

    let aBasis: Handle<dyn Adaptor3dSurface> = anAdaptor.basis_surface().unwrap();
    assert!(!aBasis.is_null());
    assert_eq!(aBasis.get_type().unwrap(), SurfaceType::Plane);
    EXPECT_NEAR(anAdaptor.offset_value().unwrap(), 2.5, THE_TOLERANCE);
}

//=================================================================================================

// `GeomAdaptor_TransformedSurface(aPlane, 2.0, 8.0, 3.0, 9.0, theTrsf, 0.1, 0.2)`.
fn RangedPlaneAdaptor(theTrsf: Transform3) -> GeomAdaptorTransformedSurface {
    let aRange = ((2.0, 8.0, 3.0, 9.0), (0.1, 0.2));
    GeomAdaptorTransformedSurface::with_range(&PlaneXOY(), aRange, theTrsf).unwrap()
}

#[test]
fn GeomAdaptor_TransformedSurfaceTest__UTrimPreservesRestrictedBounds() {
    let anAdaptor = RangedPlaneAdaptor(createTranslation(0.0, 0.0, 5.0));

    let aTrimmedBase = anAdaptor.u_trim(4.0, 6.0, 0.05).unwrap();
    let aTrimmed: Handle<GeomAdaptorSurface> = aTrimmedBase.cast();
    assert!(!aTrimmed.is_null());

    EXPECT_NEAR(aTrimmed.first_u_parameter().unwrap(), 4.0, THE_TOLERANCE);
    EXPECT_NEAR(aTrimmed.last_u_parameter().unwrap(), 6.0, THE_TOLERANCE);
    EXPECT_NEAR(aTrimmed.first_v_parameter().unwrap(), 3.0, THE_TOLERANCE);
    EXPECT_NEAR(aTrimmed.last_v_parameter().unwrap(), 9.0, THE_TOLERANCE);
    EXPECT_NEAR(aTrimmed.tolerance_u(), 0.05, THE_TOLERANCE);
    EXPECT_NEAR(aTrimmed.tolerance_v(), 0.2, THE_TOLERANCE);

    let aPoint: Point3 = aTrimmed.eval_d0(4.0, 3.0).unwrap();
    EXPECT_NEAR(aPoint.z(), 5.0, THE_TOLERANCE);
}

//=================================================================================================

#[test]
fn GeomAdaptor_TransformedSurfaceTest__VTrimPreservesRestrictedBounds() {
    let anAdaptor = RangedPlaneAdaptor(createTranslation(0.0, 0.0, 7.0));

    let aTrimmedBase = anAdaptor.v_trim(4.0, 6.0, 0.15).unwrap();
    let aTrimmed: Handle<GeomAdaptorSurface> = aTrimmedBase.cast();
    assert!(!aTrimmed.is_null());

    EXPECT_NEAR(aTrimmed.first_u_parameter().unwrap(), 2.0, THE_TOLERANCE);
    EXPECT_NEAR(aTrimmed.last_u_parameter().unwrap(), 8.0, THE_TOLERANCE);
    EXPECT_NEAR(aTrimmed.first_v_parameter().unwrap(), 4.0, THE_TOLERANCE);
    EXPECT_NEAR(aTrimmed.last_v_parameter().unwrap(), 6.0, THE_TOLERANCE);
    EXPECT_NEAR(aTrimmed.tolerance_u(), 0.1, THE_TOLERANCE);
    EXPECT_NEAR(aTrimmed.tolerance_v(), 0.15, THE_TOLERANCE);

    let aPoint: Point3 = aTrimmed.eval_d0(2.0, 4.0).unwrap();
    EXPECT_NEAR(aPoint.z(), 7.0, THE_TOLERANCE);
}

//=================================================================================================

#[test]
fn GeomAdaptor_TransformedSurfaceTest__GridEvalUsesTransformedGeometryOnlyOnce() {
    let aPlane = PlaneXOY();
    let anAdaptor = Transformed(&aPlane, createTranslation(0.0, 0.0, 5.0));

    let anEval = GeomGridEvalSurface::from_adaptor(&anAdaptor).unwrap();

    let mut aUParams: Array1<'static, f64> = Array1::with_bounds(1, 2).unwrap();
    let mut aVParams: Array1<'static, f64> = Array1::with_bounds(1, 2).unwrap();
    aUParams[1] = 0.0;
    aUParams[2] = 1.0;
    aVParams[1] = 0.0;
    aVParams[2] = 1.0;

    let aGrid: Array2<Point3> = anEval.evaluate_grid(&aUParams, &aVParams).unwrap();
    EXPECT_NEAR(aGrid[(1, 1)].z(), 5.0, THE_TOLERANCE);
    EXPECT_NEAR(aGrid[(2, 2)].z(), 5.0, THE_TOLERANCE);
}
