//! BRepFill_TrimShellCorner: the trimming of the faces met at a corner of a pipe (its
//! full runs are compared with OCCT through the right and round corners of the sweep
//! probe; these tests pin its contract).
use rustycad_tkbool::brep_fill::{BRepFillTrimShellCorner, TransitionStyle};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::{Array2, HArray2};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Axis2, Direction3, Point3, Vector3};

/// A table of null shapes with the bounds.
fn table(rows: i32, cols: i32) -> Result<Handle<HArray2<Shape>>, Failure> {
    let array =
        Array2::with_bounds(1, rows, 1, cols).map_err(|e| Failure::out_of_range(&e.to_string()))?;
    Ok(Handle::new(HArray2::from_owned(array)))
}

/// The bisector plane frame of the tests.
fn axe() -> Result<Axis2, Failure> {
    Axis2::new(Point3::new(0., 0., 1.), Direction3::Z)
}

#[test]
fn faces_not_in_two_columns_are_not_trimmed() -> Result<(), Failure> {
    let mut trim = BRepFillTrimShellCorner::new(
        &table(2, 3)?,
        TransitionStyle::Right,
        &axe()?,
        &Vector3::new(0., 1., 0.),
    )?;
    trim.add_bounds(&table(2, 2)?)?;
    trim.add_u_edges(&table(3, 2)?)?;
    trim.add_v_edges(&table(2, 3)?, 2)?;
    trim.perform()?;
    assert!(!trim.is_done());
    assert!(!trim.has_section());
    assert!(trim.modified(&Shape::default()).is_empty());
    Ok(())
}

#[test]
fn a_null_table_is_refused() -> Result<(), Failure> {
    let null: Handle<HArray2<Shape>> = Handle::default();
    let error = BRepFillTrimShellCorner::new(
        &null,
        TransitionStyle::Round,
        &axe()?,
        &Vector3::new(0., 1., 0.),
    )
    .err();
    assert_eq!(
        error.map(|e| e.exception_type().to_string()),
        Some("Standard_NullObject".to_string())
    );
    let mut trim = BRepFillTrimShellCorner::new(
        &table(1, 2)?,
        TransitionStyle::Round,
        &axe()?,
        &Vector3::new(0., 1., 0.),
    )?;
    assert!(trim.add_bounds(&null).is_err());
    assert!(trim.add_u_edges(&null).is_err());
    assert!(trim.add_v_edges(&null, 1).is_err());
    // A v-edge column outside the table is out of range.
    assert!(trim.add_v_edges(&table(1, 2)?, 3).is_err());
    Ok(())
}
