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

//! Direct translation of Geom2dGcc_Lin2d2Tan_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg2d::geom2d_adaptor::{Geom2dAdaptorCurve, make_curve};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve, GeomEllipse, GeomPlane, GeomSurface};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkgeomalgo::gcc_ent::Position;
use rustycad_tkgeomalgo::geom2d_gcc::{Lin2d2Tan as Geom2dGcc_Lin2d2Tan, QualifiedCurve};
use rustycad_tkgeombase::proj_lib::ProjectedCurve;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Axis2, Direction3, Plane3, Point2, Point3};

// GeomAPI::To2d(C, P), which the suite calls: the projection of the curve on the
// plane (ProjLib_ProjectedCurve) as a Geom2d curve (Geom2dAdaptor::MakeCurve), a
// null curve for an offset or other projection. GeomAPI is not ported yet, so the
// suite carries its five-line body.
fn GeomAPI_To2d(C: &Handle<dyn GeomCurve>, P: &Plane3) -> Handle<dyn Geom2dCurve> {
    let HC: Handle<dyn Adaptor3dCurve> =
        Handle::new(GeomAdaptorCurve::with_curve(C).unwrap()).cast();
    let Plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::from_pln(*P)).cast();
    let HS: Handle<dyn Adaptor3dSurface> =
        Handle::new(GeomAdaptorSurface::with_surface(&Plane).unwrap()).cast();
    let Proj = ProjectedCurve::with_curve(&HS, &HC).unwrap();
    let aType: CurveType = Proj.get_type().unwrap();
    if aType == CurveType::OffsetCurve || aType == CurveType::OtherCurve {
        return Handle::default();
    }
    make_curve(&Proj).unwrap()
}

fn gp_Ax2(P: Point3, N: (f64, f64, f64), Vx: (f64, f64, f64)) -> Axis2 {
    Axis2::with_x_direction(
        P,
        Direction3::new(N.0, N.1, N.2).unwrap(),
        Direction3::new(Vx.0, Vx.1, Vx.2).unwrap(),
    )
    .unwrap()
}

// Test OCC813: Geom2dGcc_Lin2d2Tan - tangent line between a 2D ellipse and a point.
// Migrated from QABugs_17.cxx OCC813
#[test]
fn Geom2dGcc_Lin2d2TanTest__OCC813_EllipseAndPoint() {
    // Construct 3D ellipse and projection plane
    let anAx2 = gp_Ax2(
        Point3::new(1262.224429, 425.040878, 363.609716),
        (0.173648, 0.984808, 0.000000),
        (-0.932169, 0.164367, -0.322560),
    );

    let anEll: Handle<dyn GeomCurve> =
        Handle::new(GeomEllipse::new(anAx2, 150.0, 100.0).unwrap()).cast();
    let aPlane = GeomPlane::new(anAx2.axis3());
    let aPln: Plane3 = aPlane.pln();

    // Project ellipse onto the plane and build 2D qualified curve
    let aCurve2d = GeomAPI_To2d(&anEll, &aPln);
    let anAdapt = Geom2dAdaptorCurve::with_curve(&aCurve2d).unwrap();
    let aQCurve = QualifiedCurve::new(&anAdapt, Position::Outside).unwrap();

    // Query tangent line from 2D point to the projected ellipse
    let aPnt2d = Point2::new(200.0, 200.0);
    let aLinTan = Geom2dGcc_Lin2d2Tan::through_point(&aQCurve, aPnt2d, 0.1).unwrap();

    assert!(
        aLinTan.nb_solutions() > 0,
        "Expected at least one tangent line solution"
    );
}

// Test OCC814: Geom2dGcc_Lin2d2Tan - tangent line between a 2D circle and a 2D ellipse.
// Migrated from QABugs_17.cxx OCC814
#[test]
fn Geom2dGcc_Lin2d2TanTest__OCC814_CircleAndEllipse() {
    let anAx2 = gp_Ax2(
        Point3::new(1262.224429, 425.040878, 363.609716),
        (0.173648, 0.984808, 0.000000),
        (-0.932169, 0.164367, -0.322560),
    );

    let aCir: Handle<dyn GeomCurve> = Handle::new(
        GeomCircle::new(
            gp_Ax2(
                Point3::new(823.687192, 502.366825, 478.960440),
                (0.173648, 0.984808, 0.000000),
                (-0.932169, 0.164367, -0.322560),
            ),
            50.0,
        )
        .unwrap(),
    )
    .cast();
    let anEll: Handle<dyn GeomCurve> =
        Handle::new(GeomEllipse::new(anAx2, 150.0, 100.0).unwrap()).cast();
    let aPlane = GeomPlane::new(anAx2.axis3());
    let aPln: Plane3 = aPlane.pln();

    // Project both curves onto the plane and build qualified curves
    let aCurve2d = GeomAPI_To2d(&anEll, &aPln);
    let aFromCurve2d = GeomAPI_To2d(&aCir, &aPln);
    let anAdaptEll = Geom2dAdaptorCurve::with_curve(&aCurve2d).unwrap();
    let anAdaptCir = Geom2dAdaptorCurve::with_curve(&aFromCurve2d).unwrap();
    let aQEll = QualifiedCurve::new(&anAdaptEll, Position::Outside).unwrap();
    let aQCir = QualifiedCurve::new(&anAdaptCir, Position::Outside).unwrap();

    let aLinTan = Geom2dGcc_Lin2d2Tan::new(&aQEll, &aQCir, 0.1).unwrap();

    assert!(
        aLinTan.nb_solutions() > 0,
        "Expected at least one tangent line solution"
    );
}
