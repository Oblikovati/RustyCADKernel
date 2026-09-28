//! TopTools behaviour: the location and shape sets, their texts and their failures.
//! The native kernel comparison is the br_shapeset, br_setread, br_locset and
//! br_locread probes.
mod support;
use rustycad_tkbrep::topods::{Builder, Compound, Shape};
use rustycad_tkbrep::toptools::{
    LocationSet, ShapeMapHasher, ShapeSet, ShapeSetData, THE_ASCII_VERSIONS, TopToolsShapeSet,
    format_version,
};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::ncollection::HashPolicy;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::storage::IoStream;
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::geom_tools::ConsoleOutput;
use rustycad_tkmath::gp::{Point3, Transform3};
use rustycad_tkmath::toploc::Location;
use std::io::{Read, Seek, SeekFrom};
use std::sync::{Arc, Mutex};
use support::{edge_between, make_vertex};

/// A console that records what the readers print, instead of standard output.
#[derive(Default)]
struct RecordingConsole {
    text: Mutex<Vec<u8>>,
}
impl ConsoleOutput for RecordingConsole {
    fn write(&self, bytes: &[u8]) {
        self.text.lock().unwrap().extend_from_slice(bytes);
    }
}
impl RecordingConsole {
    fn text(&self) -> String {
        String::from_utf8(self.text.lock().unwrap().clone()).unwrap()
    }
}

fn written(write: impl FnOnce(&mut IoStream)) -> String {
    let mut stream = IoStream::from_bytes(Vec::new());
    write(&mut stream);
    let mut device = stream.into_device().unwrap();
    device.seek(SeekFrom::Start(0)).unwrap();
    let mut bytes = String::new();
    device.read_to_string(&mut bytes).unwrap();
    bytes
}

fn translation(x: f64) -> Location {
    Location::new(Transform3::translation([x, 0.0, 0.0]))
}

#[test]
fn location_set_stores_datums_before_complex_locations() {
    let (a, b) = (translation(1.0), translation(2.0));
    let complex = a.powered(2).unwrap().multiplied(&b).unwrap();
    let mut set = LocationSet::new();
    assert_eq!(set.add(&complex), 3);
    // The chain is stored head first: b, then a (power 2).
    assert_eq!(set.add(&a), 2);
    assert_eq!(set.index(&complex), 3);
    assert!(set.location(0).is_identity() && set.location(4).is_identity());
    let text = written(|stream| set.write(stream, &ProgressRange::default()));
    assert!(text.starts_with("Locations 3\n1\n"));
    assert!(text.ends_with("2  1 1 2 2 0\n"));
    let mut back = LocationSet::new();
    back.read(
        &mut IoStream::from_bytes(text.into_bytes()),
        &ProgressRange::default(),
    )
    .unwrap();
    // The read datums come from SetValues: same values, Compound form.
    let (read, original) = (back.location(3).transformation(), complex.transformation());
    assert_eq!(read.translation_part(), original.translation_part());
    set.clear();
    assert!(set.location(1).is_identity());
}

#[test]
fn location_set_reader_reports_bad_tables() {
    let console = Arc::new(RecordingConsole::default());
    let mut set = LocationSet::with_console(console.clone());
    set.read(
        &mut IoStream::from_bytes(b"Places 1\n".to_vec()),
        &ProgressRange::default(),
    )
    .unwrap();
    assert_eq!(console.text(), "Not a location table \n");
    let failure = set
        .read(
            &mut IoStream::from_bytes(b"Locations 1\n2  4 1 0\n".to_vec()),
            &ProgressRange::default(),
        )
        .unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_OutOfRange");
    let singular = b"Locations 1\n1\n0 0 0 0 0 0 0 0 0 0 0 0\n".to_vec();
    let failure = set
        .read(
            &mut IoStream::from_bytes(singular),
            &ProgressRange::default(),
        )
        .unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_ConstructionError");
}

#[test]
fn location_dump_names_elementary_and_complex_locations() {
    let a = translation(1.0);
    let mut set = LocationSet::new();
    set.add(&a.powered(3).unwrap());
    let text = written(|stream| set.dump(stream));
    assert!(text.contains("    1 : \nElementary location\n"));
    assert!(text.contains("    2 : \nComplex : L1^3\n"));
}

fn triangle() -> Compound {
    let v: Vec<_> = (0..3)
        .map(|x| make_vertex(Point3::new(f64::from(x), 0.0, 0.0)))
        .collect();
    let mut compound = Compound::default();
    Builder.make_compound(&mut compound);
    let offset = translation(1.0);
    for (a, b) in [(0, 1), (1, 2), (2, 0)] {
        let edge = edge_between(&v[a], &v[b]).moved(&offset, false).unwrap();
        Builder.add(&mut compound, &edge).unwrap();
    }
    compound
}

#[test]
fn shape_set_adds_sub_shapes_first_and_indexes_them_unlocated() {
    let compound = triangle();
    let mut set = TopToolsShapeSet::new();
    assert_eq!(set.add(&compound).unwrap(), 7);
    assert_eq!(set.add(&Shape::default()).unwrap(), 0);
    assert_eq!(set.nb_shapes(), 7);
    assert_eq!(
        set.shape(0).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    assert!(set.shape(7).unwrap().is_same(&compound));
    let mut text = AsciiString::new(">");
    set.dump_extent_string(&mut text).unwrap();
    assert!(
        text.as_bytes()
            .starts_with(b"> VERTEX    : 3\n EDGE      : 3\n")
    );
}

#[test]
fn shape_set_writes_the_version_flags_and_references() {
    let mut set = TopToolsShapeSet::new();
    set.add(&triangle()).unwrap();
    set.set_format_nb(9);
    assert_eq!(set.format_nb(), format_version::CURRENT);
    set.set_format_nb(format_version::VERSION_2);
    let text = written(|stream| set.write(stream, &ProgressRange::default()).unwrap());
    assert!(text.starts_with(&format!("\n{}\nLocations 1\n1\n", THE_ASCII_VERSIONS[2])));
    assert!(text.contains("\nTShapes 7\nVe\n\n0101101\n*\n"));
    // A compound is created free, modified and not orientable.
    assert!(text.contains("Co\n\n1100000\n+5 1 +3 1 +2 1 *\n"), "{text}");
}

#[test]
fn shape_references_round_trip_and_null_is_a_star() {
    let compound = triangle();
    let mut set = TopToolsShapeSet::new();
    set.add(&compound).unwrap();
    let reference = written(|stream| set.write_shape(&compound.reversed(), stream).unwrap());
    assert_eq!(reference, "-1 0 ");
    let back = set
        .read_shape(&mut IoStream::from_bytes(reference.into_bytes()))
        .unwrap();
    assert!(back.is_equal(&compound.reversed()));
    assert_eq!(
        written(|stream| set.write_shape(&Shape::default(), stream).unwrap()),
        "*"
    );
    assert!(
        set.read_shape(&mut IoStream::from_bytes(b"* ".to_vec()))
            .unwrap()
            .is_null()
    );
    let dumped = written(|stream| set.dump_shape(&Shape::default(), stream).unwrap());
    assert_eq!(dumped, "Null shape\nShape : 0, EXTERNAL\n");
}

#[test]
fn the_base_set_cannot_read_shapes_it_does_not_make() {
    let mut set = TopToolsShapeSet::new();
    set.add(&triangle()).unwrap();
    let text = written(|stream| set.write(stream, &ProgressRange::default()).unwrap());
    let mut back = TopToolsShapeSet::new();
    let failure = back
        .read(
            &mut IoStream::from_bytes(text.into_bytes()),
            &ProgressRange::default(),
        )
        .unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_NullObject");
}

#[test]
fn the_base_set_reports_missing_versions_and_tables() {
    let console = Arc::new(RecordingConsole::default());
    let mut set = TopToolsShapeSet::with_console(console.clone());
    set.read(
        &mut IoStream::from_bytes(b"\nnothing\n".to_vec()),
        &ProgressRange::default(),
    )
    .unwrap();
    let table = format!("{}\r\nLocations 0\nShapes 0\n", THE_ASCII_VERSIONS[1]);
    set.read(
        &mut IoStream::from_bytes(table.into_bytes()),
        &ProgressRange::default(),
    )
    .unwrap();
    assert_eq!(set.format_nb(), 1);
    assert_eq!(
        console.text(),
        "File was not written with this version of the topology\nNot a TShape table\n"
    );
}

/// A derived set that makes TopoDS-only shapes and counts the version 1 checks.
struct CountingSet {
    data: ShapeSetData,
    checks: i32,
}
impl ShapeSet for CountingSet {
    fn shape_set_data(&self) -> &ShapeSetData {
        &self.data
    }
    fn shape_set_data_mut(&mut self) -> &mut ShapeSetData {
        &mut self.data
    }
    fn read_shape_geometry(
        &mut self,
        kind: ShapeEnum,
        _: &mut IoStream,
        shape: &mut Shape,
    ) -> Result<(), Failure> {
        *shape = match kind {
            ShapeEnum::Vertex => make_vertex(Point3::default()).into(),
            ShapeEnum::Edge => edge_between(
                &make_vertex(Point3::default()),
                &make_vertex(Point3::default()),
            )
            .empty_copied()?,
            _ => {
                let mut compound = Compound::default();
                Builder.make_compound(&mut compound);
                compound.into()
            }
        };
        Ok(())
    }
    fn add_shapes(&mut self, shape: &mut Shape, child: &Shape) -> Result<(), Failure> {
        Builder.add(shape, child)
    }
    fn check(&mut self, _: ShapeEnum, _: &mut Shape) -> Result<(), Failure> {
        self.checks += 1;
        Ok(())
    }
}

#[test]
fn a_derived_set_reads_back_what_it_wrote() {
    let compound = triangle();
    let mut set = CountingSet {
        data: ShapeSetData::new(),
        checks: 0,
    };
    set.add(&compound).unwrap();
    set.set_format_nb(format_version::VERSION_1);
    let text = written(|stream| set.write(stream, &ProgressRange::default()).unwrap());
    let mut back = CountingSet {
        data: ShapeSetData::new(),
        checks: 0,
    };
    back.read(
        &mut IoStream::from_bytes(text.clone().into_bytes()),
        &ProgressRange::default(),
    )
    .unwrap();
    assert_eq!((back.nb_shapes(), back.checks, back.format_nb()), (7, 7, 1));
    assert_eq!(back.shape(7).unwrap().nb_children(), 3);
    assert!(!back.shape(1).unwrap().checked().unwrap());
    let again = written(|stream| back.write(stream, &ProgressRange::default()).unwrap());
    assert_eq!(again, text);
    back.clear();
    assert_eq!(back.nb_shapes(), 0);
}

#[test]
fn the_map_hasher_ignores_orientation() {
    let vertex: Shape = make_vertex(Point3::default()).into();
    assert!(ShapeMapHasher.equal(&vertex, &vertex.reversed()));
    assert!(!ShapeMapHasher.equal(&vertex, &vertex.moved(&translation(1.0), false).unwrap()));
    assert_eq!(
        ShapeMapHasher.hash(&vertex),
        ShapeMapHasher.hash(&vertex.reversed())
    );
}
