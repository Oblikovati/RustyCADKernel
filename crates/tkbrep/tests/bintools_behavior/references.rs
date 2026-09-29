//! The shape-reference format: BinTools_ShapeWriter and BinTools_ShapeReader.
use super::fakes::written;
use super::support::{make_box, make_vertex};
use rustycad_tkbrep::bintools::{BinShapeReader, BinShapeSetBase, BinShapeWriter};
use rustycad_tkbrep::brep::access::vertex_tshape;
use rustycad_tkbrep::topods::{Shape, ShapeIterator};
use rustycad_tkernel::storage::IoStream;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::Point3;

fn read_shapes(bytes: Vec<u8>, skip: i64, count: usize) -> Vec<Shape> {
    let mut reader = BinShapeReader::new();
    let mut stream = IoStream::from_bytes(bytes);
    stream.seekg(skip);
    (0..count)
        .map(|_| {
            let mut shape = Shape::default();
            reader.read_shape(&mut stream, &mut shape).unwrap();
            shape
        })
        .collect()
}

#[test]
fn a_box_round_trips_and_shares_its_edges() {
    let solid = make_box(1.0, 2.0, 3.0);
    let bytes = written(|stream| BinShapeWriter::new().write_shape(&solid, stream).unwrap());
    let back = read_shapes(bytes.clone(), 0, 1).remove(0);
    let again = written(|stream| BinShapeWriter::new().write_shape(&back, stream).unwrap());
    assert_eq!(bytes, again);
}

#[test]
fn a_shape_written_twice_is_a_back_reference_the_second_time() {
    let vertex = make_vertex(Point3::new(1.0, 2.0, 3.0));
    let reversed = vertex.oriented(Orientation::Reversed);
    let mut writer = BinShapeWriter::new();
    let bytes = written(|stream| {
        writer.write_shape(&vertex, stream).unwrap();
        writer.write_shape(&reversed, stream).unwrap();
    });
    // Reference8, the delta back to the record at 0, EmptyLocation, REVERSED.
    let first = bytes.len() - 4;
    assert_eq!(&bytes[first..], &[1, first as u8, 7, 1]);
    let shapes = read_shapes(bytes.clone(), 0, 2);
    assert!(shapes[1].is_same(&shapes[0]));
    assert_eq!(shapes[1].orientation(), Orientation::Reversed);
    // Starting at the reference, the reader goes back to the unread record.
    let later = read_shapes(bytes, first as i64, 1);
    assert_eq!(later[0].orientation(), Orientation::Reversed);
    writer.clear();
    let fresh = written(|stream| writer.write_shape(&reversed, stream).unwrap());
    assert_eq!(fresh.len(), first);
}

#[test]
fn a_null_shape_is_one_byte_and_reads_back_null() {
    let bytes = written(|stream| {
        BinShapeWriter::new()
            .write_shape(&Shape::default(), stream)
            .unwrap()
    });
    assert_eq!(bytes, [198]);
    assert!(read_shapes(bytes, 0, 1)[0].is_null());
}

#[test]
fn an_unknown_shape_code_raises_the_wrapped_topology_failure() {
    let mut reader = BinShapeReader::new();
    let mut shape = Shape::default();
    let mut stream = IoStream::from_bytes(vec![5]);
    let error = reader.read_shape(&mut stream, &mut shape).unwrap_err();
    assert_eq!(
        error.message(),
        "EXCEPTION in BinTools_ShapeReader::Read\nStandard_Failure: Unexpected topology type = -49\n\n"
    );
}

#[test]
fn a_point_on_a_pcurve_is_kept_only_without_a_surface() {
    let mut record = vec![228, 7];
    for value in [1e-7, 1.0, 2.0, 3.0] {
        record.extend(f64::to_ne_bytes(value));
    }
    let point_on_pcurve = |surface: &[u8]| {
        let mut rep = vec![2];
        rep.extend(0.5_f64.to_ne_bytes());
        rep.push(11);
        rep.push(1);
        for value in [0.0, 0.0, 1.0, 0.0] {
            rep.extend(f64::to_ne_bytes(value));
        }
        rep.extend_from_slice(surface);
        rep.push(7);
        rep
    };
    let mut plane = vec![13, 1];
    for value in [0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0] {
        plane.extend(f64::to_ne_bytes(value));
    }
    for (surface, kept) in [(vec![14], 1), (plane, 0)] {
        let mut bytes = record.clone();
        bytes.extend(point_on_pcurve(&surface));
        bytes.extend([0, 75, 199]);
        let vertex = read_shapes(bytes, 0, 1).remove(0);
        let points = vertex_tshape(&vertex, "test").unwrap().points();
        assert_eq!(points.len(), kept, "surface record {:?}", &surface[..1]);
        assert!(!ShapeIterator::new(&vertex, false, false).more());
    }
}
