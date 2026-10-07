//! Direct translation of the BRepTools_Test.cxx cases that build their input with the TKBO
//! Boolean operations (src/ModelingData/TKBRep/GTests/BRepTools_Test.cxx at 3d097a0).
#![allow(non_snake_case)]

use rustycad_tkbo::brep_algo_api::{BRepAlgoAPICut, BRepAlgoAPIFuse};
use rustycad_tkbrep::brep_tools::uv_bounds;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Face, Shape};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::gp::{Axis1, Axis2, Direction3, Point3, Transform3, Vector3};
use rustycad_tkmesh::brep_mesh::BRepMeshIncrementalMesh;
use rustycad_tkmesh::imesh_tools::MeshParameters;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape,
    BRepBuilderAPIMakeWire, BRepBuilderAPITransform,
};
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;
use rustycad_tktopalgo::brep_gprop::BRepGProp;
use std::f64::consts::PI;

fn expect_near(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{actual} differs from {expected} by more than {tolerance}"
    );
}

fn UniqueFace(shape: &Shape, index: usize) -> Face {
    let mut faces = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(shape, ShapeEnum::Face, &mut faces).unwrap();
    if index > 0 && index <= faces.extent() {
        Face::unchecked(faces.find_key(index).unwrap().clone())
    } else {
        Face::default()
    }
}

fn MakeCircleFace(center: Point3, normal: Direction3, radius: f64) -> Face {
    let circle: Handle<dyn GeomCurve> =
        Handle::new(GeomCircle::new(Axis2::new(center, normal).unwrap(), radius).unwrap()).cast();
    let edge = BRepBuilderAPIMakeEdge::from_curve(&circle)
        .unwrap()
        .edge()
        .unwrap();
    let wire = BRepBuilderAPIMakeWire::from_edge(&edge)
        .unwrap()
        .wire()
        .unwrap();
    BRepBuilderAPIMakeFace::from_wire(&wire, false)
        .unwrap()
        .face()
        .unwrap()
}

fn CutAndSelectFirstFace(face: &Face, tool: &Face) -> Face {
    let mut cut = BRepAlgoAPICut::from_shapes(face, tool).unwrap();
    cut.build().unwrap();
    if cut.is_done() {
        UniqueFace(cut.shape(), 1)
    } else {
        Face::default()
    }
}

fn ExpectBoxUvBounds(face: &Face) {
    let (u_min, u_max, v_min, v_max) = uv_bounds(face).unwrap();
    expect_near(u_min, 0.0, 1.0e-10);
    expect_near(u_max, 1500.0, 1.0e-10);
    expect_near(v_min, 0.0, 1.0e-10);
    expect_near(v_max, 1500.0, 1.0e-10);
}

fn ExpectSamePrincipalProperties(exact: &GProps, triangulated: &GProps) {
    expect_near(exact.mass(), triangulated.mass(), 1.0e-7);
    let (exact_x, exact_y, exact_z) = exact.principal_properties().unwrap().moments();
    let (mesh_x, mesh_y, mesh_z) = triangulated.principal_properties().unwrap().moments();
    expect_near(exact_x, mesh_x, 1.0e-7);
    expect_near(exact_y, mesh_y, 1.0e-7);
    expect_near(exact_z, mesh_z, 1.0e-7);
}

fn CompareExactAndTriangulatedProperties(shape: &Shape, property_type: i32) {
    let mut exact = GProps::default();
    let mut triangulated = GProps::default();
    match property_type {
        0 => {
            BRepGProp::linear_properties(shape, &mut exact, false, false).unwrap();
            BRepGProp::linear_properties(shape, &mut triangulated, false, true).unwrap();
        }
        1 => {
            BRepGProp::surface_properties(shape, &mut exact, false, false).unwrap();
            BRepGProp::surface_properties(shape, &mut triangulated, false, true).unwrap();
        }
        _ => {
            BRepGProp::volume_properties(shape, &mut exact, false, false, false).unwrap();
            BRepGProp::volume_properties(shape, &mut triangulated, false, false, true).unwrap();
        }
    }
    ExpectSamePrincipalProperties(&exact, &triangulated);
}

// bugs/modalg_7/bug29734: global properties computed from planar face
// triangulations must agree with the exact-geometry calculations.
#[test]
fn BRepTools_Test__ModalgBug_29734_ExactAndTriangulatedProperties() {
    let first_box = BRepPrimAPIMakeBox::from_sizes(1.0, 2.0, 3.0)
        .unwrap()
        .shape()
        .unwrap()
        .clone();
    let second_box = BRepPrimAPIMakeBox::from_sizes(3.0, 2.0, 1.0)
        .unwrap()
        .shape()
        .unwrap()
        .clone();

    let mut translation = Transform3::default();
    translation.set_translation(Vector3::new(0.5, 0.5, 0.5));
    let translated = BRepBuilderAPITransform::with_shape(&second_box, translation, false, false)
        .unwrap()
        .shape()
        .unwrap()
        .clone();

    let mut rotation = Transform3::default();
    let axis = Axis1::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(1.0, 1.0, 1.0).unwrap(),
    );
    rotation.set_rotation(axis, PI / 6.0).unwrap();
    let rotated = BRepBuilderAPITransform::with_shape(&translated, rotation, false, false)
        .unwrap()
        .shape()
        .unwrap()
        .clone();
    let fused = BRepAlgoAPIFuse::from_shapes(&first_box, &rotated)
        .unwrap()
        .shape()
        .clone();
    assert!(!fused.is_null());

    BRepMeshIncrementalMesh::with_parameters(
        &fused,
        &MeshParameters {
            deflection: 0.01,
            ..MeshParameters::default()
        },
        &ProgressRange::default(),
    )
    .expect("BRepMesh_IncrementalMesh::IsDone() should be true");
    assert!(
        BRepCheckAnalyzer::new(&fused, true, false, false)
            .unwrap()
            .is_valid()
            .unwrap()
    );

    CompareExactAndTriangulatedProperties(&fused, 0);
    CompareExactAndTriangulatedProperties(&fused, 1);
    CompareExactAndTriangulatedProperties(&fused, 2);
}

// Migrated from tests/bugs/modalg_5/bug24404.  UV bounds of the selected box
// face must remain its full parametric limits after the successive cuts.
#[test]
fn BRepTools_Test__ModalgBug_24404_BoxFaceUVBounds() {
    let box_shape = BRepPrimAPIMakeBox::from_sizes(1500.0, 1500.0, 1500.0)
        .unwrap()
        .shape()
        .unwrap()
        .clone();
    let mut face = UniqueFace(&box_shape, 4);
    assert!(!face.is_null());
    ExpectBoxUvBounds(&face);

    let normal = Direction3::new(0.0, 1.0, 0.0).unwrap();
    let tools = [
        (
            Point3::new(1361.60462531413, 1500.0, 275.105307765905),
            182.781239888725,
        ),
        (
            Point3::new(398.623694869499, 1500.0, 5.77182937332096),
            181.948898616306,
        ),
        (
            Point3::new(66.2457890511211, 1500.0, 832.500519113239),
            176.393473166564,
        ),
        (
            Point3::new(1177.73545803307, 1500.0, 1406.03245550006),
            150.768085993996,
        ),
    ];
    for (center, radius) in tools {
        face = CutAndSelectFirstFace(&face, &MakeCircleFace(center, normal, radius));
        assert!(!face.is_null());
        ExpectBoxUvBounds(&face);
    }
}
