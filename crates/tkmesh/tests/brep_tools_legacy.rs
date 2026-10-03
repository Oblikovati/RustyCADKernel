//! BRepTools legacy cases whose exact input behavior depends on TKMesh.
#![allow(non_snake_case)]

use rustycad_tkbrep::bintools::{self, WriteOptions as BinaryWriteOptions};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools::{self, BRepWriteOptions};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::storage::IoStream;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::poly::mesh_purpose;
use rustycad_tkmesh::brep_mesh::BRepMeshIncrementalMesh;
use rustycad_tkmesh::imesh_tools::MeshParameters;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeShape};
use rustycad_tktopalgo::brep_gprop::BRepGProp;
use std::io::{Read, Seek, SeekFrom};

fn stream_bytes(stream: IoStream) -> Vec<u8> {
    let mut device = stream.into_device().unwrap();
    device.seek(SeekFrom::Start(0)).unwrap();
    let mut bytes = Vec::new();
    device.read_to_end(&mut bytes).unwrap();
    bytes
}

fn round_trip(shape: &Shape, binary: bool, with_triangles: bool) -> Shape {
    let mut stream = IoStream::from_bytes(Vec::new());
    if binary {
        bintools::write(
            shape,
            &mut stream,
            BinaryWriteOptions {
                with_triangles,
                with_normals: false,
                version: rustycad_tkbrep::bintools::format_version::CURRENT,
            },
            &ProgressRange::default(),
        )
        .unwrap();
    } else {
        brep_tools::write_with_options(
            shape,
            &mut stream,
            BRepWriteOptions {
                with_triangles,
                with_normals: false,
                version: rustycad_tkbrep::toptools::format_version::CURRENT,
            },
            &ProgressRange::default(),
        )
        .unwrap();
    }
    let bytes = stream_bytes(stream);
    let mut stream = IoStream::from_bytes(bytes);
    let mut restored = Shape::default();
    if binary {
        bintools::read(&mut restored, &mut stream, &ProgressRange::default()).unwrap();
    } else {
        brep_tools::read(
            &mut restored,
            &mut stream,
            &rustycad_tkbrep::brep::BRepBuilder::default(),
            &ProgressRange::default(),
        )
        .unwrap();
    }
    assert!(!restored.is_null());
    restored
}

fn triangle_count(shape: &Shape) -> i32 {
    Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)
        .unwrap()
        .map(|face| {
            let face = topods::face(&face).unwrap();
            BRepTool::triangulation(&face, mesh_purpose::NONE)
                .unwrap()
                .0
                .get()
                .map_or(0, |triangulation| triangulation.nb_triangles())
        })
        .sum()
}

fn expect_equivalent_shape(expected: &Shape, actual: &Shape, triangles: i32) {
    assert_eq!(actual.shape_type().unwrap(), expected.shape_type().unwrap());
    for kind in [
        ShapeEnum::Solid,
        ShapeEnum::Shell,
        ShapeEnum::Face,
        ShapeEnum::Edge,
        ShapeEnum::Vertex,
    ] {
        assert_eq!(
            Explorer::new(actual, kind, ShapeEnum::Shape)
                .unwrap()
                .count(),
            Explorer::new(expected, kind, ShapeEnum::Shape)
                .unwrap()
                .count()
        );
    }
    assert_eq!(triangle_count(actual), triangles);
    let mut expected_volume = GProps::default();
    let mut actual_volume = GProps::default();
    BRepGProp::volume_properties(expected, &mut expected_volume, false, false, false).unwrap();
    BRepGProp::volume_properties(actual, &mut actual_volume, false, false, false).unwrap();
    assert!((actual_volume.mass() - expected_volume.mass()).abs() <= 1.0e-7);
    let mut expected_surface = GProps::default();
    let mut actual_surface = GProps::default();
    BRepGProp::surface_properties(expected, &mut expected_surface, false, false).unwrap();
    BRepGProp::surface_properties(actual, &mut actual_surface, false, false).unwrap();
    assert!((actual_surface.mass() - expected_surface.mass()).abs() <= 1.0e-7);
}

#[test]
fn BRepTools_Test__ModDataBug_31946_TriangleOptions() {
    let mut maker = BRepPrimAPIMakeBox::from_sizes(10.0, 20.0, 30.0).unwrap();
    let shape: Shape = maker.shape().unwrap().clone();
    BRepMeshIncrementalMesh::with_parameters(
        &shape,
        &MeshParameters {
            deflection: 0.1,
            ..MeshParameters::default()
        },
        &ProgressRange::default(),
    )
    .unwrap();
    assert_eq!(triangle_count(&shape), 12);

    for binary in [false, true] {
        let with_triangles = round_trip(&shape, binary, true);
        let without_triangles = round_trip(&shape, binary, false);
        expect_equivalent_shape(&shape, &with_triangles, 12);
        expect_equivalent_shape(&shape, &without_triangles, 0);
    }
}

#[test]
fn BRepTools_Test__ModDataBug_27356_CleanRemovesFreeEdgePolygon3d() {
    let an_edge = free_line_edge();
    let edge_shape = an_edge.clone().into_shape();
    BRepMeshIncrementalMesh::with_parameters(
        &edge_shape,
        &MeshParameters {
            deflection: 0.01,
            ..MeshParameters::default()
        },
        &ProgressRange::default(),
    )
    .expect("BRepMesh_IncrementalMesh::IsDone() should be true");

    let (polygon, _a_location) = BRepTool::polygon3d(&an_edge).unwrap();
    assert!(!polygon.is_null());

    brep_tools::clean(&edge_shape, false).unwrap();

    let (polygon, _) = BRepTool::polygon3d(&an_edge).unwrap();
    assert!(polygon.is_null());
}

fn free_line_edge() -> rustycad_tkbrep::topods::Edge {
    let maker =
        BRepBuilderAPIMakeEdge::from_points(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0))
            .unwrap();
    let edge = maker.edge().unwrap();
    assert!(!edge.is_null());
    edge
}
