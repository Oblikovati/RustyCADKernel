// Rust translation of FoundationClasses/TKernel/GTests/Standard_Dump_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Complete Standard_Dump legacy assertions against real geometry types.
#![allow(non_snake_case)]
use rustycad_tkernel::standard::dump::{DumpStream, add_values_separator};
use rustycad_tkmath::{
    bnd::BoundingBox3,
    gp::{Axis3, Direction3, Point3},
};
#[test]
fn Standard_DumpTest__AddValuesSeparator_EmptyStream() {
    let mut stream = DumpStream::default();
    add_values_separator(&mut stream).unwrap();
    assert!(stream.text().is_empty());
}
#[test]
fn Standard_DumpTest__AddValuesSeparator_AfterContent() {
    let mut stream = DumpStream::default();
    stream.append("\"Field1\": 42");
    add_values_separator(&mut stream).unwrap();
    assert_eq!(stream.text(), "\"Field1\": 42, ");
}
#[test]
fn Standard_DumpTest__AddValuesSeparator_AfterOpeningBrace() {
    let mut stream = DumpStream::default();
    stream.append("{");
    add_values_separator(&mut stream).unwrap();
    assert_eq!(stream.text(), "{");
}
#[test]
fn Standard_DumpTest__AddValuesSeparator_AfterExistingSeparator() {
    let mut stream = DumpStream::default();
    stream.append("\"Field1\": 42, ");
    add_values_separator(&mut stream).unwrap();
    assert_eq!(stream.text(), "\"Field1\": 42, ");
}
#[test]
fn Standard_DumpTest__AddValuesSeparator_MultipleFields() {
    let mut stream = DumpStream::default();
    stream.append("\"Field1\": 42");
    add_values_separator(&mut stream).unwrap();
    stream.append("\"Field2\": \"value\"");
    add_values_separator(&mut stream).unwrap();
    stream.append("\"Field3\": [1, 2, 3]");
    assert_eq!(
        stream.text(),
        "\"Field1\": 42, \"Field2\": \"value\", \"Field3\": [1, 2, 3]"
    );
}
#[test]
fn Standard_DumpTest__gp_Pnt_DumpAndInit() {
    let point = Point3::new(1.5, 2.5, 3.5);
    let mut stream = DumpStream::default();
    point.dump_json(&mut stream).unwrap();
    assert!(stream.text().contains("\"gp_Pnt\""));
    assert!(stream.text().contains("[1.5, 2.5, 3.5]"));
    let mut restored = Point3::default();
    let mut position = 1;
    assert!(
        restored
            .init_from_json(stream.text(), &mut position)
            .unwrap()
    );
    assert_eq!(point.x(), restored.x());
    assert_eq!(point.y(), restored.y());
    assert_eq!(point.z(), restored.z());
}
#[test]
fn Standard_DumpTest__gp_Ax3_DumpAndInit_MultipleSeparators() {
    let axis = Axis3::new(Point3::new(1.0, 2.0, 3.0), Direction3::Z, Direction3::X).unwrap();
    let mut stream = DumpStream::default();
    axis.dump_json(&mut stream).unwrap();
    for name in ["Direction", "XDirection", "YDirection"] {
        assert!(stream.text().contains(&format!(", \"{name}\"")));
        assert!(!stream.text().contains(&format!("]\"{name}\"")));
    }
    let mut restored = Axis3::default();
    let mut position = 1;
    assert!(
        restored
            .init_from_json(stream.text(), &mut position)
            .unwrap()
    );
    assert!(axis.location().is_equal(&restored.location(), 1e-10));
    assert!(axis.direction().is_equal(&restored.direction(), 1e-10));
}
#[test]
fn Standard_DumpTest__Bnd_Box_ComplexDump() {
    let mut bounds = BoundingBox3::new(
        Point3::new(-5.0, -10.0, -15.0),
        Point3::new(20.0, 30.0, 40.0),
    );
    bounds.set_gap(1.5);
    let mut stream = DumpStream::default();
    bounds.dump_json(&mut stream).unwrap();
    for name in ["CornerMax", "Gap", "Flags"] {
        assert!(stream.text().contains(&format!(", \"{name}\"")));
    }
    let mut restored = BoundingBox3::default();
    let mut position = 1;
    assert!(
        restored
            .init_from_json(stream.text(), &mut position)
            .unwrap()
    );
    let original = bounds.get().unwrap();
    let copy = restored.get().unwrap();
    for index in 0..6 {
        assert_eq!(original[index], copy[index]);
    }
    assert_eq!(bounds.gap(), restored.gap());
}
#[test]
fn Standard_DumpTest__TellP_BehaviorValidation() {
    let mut stream = DumpStream::default();
    let first = stream.position();
    assert!(first == 0 || first == -1);
    stream.append("test");
    assert!(stream.position() > 0);
    let mut empty = DumpStream::default();
    add_values_separator(&mut empty).unwrap();
    assert!(empty.text().is_empty());
    let mut content = DumpStream::default();
    content.append("content");
    let before = content.text().to_owned();
    add_values_separator(&mut content).unwrap();
    assert_ne!(before, content.text());
}
#[test]
fn Standard_DumpTest__ConsecutiveDumps() {
    let mut stream = DumpStream::default();
    Point3::new(1.0, 2.0, 3.0).dump_json(&mut stream).unwrap();
    add_values_separator(&mut stream).unwrap();
    Point3::new(4.0, 5.0, 6.0).dump_json(&mut stream).unwrap();
    assert!(stream.text().contains(", \"gp_Pnt\""));
}
#[test]
fn Standard_DumpTest__StreamWithOnlyOpenBrace() {
    let mut stream = DumpStream::default();
    stream.append("{");
    let before = stream.text().to_owned();
    add_values_separator(&mut stream).unwrap();
    assert_eq!(before, stream.text());
}
#[test]
fn Standard_DumpTest__VoidBoxSerialization() {
    let bounds = BoundingBox3::default();
    let mut stream = DumpStream::default();
    bounds.dump_json(&mut stream).unwrap();
    assert!(!stream.text().is_empty());
    let mut restored = BoundingBox3::default();
    let mut position = 1;
    assert!(
        restored
            .init_from_json(stream.text(), &mut position)
            .unwrap()
    );
    assert!(restored.is_void());
}
