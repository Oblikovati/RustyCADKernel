//! Wrapper state and history behavior from `BRepOffsetAPI_MakeOffsetShape`.
use rustycad_tkbrep::topods::Shape;
use rustycad_tkmath::gp::Plane3;
use rustycad_tkoffset::brep_offset_api::BRepOffsetAPIMakeOffsetShape;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape};

#[test]
fn default_build_and_history_match_native() -> Result<(), rustycad_tkernel::standard::Failure> {
    let mut maker = BRepOffsetAPIMakeOffsetShape::new();
    let null = Shape::default();
    assert!(!maker.is_done());
    assert!(maker.make_shape_base().my_shape.is_null());
    assert!(maker.generated(&null)?.is_empty());
    assert!(maker.modified(&null)?.is_empty());
    assert!(!maker.is_deleted(&null)?);
    maker.build(&Default::default())?;
    assert!(!maker.is_done());
    assert!(maker.make_shape_base().my_shape.is_null());
    assert_eq!(not_done_error(&mut maker), "StdFail_NotDone");
    Ok(())
}

#[test]
fn simple_perform_history_and_failed_reinitialization_match_native()
-> Result<(), rustycad_tkernel::standard::Failure> {
    let null = Shape::default();
    let mut maker = BRepOffsetAPIMakeOffsetShape::new();
    assert!(maker.generated(&null)?.is_empty());
    assert_null_run_history(&mut maker, &null)?;
    let source = plane_face()?;
    let previous = assert_plane_history(&mut maker, &source)?;
    assert_failed_rerun(&mut maker, &null, &previous)?;
    Ok(())
}

fn not_done_error(maker: &mut BRepOffsetAPIMakeOffsetShape) -> String {
    match maker.shape() {
        Err(error) => error.exception_type().to_owned(),
        Ok(_) => panic!("Shape() must fail while the builder is not done"),
    }
}

fn assert_null_run_history(
    maker: &mut BRepOffsetAPIMakeOffsetShape,
    null: &Shape,
) -> Result<(), rustycad_tkernel::standard::Failure> {
    maker.perform_by_simple(null, 1.0)?;
    assert!(!maker.is_done() && maker.make_shape_base().my_shape.is_null());
    assert_generated_unrecorded_error(maker, null);
    assert!(maker.modified(null)?.is_empty());
    assert!(!maker.is_deleted(null)?);
    Ok(())
}

fn assert_generated_unrecorded_error(maker: &mut BRepOffsetAPIMakeOffsetShape, null: &Shape) {
    let error = match maker.generated(null) {
        Err(error) => error,
        Ok(_) => panic!("Generated(null) after null perform must fail"),
    };
    assert_eq!(error.exception_type(), "Standard_NoSuchObject");
}

fn assert_plane_history(
    maker: &mut BRepOffsetAPIMakeOffsetShape,
    source: &Shape,
) -> Result<Shape, rustycad_tkernel::standard::Failure> {
    maker.perform_by_simple(source, 1.5)?;
    assert!(maker.is_done() && !maker.shape()?.is_null());
    assert_eq!(maker.generated(source)?.len(), 1);
    assert!(maker.modified(source)?.is_empty() && !maker.is_deleted(source)?);
    let previous = maker.shape()?.clone();
    maker.build(&Default::default())?;
    assert!(maker.is_done() && maker.shape()?.is_same(&previous));
    Ok(previous)
}

fn assert_failed_rerun(
    maker: &mut BRepOffsetAPIMakeOffsetShape,
    null: &Shape,
    previous: &Shape,
) -> Result<(), rustycad_tkernel::standard::Failure> {
    maker.perform_by_simple(null, 3.0)?;
    assert!(!maker.is_done());
    assert!(maker.make_shape_base().my_shape.is_same(previous));
    assert_eq!(not_done_error(maker), "StdFail_NotDone");
    Ok(())
}

fn plane_face() -> Result<Shape, rustycad_tkernel::standard::Failure> {
    let mut maker =
        BRepBuilderAPIMakeFace::from_plane_bounds(Plane3::default(), (0.0, 1.0, 0.0, 2.0))?;
    Ok(maker.shape()?.clone())
}
