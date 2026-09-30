//! 1:1 translation of src/ModelingAlgorithms/TKPrim/GTests/BRepPrimAPI_MakeTorus_Test.cxx.
#![allow(non_snake_case)]

use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkg3d::geom::GeomToroidalSurface;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Axis2, Direction3, Point3};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeTorus;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;
use std::f64::consts::{FRAC_PI_2, PI};

/// The face-counting `TopExp_Explorer` loops of the suite.
fn face_count(shape: &Shape) -> i32 {
    let mut count = 0;
    let mut explorer = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    while explorer.more() {
        count += 1;
        explorer.next().unwrap();
    }
    count
}

/// `BRepCheck_Analyzer(aShape).IsValid()`.
fn is_valid(shape: &Shape) -> bool {
    BRepCheckAnalyzer::new(shape, true, false, false)
        .unwrap()
        .is_valid()
        .unwrap()
}

/// `gp_Ax2(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1))`.
fn z_axis() -> Axis2 {
    Axis2::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap()
}

/// Helper function to create partial torus and check validity.
/// @param theR1 Major radius
/// @param theR2 Minor radius
/// @param theAngle1 First angle on the minor circle (V parameter)
/// @param theAngle2 Second angle on the minor circle (V parameter)
/// @param theAngle Angle around the major circle (U parameter), default 2*PI
/// @return true if the resulting shape is valid
fn create_and_check_partial_torus(
    the_r1: f64,
    the_r2: f64,
    the_angle1: f64,
    the_angle2: f64,
    the_angle: f64,
) -> bool {
    let an_axis = z_axis();
    let mut a_make_torus = BRepPrimAPIMakeTorus::with_axes_pipe_angle(
        an_axis, the_r1, the_r2, the_angle1, the_angle2, the_angle,
    )
    .unwrap();
    let a_shape = a_make_torus.shape().unwrap().clone();
    if !a_make_torus.is_done() || a_shape.is_null() {
        return false;
    }
    is_valid(&a_shape)
}

// Test full torus creation (no angle parameters)
#[test]
fn BRepPrimAPI_MakeTorusTest__FullTorus() {
    let (r1, r2) = (10.0, 2.0);
    let mut a_make_torus = BRepPrimAPIMakeTorus::new(r1, r2).unwrap();
    let a_shape = a_make_torus.shape().unwrap().clone();
    assert!(a_make_torus.is_done(), "Full torus creation failed");
    assert!(!a_shape.is_null(), "Resulting shape is null");
    assert!(is_valid(&a_shape), "Full torus shape is not valid");
    // A full torus should have exactly 1 face
    assert_eq!(face_count(&a_shape), 1, "Expected 1 face for full torus");
}

// Test partial torus with only U angle (slice around Z axis)
#[test]
fn BRepPrimAPI_MakeTorusTest__PartialTorusAngleOnly() {
    let (r1, r2) = (10.0, 2.0);
    let angle = PI; // Half torus around Z axis
    let mut a_make_torus = BRepPrimAPIMakeTorus::with_angle(r1, r2, angle).unwrap();
    let a_shape = a_make_torus.shape().unwrap().clone();
    assert!(
        a_make_torus.is_done(),
        "Partial torus (angle only) creation failed"
    );
    assert!(!a_shape.is_null(), "Resulting shape is null");
    assert!(
        is_valid(&a_shape),
        "Partial torus (angle only) shape is not valid"
    );
    // Should have: 1 lateral + 2 planar (start/end sides)
    assert_eq!(
        face_count(&a_shape),
        3,
        "Expected 3 faces for partial torus with angle only"
    );
}

// Test that the lateral face surface has correct parameterization
#[test]
fn BRepPrimAPI_MakeTorusTest__LateralFaceParameterization() {
    let (r1, r2) = (5.0, 1.0);
    let mut a_make_torus = BRepPrimAPIMakeTorus::new(r1, r2).unwrap();
    let a_shape = a_make_torus.shape().unwrap().clone();
    assert!(a_make_torus.is_done());

    let an_exp = Explorer::new(&a_shape, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    assert!(an_exp.more(), "No faces found");
    let a_face = topods::face(an_exp.current()).unwrap();
    let a_surface = BRepTool::surface(&a_face).unwrap();
    let a_torus_surf = a_surface.cast::<GeomToroidalSurface>();
    let a_torus_surf = a_torus_surf
        .get()
        .expect("Lateral face is not a toroidal surface");
    assert!((a_torus_surf.major_radius() - r1).abs() <= 1e-10);
    assert!((a_torus_surf.minor_radius() - r2).abs() <= 1e-10);

    // At V = PI/2 (top of tube), Z should be +R2
    let a_pnt = a_surface.d0(0.0, FRAC_PI_2).unwrap();
    assert!(
        (a_pnt.z() - r2).abs() <= 1e-10,
        "At V=PI/2, Z should be +MinorRadius"
    );

    // At V = -PI/2 (bottom of tube), Z should be -R2
    let a_pnt = a_surface.d0(0.0, -FRAC_PI_2).unwrap();
    assert!(
        (a_pnt.z() - -r2).abs() <= 1e-10,
        "At V=-PI/2, Z should be -MinorRadius"
    );
}

// =============================================================================
// Tests for partial torus with V parameter range (angle1, angle2)
// These test various angle ranges to verify which configurations produce valid shapes.
// =============================================================================

// Working case: V range symmetric around 0 (-PI/2 to +PI/2)
// Bug 33133 original report - this case actually works
#[test]
fn BRepPrimAPI_MakeTorusTest__PartialTorus_SymmetricAroundZero() {
    let (r1, r2) = (1.0, 0.1);
    let angle1 = -FRAC_PI_2; // -90 degrees
    let angle2 = FRAC_PI_2; // +90 degrees
    let angle = 2.094; // ~120 degrees

    let an_axis = z_axis();
    let mut a_make_torus =
        BRepPrimAPIMakeTorus::with_axes_pipe_angle(an_axis, r1, r2, angle1, angle2, angle).unwrap();
    let a_shape = a_make_torus.shape().unwrap().clone();
    assert!(a_make_torus.is_done(), "Torus creation failed");
    assert!(!a_shape.is_null(), "Resulting shape is null");
    assert!(
        is_valid(&a_shape),
        "Symmetric partial torus should be valid"
    );

    // Count faces: 1 lateral + 2 top/bottom + 2 start/end = 5
    assert_eq!(
        face_count(&a_shape),
        5,
        "Expected 5 faces for partial torus with sides"
    );
}

// Working case: Smaller symmetric range (-45 to +45 degrees)
#[test]
fn BRepPrimAPI_MakeTorusTest__PartialTorus_SmallSymmetricRange() {
    let angle1 = -PI / 4.0; // -45 degrees
    let angle2 = PI / 4.0; // +45 degrees
    let is_valid = create_and_check_partial_torus(10.0, 2.0, angle1, angle2, PI / 4.0);
    assert!(
        is_valid,
        "Small symmetric range (-45 to +45) should produce valid shape"
    );
}

// Bug 23612: V range from 90 to 270 degrees (positive values, not crossing 0)
// This test verifies the fix for inverted height ordering in BRepPrim_OneAxis
#[test]
fn BRepPrimAPI_MakeTorusTest__PartialTorus_Bug23612() {
    let (r1, r2) = (10.0, 2.0);
    let angle1 = FRAC_PI_2; // 90 degrees
    let angle2 = 3.0 * FRAC_PI_2; // 270 degrees
    let angle = PI / 4.0; // 45 degrees

    let an_axis = z_axis();
    let mut a_make_torus =
        BRepPrimAPIMakeTorus::with_axes_pipe_angle(an_axis, r1, r2, angle1, angle2, angle).unwrap();
    let a_shape = a_make_torus.shape().unwrap().clone();
    assert!(a_make_torus.is_done(), "Torus creation failed");
    assert!(!a_shape.is_null(), "Resulting shape is null");

    // Fixed: V range (90, 270) now produces valid shape with correct height ordering handling
    assert!(
        is_valid(&a_shape),
        "Bug 23612 fixed: V range (90, 270) should produce valid shape"
    );
}

// V range from 0 to 180 degrees (top half of minor circle)
// This is an "equal heights" case: sin(0) = sin(180) = 0
// Known limitation: Equal heights cause degenerate AxisEdge topology,
// resulting in UnorientableShape. This is separate from Bug 23612.
#[test]
fn BRepPrimAPI_MakeTorusTest__PartialTorus_ZeroTo180() {
    let angle1 = 0.0;
    let angle2 = PI; // 180 degrees
    let is_valid = create_and_check_partial_torus(10.0, 2.0, angle1, angle2, PI / 4.0);
    // Known limitation: equal heights at endpoints cause topology issues
    // TODO: This could be fixed by using a different topology when heights are equal
    assert!(
        !is_valid,
        "Known limitation: equal heights (sin(0)=sin(180)=0) not supported"
    );
}

// V range from -180 to 0 degrees (bottom half of minor circle)
// This is an "equal heights" case: sin(-180) = sin(0) = 0
// Known limitation: see PartialTorus_ZeroTo180
#[test]
fn BRepPrimAPI_MakeTorusTest__PartialTorus_Minus180ToZero() {
    let angle1 = -PI; // -180 degrees
    let angle2 = 0.0;
    let is_valid = create_and_check_partial_torus(10.0, 2.0, angle1, angle2, PI / 4.0);
    // Known limitation: equal heights at endpoints cause topology issues
    assert!(
        !is_valid,
        "Known limitation: equal heights (sin(-180)=sin(0)=0) not supported"
    );
}

// V range from 180 to 360 degrees (bottom half of minor circle, alternate params)
// This is an "equal heights" case: sin(180) = sin(360) = 0
// Known limitation: see PartialTorus_ZeroTo180
#[test]
fn BRepPrimAPI_MakeTorusTest__PartialTorus_180To360() {
    let angle1 = PI; // 180 degrees
    let angle2 = 2.0 * PI; // 360 degrees
    let is_valid = create_and_check_partial_torus(10.0, 2.0, angle1, angle2, PI / 4.0);
    // Known limitation: equal heights at endpoints cause topology issues
    assert!(
        !is_valid,
        "Known limitation: equal heights (sin(180)=sin(360)=0) not supported"
    );
}

// V range with both positive values not crossing 0 (+45 to +135 degrees)
// This is an "equal heights" case: sin(45) = sin(135) ~ 0.707
// Known limitation: see PartialTorus_ZeroTo180
#[test]
fn BRepPrimAPI_MakeTorusTest__PartialTorus_PositiveRange() {
    let angle1 = PI / 4.0; // 45 degrees
    let angle2 = 3.0 * PI / 4.0; // 135 degrees
    let is_valid = create_and_check_partial_torus(10.0, 2.0, angle1, angle2, PI / 4.0);
    // Known limitation: equal heights at endpoints cause topology issues
    assert!(
        !is_valid,
        "Known limitation: equal heights (sin(45)=sin(135)) not supported"
    );
}

// V range with both negative values not crossing 0 (-135 to -45 degrees)
// This is an "equal heights" case: sin(-135) = sin(-45) ~ -0.707
// Known limitation: see PartialTorus_ZeroTo180
#[test]
fn BRepPrimAPI_MakeTorusTest__PartialTorus_NegativeRange() {
    let angle1 = -3.0 * PI / 4.0; // -135 degrees
    let angle2 = -PI / 4.0; // -45 degrees
    let is_valid = create_and_check_partial_torus(10.0, 2.0, angle1, angle2, PI / 4.0);
    // Known limitation: equal heights at endpoints cause topology issues
    assert!(
        !is_valid,
        "Known limitation: equal heights (sin(-135)=sin(-45)) not supported"
    );
}

// V range asymmetric but crossing 0 (-30 to +60 degrees)
// This works because the range stays within (-PI/2, PI/2)
#[test]
fn BRepPrimAPI_MakeTorusTest__PartialTorus_AsymmetricCrossingZero() {
    let angle1 = -PI / 6.0; // -30 degrees
    let angle2 = PI / 3.0; // +60 degrees
    let is_valid = create_and_check_partial_torus(10.0, 2.0, angle1, angle2, PI / 4.0);
    assert!(
        is_valid,
        "Asymmetric range crossing 0 but within (-PI/2, PI/2) should be valid"
    );
}

// Small positive range (+30 to +60 degrees)
// This works because the range stays within (-PI/2, PI/2)
#[test]
fn BRepPrimAPI_MakeTorusTest__PartialTorus_SmallPositiveRange() {
    let angle1 = PI / 6.0; // 30 degrees
    let angle2 = PI / 3.0; // 60 degrees
    let is_valid = create_and_check_partial_torus(10.0, 2.0, angle1, angle2, PI / 4.0);
    assert!(
        is_valid,
        "Small positive range within (-PI/2, PI/2) should be valid"
    );
}

// =============================================================================
// Root cause analysis test
// =============================================================================

/// This test documents the root cause of bugs 23612/33133:
/// BRepPrim_OneAxis assumes MeridianValue(VMax).Y() > MeridianValue(VMin).Y(),
/// i.e., that the "top" (max V parameter) is geometrically above the "bottom".
/// For circular meridians (torus), this is only true for certain V ranges.
///
/// The meridian circle parameterization for torus:
///   X(V) = Major + Minor * cos(V)  [distance from axis]
///   Y(V) = Minor * sin(V)          [height/Z coordinate]
///
/// Working ranges: Y increases as V increases (e.g., -90 to +90 deg)
/// Failing ranges: Y decreases as V increases (e.g., 90 to 270 deg)
///                 or Y is equal at VMin and VMax (e.g., 45 to 135 deg)
#[test]
fn BRepPrimAPI_MakeTorusTest__RootCause_HeightOrderingAssumption() {
    let (r1, r2) = (10.0, 2.0);

    // For a torus meridian: Y(V) = R2 * sin(V)
    let compute_height = |angle_rad: f64| r2 * angle_rad.sin();

    // Case 1: Working range (-90 to +90 deg)
    // Y(-90 deg) = -2, Y(+90 deg) = +2 => Y(VMax) > Y(VMin)
    {
        let v_min = -FRAC_PI_2;
        let v_max = FRAC_PI_2;
        let y_min = compute_height(v_min);
        let y_max = compute_height(v_max);
        assert!(y_max > y_min, "Working case: Y(VMax) should be > Y(VMin)");

        let is_valid = create_and_check_partial_torus(r1, r2, v_min, v_max, PI / 4.0);
        assert!(
            is_valid,
            "Range with correct height ordering should produce valid shape"
        );
    }

    // Case 2: Previously failing range (90 to 270 deg) - Bug 23612 (FIXED)
    // Y(90 deg) = +2, Y(270 deg) = -2 => Y(VMax) < Y(VMin) - height ordering is inverted
    // Fix: BRepPrim_OneAxis now detects inverted height ordering and adjusts
    // edge vertex 'first' flags and wire 'reversed' flags accordingly
    {
        let v_min = FRAC_PI_2; // 90 deg
        let v_max = 3.0 * FRAC_PI_2; // 270 deg
        let y_min = compute_height(v_min);
        let y_max = compute_height(v_max);
        assert!(
            y_max < y_min,
            "Height ordering is inverted: Y(VMax) < Y(VMin)"
        );

        let is_valid = create_and_check_partial_torus(r1, r2, v_min, v_max, PI / 4.0);
        // Fixed: BRepPrim_OneAxis now correctly handles inverted height ordering
        assert!(
            is_valid,
            "Bug 23612 fixed: inverted height ordering now handled correctly"
        );
    }

    // Case 3: Equal heights (45 to 135 deg) - Known Limitation
    // Y(45 deg) ~ 1.41, Y(135 deg) ~ 1.41 => Heights are equal at endpoints
    // The range covers distinct geometric regions (arc from 45 to 135 deg), but the
    // resulting topology has a degenerate AxisEdge (zero length) which cannot form
    // an orientable shell with the current face/wire structure.
    {
        let v_min = PI / 4.0; // 45 deg
        let v_max = 3.0 * PI / 4.0; // 135 deg
        let y_min = compute_height(v_min);
        let y_max = compute_height(v_max);
        assert!(
            (y_max - y_min).abs() <= 1e-10,
            "Heights at VMin and VMax are equal"
        );

        let is_valid = create_and_check_partial_torus(r1, r2, v_min, v_max, PI / 4.0);
        // Known limitation: equal heights cause degenerate AxisEdge topology
        // This is a separate issue from Bug 23612 (inverted heights)
        assert!(
            !is_valid,
            "Known limitation: equal heights not yet supported"
        );
    }
}

// Test full torus (no sides) with equal heights - should work
#[test]
fn BRepPrimAPI_MakeTorusTest__FullTorus_EqualHeights() {
    let (r1, r2) = (10.0, 2.0);
    let angle1 = 0.0; // 0 degrees
    let angle2 = PI; // 180 degrees

    let an_axis = z_axis();

    // Full revolution (2*PI) - no sides
    let mut a_make_torus =
        BRepPrimAPIMakeTorus::with_axes_pipe(an_axis, r1, r2, angle1, angle2).unwrap();
    let a_shape = a_make_torus.shape().unwrap().clone();
    assert!(a_make_torus.is_done(), "Torus creation failed");
    assert!(!a_shape.is_null(), "Shape is null");
    assert!(
        is_valid(&a_shape),
        "Full torus (no sides) with equal heights should be valid"
    );
}
