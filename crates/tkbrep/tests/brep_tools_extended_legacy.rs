//! Exact stream, format-version, and parallel-read mirrors of BRepTools_Test.cxx.
#![allow(non_snake_case)]
mod support;

use rustycad_tkbrep::bintools::{self, WriteOptions as BinaryWriteOptions};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::{self, BRepWriteOptions};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::format_version as ascii_version;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::storage::IoStream;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::poly::mesh_purpose;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeShape, BRepBuilderAPINurbsConvert};
use rustycad_tktopalgo::brep_gprop::BRepGProp;
use std::io::{Read, Seek, SeekFrom};
use std::sync::Arc;
use std::thread;
use support::make_box;

fn stream_bytes(stream: IoStream) -> Vec<u8> {
    let mut device = stream.into_device().unwrap();
    device.seek(SeekFrom::Start(0)).unwrap();
    let mut bytes = Vec::new();
    device.read_to_end(&mut bytes).unwrap();
    bytes
}

fn ascii_box_bytes() -> Vec<u8> {
    let shape: Shape = make_box(10.0, 20.0, 30.0).into();
    let mut stream = IoStream::from_bytes(Vec::new());
    brep_tools::write(&shape, &mut stream, &ProgressRange::default()).unwrap();
    assert!(stream.good());
    assert!(stream.tellp() > 0);
    stream_bytes(stream)
}

fn read_volume_mass(bytes: &[u8]) -> Option<f64> {
    let mut stream = IoStream::from_bytes(bytes.to_vec());
    let mut shape = Shape::default();
    if brep_tools::read(
        &mut shape,
        &mut stream,
        &BRepBuilder::default(),
        &ProgressRange::default(),
    )
    .is_err()
        || shape.is_null()
    {
        return None;
    }
    let mut properties = GProps::default();
    BRepGProp::volume_properties(&shape, &mut properties, false, false, false).ok()?;
    Some(properties.mass().abs())
}

fn parallel_masses(bytes: Arc<Vec<u8>>, count: usize) -> Vec<Option<f64>> {
    let workers = (0..count)
        .map(|_| {
            let bytes = Arc::clone(&bytes);
            thread::spawn(move || read_volume_mass(&bytes))
        })
        .collect::<Vec<_>>();
    workers
        .into_iter()
        .map(|worker| worker.join().unwrap_or(None))
        .collect()
}

#[test]
fn BRepTools_Test__ParallelRead_IdenticalMass() {
    let bytes = Arc::new(ascii_box_bytes());
    let reference_mass = read_volume_mass(&bytes).expect("single-threaded read succeeds");
    assert!(reference_mass > 0.0);

    let masses = parallel_masses(bytes, 200);
    assert!(masses.iter().all(Option::is_some));
    for (index, mass) in masses.into_iter().enumerate() {
        let mass = mass.expect("the parallel read produced a shape");
        assert!(
            (mass - reference_mass).abs() <= 1.0e-6,
            "Shape {index} mass {mass} != reference {reference_mass}"
        );
    }
}

#[test]
fn BRepTools_Test__ParallelRead_RepeatedRuns() {
    let bytes = Arc::new(ascii_box_bytes());
    let reference_mass = read_volume_mass(&bytes).expect("single-threaded read succeeds");
    let epsilon = 1.0e-6;

    for run in 0..5 {
        let masses = parallel_masses(Arc::clone(&bytes), 100);
        assert!(masses.iter().all(Option::is_some), "Run {run}: read failed");
        for (index, mass) in masses.into_iter().enumerate() {
            let mass = mass.expect("the parallel read produced a shape");
            assert!(
                (mass - reference_mass).abs() <= epsilon,
                "Run {run}, shape {index}, mass mismatch"
            );
        }
    }
}

fn round_trip(shape: &Shape, binary: bool, triangles: bool, version: i32) -> Shape {
    let mut stream = IoStream::from_bytes(Vec::new());
    if binary {
        bintools::write(
            shape,
            &mut stream,
            BinaryWriteOptions {
                with_triangles: triangles,
                with_normals: false,
                version,
            },
            &ProgressRange::default(),
        )
        .unwrap();
    } else {
        brep_tools::write_with_options(
            shape,
            &mut stream,
            BRepWriteOptions {
                with_triangles: triangles,
                with_normals: false,
                version,
            },
            &ProgressRange::default(),
        )
        .unwrap();
    }
    let bytes = stream_bytes(stream);
    assert!(!bytes.is_empty());
    let mut stream = IoStream::from_bytes(bytes);
    let mut restored = Shape::default();
    if binary {
        bintools::read(&mut restored, &mut stream, &ProgressRange::default()).unwrap();
    } else {
        brep_tools::read(
            &mut restored,
            &mut stream,
            &BRepBuilder::default(),
            &ProgressRange::default(),
        )
        .unwrap();
    }
    assert!(!restored.is_null());
    restored
}

fn subshape_count(shape: &Shape, kind: ShapeEnum) -> usize {
    Explorer::new(shape, kind, ShapeEnum::Shape)
        .unwrap()
        .count()
}

fn triangle_count(shape: &Shape) -> i32 {
    Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)
        .unwrap()
        .map(|face| {
            let face = rustycad_tkbrep::topods::face(&face).unwrap();
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
        assert_eq!(subshape_count(actual, kind), subshape_count(expected, kind));
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
fn BRepTools_Test__ModDataBug_29723_FormatVersions() {
    let source: Shape = make_box(10.0, 20.0, 30.0).into();
    let mut converter = BRepBuilderAPINurbsConvert::with_shape(&source, true).unwrap();
    let nurbs: Shape = converter.shape().unwrap().clone();
    assert!(!nurbs.is_null());
    let versions = [
        (false, ascii_version::VERSION_1),
        (false, ascii_version::VERSION_2),
        (true, 1),
        (true, 2),
        (true, 3),
    ];
    for (binary, version) in versions {
        let restored = round_trip(&nurbs, binary, true, version);
        expect_equivalent_shape(&nurbs, &restored, 0);
    }
}

#[test]
fn BRepTools_Test__DemoBinPersist_1_BinaryStreamRoundTrip() {
    let mut maker = rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox::at_point(
        rustycad_tkmath::gp::Point3::new(10.0, 20.0, 30.0),
        100.0,
        200.0,
        300.0,
    )
    .unwrap();
    let source: Shape = maker.shape().unwrap().clone();
    assert!(!source.is_null());

    let mut stream = IoStream::from_bytes(Vec::new());
    bintools::write(
        &source,
        &mut stream,
        BinaryWriteOptions::default(),
        &ProgressRange::default(),
    )
    .unwrap();
    assert!(stream.good());
    let bytes = stream_bytes(stream);
    assert!(!bytes.is_empty());
    let mut stream = IoStream::from_bytes(bytes);
    let mut restored = Shape::default();
    bintools::read(&mut restored, &mut stream, &ProgressRange::default()).unwrap();
    assert!(!restored.is_null());

    let mut source_bounds = rustycad_tkmath::bnd::BoundingBox3::default();
    let mut restored_bounds = rustycad_tkmath::bnd::BoundingBox3::default();
    rustycad_tktopalgo::brep_bnd_lib::BRepBndLib::add(&source, &mut source_bounds, true).unwrap();
    rustycad_tktopalgo::brep_bnd_lib::BRepBndLib::add(&restored, &mut restored_bounds, true)
        .unwrap();
    let restored_bounds = restored_bounds.get().unwrap();
    let source_bounds = source_bounds.get().unwrap();
    for index in 0..6 {
        assert!((restored_bounds[index] - source_bounds[index]).abs() <= 1.0e-9);
    }
    assert!(
        rustycad_tktopalgo::brep_check::BRepCheckAnalyzer::new(&restored, true, false, false)
            .unwrap()
            .is_valid()
            .unwrap()
    );
    for kind in [
        ShapeEnum::Solid,
        ShapeEnum::Shell,
        ShapeEnum::Face,
        ShapeEnum::Edge,
        ShapeEnum::Vertex,
    ] {
        assert_eq!(
            subshape_count(&restored, kind),
            subshape_count(&source, kind)
        );
    }
    for (kind, expected) in [(0, 4800.0), (1, 220000.0), (2, 6000000.0)] {
        let mut original = GProps::default();
        let mut copy = GProps::default();
        match kind {
            0 => {
                BRepGProp::linear_properties(&source, &mut original, false, false).unwrap();
                BRepGProp::linear_properties(&restored, &mut copy, false, false).unwrap();
            }
            1 => {
                BRepGProp::surface_properties(&source, &mut original, false, false).unwrap();
                BRepGProp::surface_properties(&restored, &mut copy, false, false).unwrap();
            }
            _ => {
                BRepGProp::volume_properties(&source, &mut original, false, false, false).unwrap();
                BRepGProp::volume_properties(&restored, &mut copy, false, false, false).unwrap();
            }
        }
        assert!((copy.mass() - original.mass()).abs() <= 1.0e-7);
        assert_eq!(original.mass(), expected);
    }
}
