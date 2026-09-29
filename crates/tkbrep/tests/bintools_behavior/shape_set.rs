//! The grouped format: BinTools::Write / Read and BinTools_ShapeSet.
use super::fakes::{BreakingIndicator, RecordingConsole, written};
use super::support::{make_box, make_edge, make_vertex};
use rustycad_tkbrep::bintools::{
    self, BinShapeSet, BinShapeSetBase, THE_ASCII_VERSIONS, WriteOptions, format_version,
};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Face, Shape, Vertex};
use rustycad_tkernel::message::{ProgressIndicator, ProgressRange};
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::storage::IoStream;
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Axis3, Point3};
use rustycad_tkmath::poly::{Triangle, Triangulation, mesh_purpose};
use std::sync::Arc;
use std::sync::atomic::AtomicI32;

fn options(version: i32) -> WriteOptions {
    WriteOptions {
        version,
        ..WriteOptions::default()
    }
}

fn write_set(shape: &Shape, options: WriteOptions) -> Vec<u8> {
    written(|stream| bintools::write(shape, stream, options, &ProgressRange::default()).unwrap())
}

fn read_set(bytes: &[u8]) -> Shape {
    let mut shape = Shape::default();
    let mut stream = IoStream::from_bytes(bytes.to_vec());
    bintools::read(&mut shape, &mut stream, &ProgressRange::default()).unwrap();
    shape
}

fn faces(shape: &Shape) -> Vec<Shape> {
    let mut explorer = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    let mut found = Vec::new();
    while explorer.more() {
        found.push(explorer.current().clone());
        explorer.next().unwrap();
    }
    found
}

#[test]
fn a_box_round_trips_byte_for_byte_from_version_2() {
    let solid = make_box(1.0, 2.0, 3.0);
    for version in format_version::VERSION_2..=format_version::UPPER {
        let bytes = write_set(&solid, options(version));
        assert!(
            bytes.starts_with(format!("\n{}\n", THE_ASCII_VERSIONS[version as usize]).as_bytes())
        );
        assert_eq!(
            write_set(&read_set(&bytes), options(version)),
            bytes,
            "version {version}"
        );
    }
}

#[test]
fn version_1_faces_come_back_checked_after_their_uv_points_are_updated() {
    let solid = make_box(1.0, 2.0, 3.0);
    // BRepPrim completes every box face with BRepTools::Update, which checks it; clear
    // the flag so the read-back flag can only come from the reader's own update.
    for face in faces(&solid) {
        face.set_checked(false).unwrap();
    }
    let bytes = write_set(&solid, options(format_version::VERSION_1));
    let back = read_set(&bytes);
    for face in faces(&back) {
        assert!(
            face.checked().unwrap(),
            "BRepTools::Update marks the face checked"
        );
    }
    for face in faces(&solid) {
        assert!(!face.checked().unwrap());
    }
}

#[test]
fn a_stream_without_a_version_line_reads_nothing_and_says_so() {
    let console = Arc::new(RecordingConsole::default());
    let mut set = BinShapeSet::with_console(console.clone());
    let mut stream = IoStream::from_bytes(b"not a shape file\n".to_vec());
    set.read(&mut stream, &ProgressRange::default()).unwrap();
    assert_eq!(set.nb_shapes(), 0);
    assert_eq!(
        console.text(),
        "BinTools_ShapeSet::Read: File was not written with this version of the topology\n"
    );
}

#[test]
fn a_version_line_ending_with_a_carriage_return_is_recognised() {
    let bytes = write_set(&make_vertex(Point3::new(1.0, 2.0, 3.0)), options(3));
    let end = bytes.windows(4).position(|w| w == b"(c)\n").unwrap() + 3;
    let mut crlf = bytes[..end].to_vec();
    crlf.push(b'\r');
    crlf.extend_from_slice(&bytes[end..]);
    let mut set = BinShapeSet::new();
    let mut stream = IoStream::from_bytes(crlf);
    set.read(&mut stream, &ProgressRange::default()).unwrap();
    assert_eq!(set.format_nb(), 3);
    assert_eq!(set.nb_shapes(), 1);
}

#[test]
fn an_unsupported_format_number_is_ignored() {
    let mut set = BinShapeSet::new();
    set.set_format_nb(0);
    set.set_format_nb(5);
    assert_eq!(set.format_nb(), format_version::CURRENT);
}

fn triangulated_face(with_surface: bool) -> Face {
    let nodes = [
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    ];
    let triangulation = Triangulation::with_sizes(3, 1, false, false).unwrap();
    for (index, node) in nodes.iter().enumerate() {
        triangulation.set_node(index as i32 + 1, *node).unwrap();
    }
    triangulation
        .set_triangle(1, Triangle::new(1, 2, 3))
        .unwrap();
    triangulation.compute_normals().unwrap();
    let triangulation = Handle::new(triangulation);
    let builder = BRepBuilder::default();
    let mut face = Face::default();
    if with_surface {
        let plane: Handle<dyn GeomSurface> =
            Handle::new(GeomPlane::new(Axis3::default())).into_cast();
        builder
            .make_face_on_surface(&mut face, &plane, 1e-7)
            .unwrap();
        builder
            .update_face_triangulation(&face, &triangulation, true)
            .unwrap();
    } else {
        builder
            .make_face_with_triangulation(&mut face, &triangulation)
            .unwrap();
    }
    face
}

#[test]
fn a_face_without_surface_always_keeps_its_triangulation_and_normals() {
    let face = triangulated_face(false);
    let no_triangles = WriteOptions {
        with_triangles: false,
        ..options(4)
    };
    let back = read_set(&write_set(&face, no_triangles));
    let (triangulation, _) =
        BRepTool::triangulation(&Face::unchecked(back.clone()), mesh_purpose::NONE).unwrap();
    assert!(triangulation.has_normals());
    let back = read_set(&write_set(
        &face,
        WriteOptions {
            with_triangles: false,
            ..options(3)
        },
    ));
    let (triangulation, _) =
        BRepTool::triangulation(&Face::unchecked(back), mesh_purpose::NONE).unwrap();
    assert!(!triangulation.is_null());
    assert!(!triangulation.has_normals(), "version 3 has no normals");
}

#[test]
fn a_face_on_a_surface_keeps_its_normals_only_when_asked() {
    let face = triangulated_face(true);
    let without = write_set(&face, options(4));
    let with = write_set(
        &face,
        WriteOptions {
            with_normals: true,
            ..options(4)
        },
    );
    assert_eq!(with.len(), without.len() + 3 * 3 * 4);
    let hidden = write_set(
        &face,
        WriteOptions {
            with_triangles: false,
            ..options(4)
        },
    );
    assert!(hidden.windows(18).any(|w| w == b"Triangulations 0\n\n"));
}

#[test]
fn the_parameters_of_a_point_on_a_surface_are_swapped_by_a_round_trip() {
    let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).into_cast();
    let builder = BRepBuilder::default();
    let mut face = Face::default();
    builder
        .make_face_on_surface(&mut face, &plane, 1e-7)
        .unwrap();
    let vertex = make_vertex(Point3::new(0.25, 0.75, 0.0));
    builder
        .update_vertex_on_face(&vertex, 0.25, 0.75, &face, 1e-7)
        .unwrap();
    builder.add(&mut face, &vertex).unwrap();
    let back = read_set(&write_set(&face, options(3)));
    let back_vertex = Vertex::unchecked(first_child(&back));
    let parameters = BRepTool::parameters(&back_vertex, &Face::unchecked(back)).unwrap();
    assert_eq!((parameters.x(), parameters.y()), (0.75, 0.25));
}

fn first_child(shape: &Shape) -> Shape {
    rustycad_tkbrep::topods::ShapeIterator::new(shape, false, false)
        .value()
        .unwrap()
        .clone()
}

#[test]
fn a_tiny_parameter_is_misread_by_the_pre_6_2_1_layout_test_of_version_4() {
    let edge = make_edge(Point3::new(0.0, 0.0, 0.0), Point3::new(10.0, 0.0, 0.0));
    let vertex = make_vertex(Point3::new(3.0, 0.0, 0.0));
    let builder = BRepBuilder::default();
    let tiny = 2.0_f64.powi(-1000);
    builder
        .update_vertex_on_edge(&vertex, tiny, &edge, 1e-7)
        .unwrap();
    let mut root = edge.clone();
    builder
        .add(
            &mut root,
            &vertex.oriented(rustycad_tkg3d::top_abs::Orientation::Internal),
        )
        .unwrap();
    for (version, kept) in [(3, true), (4, false)] {
        let back = read_set(&write_set(&root, options(version)));
        let internal = Vertex::unchecked(last_child(&back));
        let parameter =
            BRepTool::parameter(&internal, &rustycad_tkbrep::topods::Edge::unchecked(back))
                .unwrap();
        assert_eq!(parameter == tiny, kept, "version {version}");
    }
}

fn last_child(shape: &Shape) -> Shape {
    let mut children = rustycad_tkbrep::topods::ShapeIterator::new(shape, false, false);
    let mut last = Shape::default();
    while children.more() {
        last = children.value().unwrap().clone();
        children.next();
    }
    last
}

#[test]
fn a_sub_shape_index_outside_the_table_raises() {
    let bytes = write_set(&make_vertex(Point3::new(1.0, 2.0, 3.0)), options(4));
    let mut broken = bytes.clone();
    let root = broken.len() - 8;
    broken[root..root + 4].copy_from_slice(&5_i32.to_ne_bytes());
    let mut shape = Shape::default();
    let mut stream = IoStream::from_bytes(broken);
    let error = bintools::read(&mut shape, &mut stream, &ProgressRange::default()).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_OutOfRange");
}

#[test]
fn a_breaking_progress_indicator_stops_the_shape_table() {
    let solid = make_box(1.0, 1.0, 1.0);
    let complete = write_set(&solid, options(4));
    let indicator = ProgressIndicator::new(Box::new(BreakingIndicator {
        calls: Arc::new(AtomicI32::new(0)),
        limit: 12,
    }));
    let partial = written(|stream| {
        bintools::write(&solid, stream, options(4), &indicator.start()).unwrap();
    });
    assert!(partial.len() < complete.len());
    assert_eq!(&partial[..40], &complete[..40]);
}

#[test]
fn the_shape_set_indexes_shapes_without_their_location() {
    let vertex = make_vertex(Point3::new(1.0, 2.0, 3.0));
    let moved = vertex
        .moved(
            &rustycad_tkmath::toploc::Location::new(rustycad_tkmath::gp::Transform3::translation(
                [1.0, 0.0, 0.0],
            )),
            false,
        )
        .unwrap();
    let mut set = BinShapeSet::new();
    assert_eq!(set.add(&moved).unwrap(), 1);
    assert_eq!(set.add(&vertex).unwrap(), 1);
    assert_eq!(set.locations().nb_locations(), 1);
    assert_eq!(set.add(&Shape::default()).unwrap(), 0);
    let reference = written(|stream| set.write_shape(&Shape::default(), stream).unwrap());
    assert_eq!(reference, b"*");
}
