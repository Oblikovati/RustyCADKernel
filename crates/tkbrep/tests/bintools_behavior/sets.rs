//! The curve, 2D curve, surface and location sets.
use super::fakes::{stream_bytes, written};
use rustycad_tkbrep::bintools::{BinCurve2dSet, BinCurveSet, BinLocationSet, BinSurfaceSet};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::storage::IoStream;
use rustycad_tkg2d::geom2d::{Geom2dCircle, Geom2dCurve};
use rustycad_tkg3d::geom::{
    GeomCircle, GeomCurve, GeomCylindricalSurface, GeomLine, GeomSurface, GeomTrimmedCurve,
};
use rustycad_tkmath::gp::{
    Axis2, Axis2d, Axis3, Direction2, Direction3, Point2, Point3, Transform3,
};
use rustycad_tkmath::toploc::Location;

fn circle() -> Handle<dyn GeomCurve> {
    let frame = Axis2::new(Point3::new(1.0, 2.0, 3.0), Direction3::default()).unwrap();
    Handle::new(GeomCircle::new(frame, 2.5).unwrap()).into_cast()
}

fn line() -> Handle<dyn GeomCurve> {
    let direction = Direction3::new(1.0, 0.0, 0.0).unwrap();
    Handle::new(GeomLine::from_point(Point3::new(0.0, 0.0, 0.0), direction)).into_cast()
}

#[test]
fn the_curve_set_skips_null_curves_and_writes_a_tagged_table() {
    let mut set = BinCurveSet::new();
    assert_eq!(set.add(&Handle::default()), 0);
    assert_eq!(set.add(&circle()), 1);
    let trimmed: Handle<dyn GeomCurve> =
        Handle::new(GeomTrimmedCurve::new(&line(), 0.0, 1.0, true, true).unwrap()).into_cast();
    assert_eq!(set.add(&trimmed), 2);
    assert!(set.curve(0).unwrap().is_null());
    assert_eq!(
        set.curve(3).err().unwrap().exception_type(),
        "Standard_OutOfRange"
    );
    let bytes = written(|stream| set.write(stream, &ProgressRange::default()).unwrap());
    assert!(bytes.starts_with(b"Curves 2\n\x02"));
    let mut back = BinCurveSet::new();
    let mut stream = IoStream::from_bytes(bytes.clone());
    back.read(&mut stream, &ProgressRange::default()).unwrap();
    let again = written(|stream| back.write(stream, &ProgressRange::default()).unwrap());
    assert_eq!(bytes, again);
    // The set is not cleared by Read, whatever its documentation says.
    let mut stream = IoStream::from_bytes(bytes);
    back.read(&mut stream, &ProgressRange::default()).unwrap();
    assert_eq!(back.extent(), 4);
}

#[test]
fn a_bad_tag_or_an_unknown_type_raises_the_wrapped_failure() {
    let mut set = BinCurveSet::new();
    let mut stream = IoStream::from_bytes(b"Curvez 1\n".to_vec());
    let error = set
        .read(&mut stream, &ProgressRange::default())
        .unwrap_err();
    assert_eq!(
        error.message(),
        "BinTools_CurveSet::Read:  Not a Curve table\n"
    );
    let mut stream =
        IoStream::from_bytes(b"Curves 1\n\x08\0\0\0\0\0\0\0\0\0\0\0\0\0\0\xf0\x3f\x2a".to_vec());
    let error = set
        .read(&mut stream, &ProgressRange::default())
        .unwrap_err();
    assert_eq!(
        error.message(),
        "EXCEPTION in BinTools_CurveSet::ReadCurve(..)\nStandard_Failure: EXCEPTION in \
         BinTools_CurveSet::ReadCurve(..)\nStandard_Failure: UNKNOWN CURVE TYPE\n\n"
    );
}

#[test]
fn the_2d_curve_and_surface_sets_store_null_handles_and_have_no_index_0() {
    let mut pcurves = BinCurve2dSet::new();
    assert_eq!(pcurves.add(&Handle::default()), 1);
    assert_eq!(pcurves.index(&Handle::default()), 1);
    assert_eq!(
        pcurves.curve2d(0).err().unwrap().exception_type(),
        "Standard_OutOfRange"
    );
    let error = written(|stream| {
        let error = pcurves
            .write(stream, &ProgressRange::default())
            .unwrap_err();
        assert_eq!(error.exception_type(), "Standard_NullObject");
    });
    assert_eq!(error, b"Curve2ds 1\n");
    let mut surfaces = BinSurfaceSet::new();
    assert_eq!(
        surfaces.surface(0).err().unwrap().exception_type(),
        "Standard_OutOfRange"
    );
    assert_eq!(surfaces.add(&Handle::default()), 1);
}

#[test]
fn an_indirect_frame_survives_the_surface_set() {
    let mut frame = Axis3::default();
    frame.y_reverse();
    let cylinder: Handle<dyn GeomSurface> =
        Handle::new(GeomCylindricalSurface::new(frame, 2.0).unwrap()).into_cast();
    let mut set = BinSurfaceSet::new();
    set.add(&cylinder);
    let bytes = written(|stream| set.write(stream, &ProgressRange::default()).unwrap());
    let mut back = BinSurfaceSet::new();
    back.read(
        &mut IoStream::from_bytes(bytes.clone()),
        &ProgressRange::default(),
    )
    .unwrap();
    let read = back.surface(1).unwrap();
    let expected = cylinder.value(0.5, 0.25).unwrap();
    assert_eq!(read.value(0.5, 0.25).unwrap(), expected);
    // `gp_Ax3(P, A, AX)` then `YReverse` may flip the sign of a zero coordinate, so the
    // bytes are stable only from the first round trip on.
    let again = written(|stream| back.write(stream, &ProgressRange::default()).unwrap());
    let mut third = BinSurfaceSet::new();
    third
        .read(
            &mut IoStream::from_bytes(again.clone()),
            &ProgressRange::default(),
        )
        .unwrap();
    let last = written(|stream| third.write(stream, &ProgressRange::default()).unwrap());
    assert_eq!(again, last);
    assert_eq!(bytes.len(), again.len());
}

#[test]
fn a_2d_circle_round_trips() {
    let axis = Axis2d::new(Point2::new(1.0, 1.0), Direction2::new(0.0, 1.0).unwrap());
    let circle: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dCircle::from_axis(axis, 3.0, false).unwrap()).into_cast();
    let mut set = BinCurve2dSet::new();
    set.add(&circle);
    let bytes = written(|stream| set.write(stream, &ProgressRange::default()).unwrap());
    let mut back = BinCurve2dSet::new();
    back.read(&mut IoStream::from_bytes(bytes), &ProgressRange::default())
        .unwrap();
    assert_eq!(
        back.curve2d(1).unwrap().value(0.7).unwrap(),
        circle.value(0.7).unwrap()
    );
}

fn translation(x: f64) -> Location {
    Location::new(Transform3::translation([x, 0.0, 0.0]))
}

#[test]
fn the_location_set_stores_datums_first_and_writes_them_as_matrices() {
    let (a, b) = (translation(1.0), translation(2.0));
    let complex = a.powered(3).unwrap().multiplied(&b).unwrap();
    let mut set = BinLocationSet::new();
    assert_eq!(set.add(&complex), 3);
    // Both datums are stored, as elementary locations, before the complex one.
    assert_eq!(set.index(&a) + set.index(&b), 3);
    assert!(set.location(0).unwrap().is_identity());
    assert_eq!(
        set.location(4).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    let mut stream = IoStream::from_bytes(Vec::new());
    set.write(&mut stream).unwrap();
    let bytes = stream_bytes(stream);
    assert!(bytes.starts_with(b"Locations 3\n\x01"));
    let mut back = BinLocationSet::new();
    back.read(&mut IoStream::from_bytes(bytes)).unwrap();
    assert_eq!(back.nb_locations(), 3);
    let read = back.location(3).unwrap().transformation();
    assert_eq!(read.translation_part().x(), 5.0);
}

#[test]
fn an_unknown_location_record_is_named_by_its_byte() {
    let mut set = BinLocationSet::new();
    let mut stream = IoStream::from_bytes(b"Locations 1\n\x03".to_vec());
    let error = set.read(&mut stream).unwrap_err();
    assert_eq!(
        error.message(),
        "EXCEPTION in BinTools_LocationSet::Read(..)\nStandard_Failure: Unexpected location's type = \u{3}\n\n"
    );
    let mut stream = IoStream::from_bytes(b"Locationz 0\n".to_vec());
    let error = set.read(&mut stream).unwrap_err();
    assert_eq!(
        error.message(),
        "BinTools_LocationSet::Read: Not a location table\n"
    );
}
