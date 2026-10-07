//! Direct translation of src/ModelingAlgorithms/TKBO/GTests/BOPAlgo_BOP_Test.cxx (3d097a0).
#![allow(non_snake_case)]

#[path = "bop_test_utilities/mod.rs"]
mod bop_test_utilities;
use bop_test_utilities::*;
use rustycad_tkbo::bop_algo::{
    BOPAlgoArgumentAnalyzer, BOPAlgoBOP, BOPAlgoCheckerSI, CheckStatus, Operation,
};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Compound, Face, Shape};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::ncollection::List;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Direction3, Plane3, Point3};
use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeHalfSpace, BRepPrimAPIMakePrism};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape, BRepBuilderAPISewing,
};
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;

fn expect_near(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{actual} differs from {expected} by more than {tolerance}"
    );
}

fn CountUniqueBOPSubShapes(shape: &Shape, kind: ShapeEnum) -> usize {
    let mut shapes = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(shape, kind, &mut shapes).unwrap();
    shapes.extent()
}

fn plane(origin: Point3, normal: (f64, f64, f64)) -> Plane3 {
    Plane3::new(
        origin,
        Direction3::new(normal.0, normal.1, normal.2).unwrap(),
    )
    .unwrap()
}

fn plane_face(plane: Plane3, bounds: (f64, f64, f64, f64)) -> Face {
    let maker = BRepBuilderAPIMakeFace::from_plane_bounds(plane, bounds).unwrap();
    assert!(maker.is_done());
    maker.face().unwrap()
}

fn MakeGridShell() -> Shape {
    let mut sewing = BRepBuilderAPISewing::new(1.0e-6, true, true, true, false);
    let grid_plane = plane(Point3::new(0.0, 0.0, 0.0), (0.0, 0.0, 1.0));
    for x in 0..3 {
        for y in 0..3 {
            let (x, y) = (f64::from(x), f64::from(y));
            sewing
                .add(&plane_face(grid_plane, (x, x + 1.0, y, y + 1.0)).into_shape())
                .unwrap();
        }
    }
    sewing.perform(&ProgressRange::default()).unwrap();
    sewing.sewed_shape().clone()
}

// Migrated from tests/bugs/modalg_5/bug25477_1.  A fuzzy fuse must bridge the
// small gap between the two boxes and produce one solid with the DRAW counts.
#[test]
fn BOPAlgo_BOPTest__ModalgBug_25477_FuzzyFuseBridgesSmallGap() {
    let box1 = CreateBox(Point3::new(0.0, 0.0, 0.0), 10.0, 10.0, 10.0);
    let box2 = CreateBox(Point3::new(10.00001, 0.0, 0.0), 10.0, 10.0, 10.0);
    let mut fuse = BOPAlgoBOP::new();
    fuse.add_argument(&box1);
    fuse.add_tool(&box2);
    fuse.set_operation(Operation::Fuse);
    fuse.options_mut().set_fuzzy_value(0.00002);
    fuse.options_mut().set_run_parallel(false);
    fuse.set_non_destructive(false);
    fuse.perform();
    assert!(!fuse.options().has_errors());
    let result = fuse.shape().clone();
    assert!(!result.is_null());
    assert_eq!(result.shape_type().unwrap(), ShapeEnum::Compound);
    for (kind, count) in [
        (ShapeEnum::Vertex, 12),
        (ShapeEnum::Edge, 20),
        (ShapeEnum::Wire, 10),
        (ShapeEnum::Face, 10),
        (ShapeEnum::Shell, 1),
        (ShapeEnum::Solid, 1),
        (ShapeEnum::Compound, 1),
        (ShapeEnum::CompSolid, 0),
    ] {
        assert_eq!(CountUniqueBOPSubShapes(&result, kind), count, "{kind:?}");
    }
    // Fuzzy classification slightly changes the bridge volume; it must remain
    // close to the sum of the two box volumes.
    expect_near(GetVolume(&result), 2000.0, 0.001);
}

// Migrated from tests/bugs/modalg_5/bug24033.  A common of a sewn planar
// shell and a box must preserve usable, non-INTERNAL orientations.
#[test]
fn BOPAlgo_BOPTest__ModalgBug_24033_CommonResultOrientations() {
    let shell = MakeGridShell();
    let box_shape = CreateBox(Point3::new(-1.0, 1.0, -1.0), 2.0, 1.0, 2.0);
    assert!(!shell.is_null());
    let mut common = BOPAlgoBOP::new();
    common.add_argument(&shell);
    common.add_tool(&box_shape);
    common.set_operation(Operation::Common);
    common.options_mut().set_run_parallel(false);
    common.set_non_destructive(false);
    common.perform();
    assert!(!common.options().has_errors());
    let result = common.shape().clone();
    assert!(!result.is_null());
    assert!(
        BRepCheckAnalyzer::new(&result, true, false, false)
            .unwrap()
            .is_valid()
            .unwrap()
    );
    for kind in [ShapeEnum::Face, ShapeEnum::Edge, ShapeEnum::Vertex] {
        let mut shapes = IndexedMapOfShape::default();
        topexp::map_shapes_of_type(&result, kind, &mut shapes).unwrap();
        assert!(shapes.extent() > 0);
        assert_ne!(
            shapes.find_key(1).unwrap().orientation(),
            Orientation::Internal
        );
    }
}

// Test direct cut operation: bcut result sphere box
#[test]
fn BOPAlgo_DirectOperationsTest__DirectCut_SphereMinusBox() {
    let result = PerformDirectBOP(&CreateUnitSphere(), &CreateUnitBox(), Operation::Cut);
    assert!(!result.is_null(), "Result shape should not be null");
    assert!(
        GetSurfaceArea(&result) > 0.0,
        "Cut result should have positive surface area"
    );
}

// Test direct fuse operation: bfuse result sphere box
#[test]
fn BOPAlgo_DirectOperationsTest__DirectFuse_SpherePlusBox() {
    let (sphere, box_shape) = (CreateUnitSphere(), CreateUnitBox());
    let result = PerformDirectBOP(&sphere, &box_shape, Operation::Fuse);
    assert!(!result.is_null(), "Result shape should not be null");
    let volume = GetVolume(&result);
    assert!(
        volume > GetVolume(&sphere),
        "Fuse result should be larger than sphere alone"
    );
    assert!(
        volume > GetVolume(&box_shape),
        "Fuse result should be larger than box alone"
    );
}

// Test direct common operation: bcommon result box1 box2
#[test]
fn BOPAlgo_DirectOperationsTest__DirectCommon_OverlappingBoxes() {
    let box1 = CreateBox(Point3::new(0.0, 0.0, 0.0), 2.0, 2.0, 2.0);
    let box2 = CreateBox(Point3::new(1.0, 1.0, 1.0), 2.0, 2.0, 2.0);
    let result = PerformDirectBOP(&box1, &box2, Operation::Common);
    ValidateResult(&result, -1.0, 1.0, false);
}

// Test direct tuc operation: btuc result box1 box2
#[test]
fn BOPAlgo_DirectOperationsTest__DirectTUC_IdenticalBoxes() {
    let (box1, box2) = CreateIdenticalBoxes();
    let result = PerformDirectBOP(&box1, &box2, Operation::Cut21);
    ValidateResult(&result, -1.0, -1.0, true);
}

// Test with NURBS converted shapes
#[test]
fn BOPAlgo_DirectOperationsTest__DirectCut_NurbsBoxMinusBox() {
    let box1 = ConvertToNurbs(&CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0));
    assert!(!box1.is_null(), "Failed to convert to NURBS");
    let box2 = CreateBox(Point3::new(0.0, 1.0, 0.0), 1.0, 0.5, 1.0);
    let result = PerformDirectBOP(&box1, &box2, Operation::Cut);
    assert!(
        GetSurfaceArea(&result) > 0.0,
        "NURBS cut result should have positive surface area"
    );
}

// Test two-step cut operation: bop sphere box; bopcut result
#[test]
fn BOPAlgo_TwoStepOperationsTest__TwoStepCut_SphereMinusBox() {
    let result = PerformTwoStepBOP(&CreateUnitSphere(), &CreateUnitBox(), Operation::Cut);
    assert!(
        GetSurfaceArea(&result) > 0.0,
        "Two-step cut result should have positive surface area"
    );
}

// Test two-step fuse operation: bop sphere box; bopfuse result
#[test]
fn BOPAlgo_TwoStepOperationsTest__TwoStepFuse_SpherePlusBox() {
    let (sphere, box_shape) = (CreateUnitSphere(), CreateUnitBox());
    let result = PerformTwoStepBOP(&sphere, &box_shape, Operation::Fuse);
    assert!(!result.is_null(), "Result shape should not be null");
    let volume = GetVolume(&result);
    assert!(
        volume > GetVolume(&sphere),
        "Two-step fuse result should be larger than sphere alone"
    );
    assert!(
        volume > GetVolume(&box_shape),
        "Two-step fuse result should be larger than box alone"
    );
}

// Test two-step common operation: bop box1 box2; bopcommon result
#[test]
fn BOPAlgo_TwoStepOperationsTest__TwoStepCommon_OverlappingBoxes() {
    let box1 = CreateBox(Point3::new(0.0, 0.0, 0.0), 2.0, 2.0, 2.0);
    let box2 = CreateBox(Point3::new(1.0, 1.0, 1.0), 2.0, 2.0, 2.0);
    let result = PerformTwoStepBOP(&box1, &box2, Operation::Common);
    ValidateResult(&result, -1.0, 1.0, false);
}

// Test two-step tuc operation: bop box1 box2; boptuc result
#[test]
fn BOPAlgo_TwoStepOperationsTest__TwoStepTUC_IdenticalBoxes() {
    let (box1, box2) = CreateIdenticalBoxes();
    let result = PerformTwoStepBOP(&box1, &box2, Operation::Cut21);
    ValidateResult(&result, -1.0, -1.0, true);
}

// Test multiple intersecting primitives
#[test]
fn BOPAlgo_ComplexOperationsTest__MultipleIntersectingPrimitives() {
    let sphere = CreateSphere(Point3::new(0.0, 0.0, 0.0), 1.5);
    let cylinder = CreateCylinder(0.8, 3.0);
    let box_shape = CreateBox(Point3::new(-0.5, -0.5, -0.5), 1.0, 1.0, 1.0);
    let intermediate = PerformDirectBOP(&sphere, &cylinder, Operation::Common);
    assert!(
        !intermediate.is_null(),
        "Intermediate result should not be null"
    );
    let result = PerformDirectBOP(&intermediate, &box_shape, Operation::Fuse);
    assert!(
        GetVolume(&result) > 0.0,
        "Complex operation result should have positive volume"
    );
}

// Test comparison between direct and two-step operations
#[test]
fn BOPAlgo_ComplexOperationsTest__DirectVsTwoStepComparison() {
    let (sphere, box_shape) = (CreateUnitSphere(), CreateUnitBox());
    let direct = PerformDirectBOP(&sphere, &box_shape, Operation::Fuse);
    let two_step = PerformTwoStepBOP(&sphere, &box_shape, Operation::Fuse);
    expect_near(GetVolume(&direct), GetVolume(&two_step), DefaultTolerance());
}

#[test]
fn BOPAlgo_DegenerateToolTest__Cut_AxisAlignedThinTool_NearlyPreservesBoxVolume() {
    let box_shape = CreateBox(Point3::new(0.0, 0.0, 0.0), 100.0, 100.0, 100.0);
    let thin = CreateBox(Point3::new(-500.0, 25.0, -500.0), 1500.0, 1.0e-6, 1500.0);
    let result = PerformDirectBOP(&box_shape, &thin, Operation::Cut);
    let overlap = 100.0 * 1.0e-6 * 100.0;
    assert!(!result.is_null());
    expect_near(GetVolume(&result), GetVolume(&box_shape) - overlap, 1.0e-4);
}

#[test]
fn BOPAlgo_DegenerateToolTest__Fuse_AxisAlignedThinTool_AddsNonOverlappingSlice() {
    let box_shape = CreateBox(Point3::new(0.0, 0.0, 0.0), 100.0, 100.0, 100.0);
    let thin = CreateBox(Point3::new(-500.0, 25.0, -500.0), 1500.0, 1.0e-6, 1500.0);
    let result = PerformDirectBOP(&box_shape, &thin, Operation::Fuse);
    let thin_volume = 1500.0 * 1.0e-6 * 1500.0;
    let overlap = 100.0 * 1.0e-6 * 100.0;
    assert!(!result.is_null());
    expect_near(
        GetVolume(&result),
        GetVolume(&box_shape) + thin_volume - overlap,
        1.0e-4,
    );
}

#[test]
fn BOPAlgo_DegenerateToolTest__Cut_LegitimateThinSlab_NotTreatedAsEmpty() {
    let box_shape = CreateBox(Point3::new(0.0, 0.0, 0.0), 100.0, 100.0, 100.0);
    let slab = CreateBox(Point3::new(0.0, 0.0, 50.0), 100.0, 100.0, 1.0);
    let result = PerformDirectBOP(&box_shape, &slab, Operation::Cut);
    assert!(!result.is_null());
    expect_near(
        GetVolume(&result),
        GetVolume(&box_shape) - GetVolume(&slab),
        DefaultTolerance(),
    );
}

#[test]
fn BOPAlgo_DegenerateToolTest__Common_SolidAndPlanarFace_Unaffected() {
    let box_shape = CreateBox(Point3::new(0.0, 0.0, 0.0), 100.0, 100.0, 100.0);
    let face = plane_face(
        plane(Point3::new(0.0, 0.0, 50.0), (0.0, 0.0, 1.0)),
        (-200.0, 200.0, -200.0, 200.0),
    );
    let result = PerformDirectBOP(&box_shape, &face.into_shape(), Operation::Common);
    assert!(!result.is_null());
    expect_near(GetSurfaceArea(&result), 1.0e4, 1.0);
}

#[test]
fn BOPAlgo_DegenerateToolTest__Cut_BySemiInfinitePrism_Unaffected() {
    let box_shape = CreateBox(Point3::new(0.0, -1.0, -1.0), 2.0, 2.0, 2.0);
    let face = plane_face(
        plane(Point3::new(-0.5, 0.0, 0.0), (1.0, 0.0, 0.0)),
        (-0.5, 0.5, -0.5, 0.5),
    );
    let mut prism = BRepPrimAPIMakePrism::infinite(
        &face,
        Direction3::new(1.0, 0.0, 0.0).unwrap(),
        true,
        false,
        true,
    )
    .unwrap();
    assert!(prism.is_done());
    let result = PerformDirectBOP(&box_shape, prism.shape().unwrap(), Operation::Cut);
    assert!(!result.is_null());
    assert!(GetVolume(&result) > 1.0);
}

#[test]
fn BOPAlgo_DegenerateToolTest__Common_SolidAndHalfspace_Unaffected() {
    let box_shape = CreateBox(Point3::new(0.0, 0.0, -30.0), 150.0, 200.0, 200.0);
    let face = plane_face(
        plane(Point3::new(0.0, 0.0, 0.0), (0.0, 0.0, 1.0)),
        (-250.0, 250.0, -250.0, 250.0),
    );
    let half_space =
        BRepPrimAPIMakeHalfSpace::with_face(&face, Point3::new(0.0, 0.0, -100.0)).unwrap();
    let half_space = half_space.solid().unwrap().clone().into_shape();
    let result = PerformDirectBOP(&box_shape, &half_space, Operation::Common);
    assert!(!result.is_null());
    assert!(GetVolume(&result) > 1.0);
}

fn ConfigureArgumentAnalyzer(analyzer: &mut BOPAlgoArgumentAnalyzer) {
    *analyzer.argument_type_mode() = true;
    *analyzer.self_inter_mode() = true;
    *analyzer.small_edge_mode() = true;
    *analyzer.rebuild_face_mode() = true;
    *analyzer.continuity_mode() = true;
    *analyzer.curve_on_surface_mode() = true;
}

// Migrated from tests/bugs/modalg_5/bug24213.  The standard sphere must be
// accepted as a valid single Boolean-operation argument.
#[test]
fn BOPAlgo_ArgumentAnalyzerTest__ModalgBug_24213_SphereIsValidBOPArgument() {
    let mut analyzer = BOPAlgoArgumentAnalyzer::new();
    analyzer.set_shape1(&CreateSphere(Point3::new(0.0, 0.0, 0.0), 10.0));
    ConfigureArgumentAnalyzer(&mut analyzer);
    analyzer.perform();
    assert!(!analyzer.has_faulty());
    assert!(analyzer.check_result().is_empty());
}

// Migrated from tests/bugs/modalg_5/bug24492.  A compound containing a solid
// and an isolated vertex must be reported as a faulty BOP argument.
#[test]
fn BOPAlgo_ArgumentAnalyzerTest__ModalgBug_24492_MixedDimensionalCompoundIsFaulty() {
    let box_shape = CreateBox(Point3::new(0.0, 0.0, 0.0), 10.0, 10.0, 10.0);
    let vertex = CreateVertex(Point3::new(3.0, 3.0, 3.0)).into_shape();
    let builder = BRepBuilder::default();
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    builder.add(&mut compound, &box_shape).unwrap();
    builder.add(&mut compound, &vertex).unwrap();
    let mut analyzer = BOPAlgoArgumentAnalyzer::new();
    analyzer.set_shape1(&compound.into_shape());
    ConfigureArgumentAnalyzer(&mut analyzer);
    analyzer.perform();
    assert!(analyzer.has_faulty());
    let results = analyzer.check_result();
    assert_eq!(results.extent(), 1);
    assert_eq!(
        results.first().unwrap().check_status(),
        CheckStatus::SelfIntersect
    );
}

/// The interfering pairs of source shapes the checker finds at `level`.
fn CountCheckerInterferences(shape: &Shape, level: i32) -> usize {
    let mut checker = BOPAlgoCheckerSI::new();
    checker.set_arguments(&[shape.clone()].into_iter().collect::<List<Shape>>());
    checker.set_level_of_check(level);
    checker.perform();
    assert!(!checker.options().has_errors());
    let data = checker.data_structure().unwrap();
    data.interferences()
        .iter()
        .filter(|pair| {
            let (first, second) = pair.indices();
            !data.is_new_shape(first) && !data.is_new_shape(second)
        })
        .count()
}

// Migrated from tests/bugs/modalg_5/bug24029.  The bopcheck level controls
// which interference families are considered for two overlapping boxes.
#[test]
fn BOPAlgo_CheckerSITest__ModalgBug_24029_CheckLevelControlsInterferences() {
    let box1 = CreateBox(Point3::new(0.0, 0.0, 0.0), 10.0, 10.0, 10.0);
    let box2 = CreateBox(Point3::new(5.0, 5.0, 5.0), 10.0, 10.0, 10.0);
    let builder = BRepBuilder::default();
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    builder.add(&mut compound, &box1).unwrap();
    builder.add(&mut compound, &box2).unwrap();
    let compound = compound.into_shape();
    let level3_count = CountCheckerInterferences(&compound, 3);
    let level4_count = CountCheckerInterferences(&compound, 4);
    let level9_count = CountCheckerInterferences(&compound, 9);
    assert_eq!(level3_count, 0);
    assert!(level4_count > 0);
    assert!(level9_count >= level4_count);
}
