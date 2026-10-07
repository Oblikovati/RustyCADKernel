//! The BOPAlgo_MakeConnected tests of BOPAlgo_MakePeriodic_Test.cxx (3d097a0).
use super::BOPTest_Utilities;
use super::helpers::*;
use super::near;
use rustycad_tkbo::bop_algo::{BOPAlgoMakeConnected, PeriodicityParams};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::precision;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Point3, Vector3};
use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeBox, BRepPrimAPIMakePrism};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeFace, BRepBuilderAPIMakePolygon, BRepBuilderAPIMakeShape,
};

fn box_at(corner: (f64, f64, f64), x: f64, y: f64, z: f64) -> Shape {
    let corner = Point3::new(corner.0, corner.1, corner.2);
    BRepPrimAPIMakeBox::at_point(corner, x, y, z)
        .unwrap()
        .shape()
        .unwrap()
        .clone()
}

fn modified(connected: &BOPAlgoMakeConnected, shape: &Shape) -> Vec<Shape> {
    connected.get_modified(shape).unwrap()
}

/// The materials on one side of the modified faces of `face`.
fn side_materials(connected: &BOPAlgoMakeConnected, face: &Shape, positive: bool) -> Vec<Shape> {
    let mut materials = Vec::new();
    for modified_face in GetFaces(&MakeCompound(&modified(connected, face))) {
        let side = match positive {
            true => connected.materials_on_positive_side(&modified_face),
            false => connected.materials_on_negative_side(&modified_face),
        };
        materials.extend(side.iter().cloned());
    }
    materials
}

// Equivalent to tests/boolean/mkconnected/A1.
#[test]
fn BOPAlgo_MakePeriodicTest__MakeConnected_A1_PeriodicConnectedSolids() {
    let box1 = MakePeriodicityProfile();
    let box2 = box_at((0.0, 0.0, 5.0), 5.0, 5.0, 5.0);
    let box3 = box_at((0.0, 5.0, 0.0), 10.0, 5.0, 10.0);
    let mut connected = BOPAlgoMakeConnected::new();
    PerformConnected(&mut connected, &[box3.clone(), box1.clone(), box2.clone()]);
    assert!(!connected.options().has_errors());
    ExpectShapeCounts(connected.shape(), [18, 31, 17, 17, 3, 3]);
    near(
        BOPTest_Utilities::GetSurfaceArea(connected.shape()),
        900.0,
        precision::confusion(),
    );
    near(
        BOPTest_Utilities::GetVolume(connected.shape()),
        1000.0,
        precision::confusion(),
    );
    assert!(!connected.history().is_null());
    assert_eq!(
        CountShapesList(&modified(&connected, &box3), ShapeEnum::Face),
        7
    );

    connected
        .make_periodic(&MakeXYZPeriodicity(10.0, 10.0, 10.0))
        .unwrap();
    assert!(!connected.periodicity_tool().options().has_errors());
    ExpectShapeCounts(connected.periodic_shape(), [26, 42, 20, 20, 3, 3]);
    assert_eq!(
        CountShapesList(&modified(&connected, &box1), ShapeEnum::Face),
        10
    );
    assert_eq!(
        CountShapesList(&modified(&connected, &box2), ShapeEnum::Vertex),
        CountShapes(&box2, ShapeEnum::Vertex)
    );
    assert_eq!(
        CountShapesList(&modified(&connected, &box3), ShapeEnum::Face),
        8
    );

    let material_face = GetFace(&box3, 2);
    assert!(!material_face.is_null());
    assert!(!GetFaces(&MakeCompound(&modified(&connected, &material_face))).is_empty());
    let positive = side_materials(&connected, &material_face, true);
    let negative = side_materials(&connected, &material_face, false);
    assert!(!positive.is_empty());
    assert!(!negative.is_empty());
    ExpectEqualProperties(&MakeCompound(&positive), &MakeCompound(&[box1, box2]), true);
    ExpectEqualProperties(&MakeCompound(&negative), &MakeCompound(&[box3]), true);
}

fn mkconnected_a2_prism() -> Shape {
    let mut polygon = BRepBuilderAPIMakePolygon::new();
    for (x, y) in [
        (3.0, 0.0),
        (3.0, 3.0),
        (0.0, 3.0),
        (0.0, 4.0),
        (4.0, 4.0),
        (4.0, 0.0),
    ] {
        polygon.add_point(Point3::new(x, y, 6.0)).unwrap();
    }
    polygon.close().unwrap();
    let face = BRepBuilderAPIMakeFace::from_wire(&polygon.wire().unwrap(), false)
        .unwrap()
        .face()
        .unwrap();
    let mut prism =
        BRepPrimAPIMakePrism::new(&face.into_shape(), Vector3::new(0.0, 0.0, 4.0), false, true)
            .unwrap();
    prism.shape().unwrap().clone()
}

fn xy_periodicity(period: f64) -> PeriodicityParams {
    let mut params = PeriodicityParams::new();
    params.periodic[0] = true;
    params.periodic[1] = true;
    params.period[0] = period;
    params.period[1] = period;
    params
}

fn repeat_around(connected: &mut BOPAlgoMakeConnected) {
    for (direction, times) in [(0, -1), (0, 1), (1, -1), (1, 1)] {
        connected.repeat_shape(direction, times).unwrap();
    }
}

fn expect_area_volume(shape: &Shape, area: f64, volume: f64) {
    near(
        BOPTest_Utilities::GetSurfaceArea(shape),
        area,
        precision::confusion(),
    );
    near(
        BOPTest_Utilities::GetVolume(shape),
        volume,
        precision::confusion(),
    );
}

// Equivalent to tests/boolean/mkconnected/A2.
#[test]
fn BOPAlgo_MakePeriodicTest__MakeConnected_A2_PeriodicAndRepeatedConnectedSolids() {
    let box1 = box_at((0.0, 0.0, 0.0), 10.0, 10.0, 5.0);
    let box2 = box_at((0.0, 0.0, 5.0), 10.0, 10.0, 1.0);
    let box3 = box_at((0.0, 0.0, 6.0), 3.0, 3.0, 4.0);
    let box5 = box_at((0.0, 9.0, 6.0), 4.0, 1.0, 4.0);
    let box6 = box_at((9.0, 9.0, 6.0), 1.0, 1.0, 4.0);
    let box7 = box_at((9.0, 0.0, 6.0), 1.0, 3.0, 4.0);
    let arguments = [
        box1,
        box2,
        box3,
        mkconnected_a2_prism(),
        box5.clone(),
        box6.clone(),
        box7.clone(),
    ];
    let mut connected = BOPAlgoMakeConnected::new();
    PerformConnected(&mut connected, &arguments);
    assert!(!connected.options().has_errors());
    ExpectShapeCounts(connected.shape(), [46, 79, 41, 41, 7, 7]);
    expect_area_volume(connected.shape(), 888.0, 696.0);

    let mut params = xy_periodicity(10.0);
    connected.make_periodic(&params).unwrap();
    assert!(!connected.periodicity_tool().options().has_errors());
    ExpectShapeCounts(connected.periodic_shape(), [49, 83, 42, 42, 7, 7]);
    expect_area_volume(connected.periodic_shape(), 888.0, 696.0);
    let material_face = GetFace(&box6, 3);
    assert!(!material_face.is_null());
    let none = Shape::default();
    let box6_alone = (&box6, &none);
    ExpectModifiedMaterials(
        &connected,
        &modified(&connected, &material_face),
        box6_alone,
        (&none, &none),
    );
    assert!(!connected.history().is_null());
    let modified_box5 = MakeCompound(&modified(&connected, &box5));
    ExpectShapeCounts(&modified_box5, [10, 15, 7, 7, 1, 1]);
    expect_area_volume(&modified_box5, 48.0, 16.0);

    repeat_around(&mut connected);
    ExpectShapeCounts(connected.periodic_shape(), [496, 947, 564, 564, 112, 112]);
    expect_area_volume(connected.periodic_shape(), 14208.0, 11136.0);
    ExpectModifiedMaterials(
        &connected,
        &modified(&connected, &material_face),
        box6_alone,
        (&box7, &none),
    );

    params.clear();
    params.periodic[..2].copy_from_slice(&[true, true]);
    params.period[..2].copy_from_slice(&[12.0, 12.0]);
    params.is_trimmed[..2].copy_from_slice(&[false, false]);
    params.period_first[..2].copy_from_slice(&[-1.0, -1.0]);
    connected.make_periodic(&params).unwrap();
    assert!(!connected.periodicity_tool().options().has_errors());
    ExpectShapeCounts(connected.periodic_shape(), [46, 79, 41, 41, 7, 7]);
    ExpectEqualProperties(connected.shape(), connected.periodic_shape(), true);
    repeat_around(&mut connected);
    ExpectShapeCounts(connected.periodic_shape(), [736, 1264, 656, 656, 112, 112]);
    expect_area_volume(connected.periodic_shape(), 14208.0, 11136.0);
    ExpectModifiedMaterials(
        &connected,
        &modified(&connected, &material_face),
        box6_alone,
        (&none, &none),
    );
}

/// The first modified face of `face`, with the material `box_shape` on its positive side.
fn expect_first_face_material(
    connected: &BOPAlgoMakeConnected,
    face: &Shape,
    box_shape: &Shape,
) -> Shape {
    let faces = GetFaces(&MakeCompound(&modified(connected, face)));
    assert!(!faces.is_empty());
    let first: Shape = (*faces[0]).clone();
    let materials: Vec<Shape> = connected
        .materials_on_positive_side(&first)
        .iter()
        .cloned()
        .collect();
    ExpectMaterials(&materials, box_shape, &Shape::default());
    first
}

fn expect_wire_face_shell_solid(shape: &Shape, faces: i32, solids: i32) {
    ExpectValidShape(shape);
    ExpectWireFaceShellSolidCounts(shape, faces, faces, solids, solids);
}

// Equivalent to tests/boolean/mkconnected/A5.
#[test]
fn BOPAlgo_MakePeriodicTest__MakeConnected_A5_PeriodicRepetitionCanBeCleared() {
    let box1 = box_at((0.0, 0.0, 0.0), 5.0, 5.0, 10.0);
    let box2 = box_at((5.0, 0.0, 0.0), 5.0, 5.0, 5.0);
    let box3 = box_at((0.0, 5.0, 0.0), 5.0, 5.0, 5.0);
    let mut connected = BOPAlgoMakeConnected::new();
    PerformConnected(&mut connected, &[box1.clone(), box2.clone(), box3]);
    assert!(!connected.options().has_errors());
    expect_wire_face_shell_solid(connected.shape(), 18, 3);
    let material_face = GetFace(&box2, 1);
    assert!(!material_face.is_null());

    connected.make_periodic(&xy_periodicity(10.0)).unwrap();
    assert!(!connected.periodicity_tool().options().has_errors());
    expect_wire_face_shell_solid(connected.periodic_shape(), 20, 3);
    let modified_face = expect_first_face_material(&connected, &material_face, &box2);
    let mut negative = Vec::new();
    for twin in connected
        .periodicity_tool()
        .get_twins(&modified_face)
        .iter()
    {
        negative.extend(connected.materials_on_negative_side(twin).iter().cloned());
    }
    ExpectMaterials(&negative, &box1, &Shape::default());

    connected.repeat_shape(0, 1).unwrap();
    connected.repeat_shape(1, 1).unwrap();
    expect_wire_face_shell_solid(connected.periodic_shape(), 76, 12);
    assert!(!connected.history().is_null());
    let generated = connected
        .history()
        .get()
        .unwrap()
        .generated(&material_face)
        .unwrap();
    assert_eq!(CountShapesList(&generated, ShapeEnum::Face), 4);
    connected.clear_repetitions().unwrap();
    ExpectWireFaceShellSolidCounts(connected.periodic_shape(), 20, 20, 3, 3);
    expect_first_face_material(&connected, &material_face, &box2);

    connected.repeat_shape(1, 1).unwrap();
    expect_wire_face_shell_solid(connected.periodic_shape(), 39, 6);
    let final_faces = GetFaces(&MakeCompound(&modified(&connected, &material_face)));
    assert!(!final_faces.is_empty());
    let final_twins: Vec<Shape> = connected
        .periodicity_tool()
        .get_twins(&final_faces[0])
        .iter()
        .cloned()
        .collect();
    assert_eq!(CountShapesList(&final_twins, ShapeEnum::Face), 3);
}
