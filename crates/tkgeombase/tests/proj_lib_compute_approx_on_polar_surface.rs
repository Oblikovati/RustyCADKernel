// Copyright (c) 2026 OPEN CASCADE SAS
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

//! Direct translation of ProjLib_ComputeApproxOnPolarSurface_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve, GeomCylindricalSurface, GeomSurface};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkgeombase::proj_lib::ComputeApproxOnPolarSurface;
use rustycad_tkmath::gp::{Axis2, Axis3, Direction3, Point2, Point3};

fn checkProjectionAtParam(
    thePCurve: &Geom2dBSplineCurve,
    theSurface: &GeomCylindricalSurface,
    theCurve: &GeomCircle,
    theParam: f64,
    theTol: f64,
) {
    let aUV: Point2 = thePCurve.value(theParam).unwrap();
    let aProjP: Point3 = theSurface.value(aUV.x(), aUV.y()).unwrap();
    let aRefP: Point3 = theCurve.value(theParam).unwrap();
    assert!(aProjP.distance(&aRefP) <= theTol);
}

#[test]
fn ProjLib_ComputeApproxOnPolarSurfaceTest__BuildInitialCurveAndProjectOnCylinder_Done() {
    let z = Direction3::new(0.0, 0.0, 1.0).unwrap();
    let x = Direction3::new(1.0, 0.0, 0.0).unwrap();
    let aCurve = Handle::new(
        GeomCircle::new(
            Axis2::with_x_direction(Point3::new(0.0, 0.0, 2.0), z, x).unwrap(),
            5.0,
        )
        .unwrap(),
    );
    let aSurface = Handle::new(
        GeomCylindricalSurface::new(Axis3::new(Point3::new(0.0, 0.0, 0.0), z, x).unwrap(), 5.0)
            .unwrap(),
    );

    let aCurveAdaptor: Handle<dyn Adaptor3dCurve> =
        Handle::new(GeomAdaptorCurve::with_curve(&aCurve.clone().cast::<dyn GeomCurve>()).unwrap())
            .cast();
    let aSurfAdaptor: Handle<dyn Adaptor3dSurface> = Handle::new(
        GeomAdaptorSurface::with_surface(&aSurface.clone().cast::<dyn GeomSurface>()).unwrap(),
    )
    .cast();

    let mut aProjector = ComputeApproxOnPolarSurface::new();
    aProjector.set_tolerance(1.0e-6);

    let anInitCurve2d = aProjector
        .build_initial_curve2d(&aCurveAdaptor, &aSurfAdaptor)
        .unwrap();
    assert!(aProjector.is_done());
    assert!(!anInitCurve2d.is_null());

    let aPCurve = aProjector
        .project_using_initial_curve2d(&aCurveAdaptor, &aSurfAdaptor, &anInitCurve2d)
        .unwrap();
    assert!(aProjector.is_done());
    assert!(!aPCurve.is_null());
    let aPCurve = aPCurve.get().unwrap();

    let aFirst: f64 = aCurve.first_parameter().max(aPCurve.first_parameter());
    let aLast: f64 = aCurve.last_parameter().min(aPCurve.last_parameter());
    assert!(aFirst < aLast);

    checkProjectionAtParam(aPCurve, &aSurface, &aCurve, aFirst, 1.0e-3);
    checkProjectionAtParam(aPCurve, &aSurface, &aCurve, 0.5 * (aFirst + aLast), 1.0e-3);
    checkProjectionAtParam(aPCurve, &aSurface, &aCurve, aLast, 1.0e-3);
}
