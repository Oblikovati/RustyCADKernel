//! Stateful shell orchestration matching `BRepOffset_MakeSimpleOffset`.
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{Face, Shape};
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::gp::{Plane3, Point3};
use rustycad_tkoffset::brep_offset::{BRepOffsetMakeSimpleOffset, BRepOffsetSimpleStatus};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape};
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;
use rustycad_tktopalgo::brep_gprop::BRepGProp;

#[test]
fn default_and_null_perform_match_native_state() {
    let mut maker = BRepOffsetMakeSimpleOffset::new();
    assert_eq!(maker.error(), BRepOffsetSimpleStatus::Ok);
    assert_eq!(maker.error_message(), "");
    assert_eq!(maker.offset_value(), 0.0);
    assert_eq!(maker.tolerance(), 1.0e-7);
    assert!(!maker.is_done() && !maker.build_solid_flag());
    assert!(maker.result_shape().is_null());
    maker.perform().unwrap();
    assert_eq!(maker.error(), BRepOffsetSimpleStatus::NullInputShape);
    assert_eq!(maker.error_message(), "Null input shape");
    assert!(!maker.is_done() && maker.result_shape().is_null());
}

#[test]
fn generated_before_perform_uses_native_unrecorded_shape_error()
-> Result<(), rustycad_tkernel::standard::Failure> {
    let source = plane_face()?;
    let maker = BRepOffsetMakeSimpleOffset::new();
    let failure = maker.generated(&source).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_NoSuchObject");
    Ok(())
}

#[test]
fn shell_perform_and_reinitialize_preserve_native_result_state()
-> Result<(), rustycad_tkernel::standard::Failure> {
    let source = plane_face()?;
    let mut maker = BRepOffsetMakeSimpleOffset::with_shape(&source, 1.5);
    maker.set_tolerance(2.0e-6);
    let generated = run_shell_offset(&mut maker, &source)?;
    reinitialize_and_check_history(&mut maker, &source, &generated)?;
    Ok(())
}

fn run_shell_offset(
    maker: &mut BRepOffsetMakeSimpleOffset,
    source: &Shape,
) -> Result<Shape, rustycad_tkernel::standard::Failure> {
    maker.perform()?;
    assert!(maker.is_done());
    assert_eq!(maker.error(), BRepOffsetSimpleStatus::Ok);
    assert!(!maker.result_shape().is_null());
    let output_face = Face::unchecked(maker.result_shape().clone());
    let surface = BRepTool::surface(&output_face)?;
    assert_eq!(surface.value(0.5, 1.0)?, Point3::new(0.5, 1.0, 1.5));
    let generated = maker.generated(source)?;
    assert!(!generated.is_null() && !generated.is_same(source));
    assert!(maker.modified(source)?.is_null());
    Ok(generated)
}

fn reinitialize_and_check_history(
    maker: &mut BRepOffsetMakeSimpleOffset,
    source: &Shape,
    generated: &Shape,
) -> Result<(), rustycad_tkernel::standard::Failure> {
    maker.set_build_solid_flag(true);
    maker.initialize(&Shape::default(), 3.0);
    assert_eq!(maker.offset_value(), 3.0);
    assert_eq!(maker.tolerance(), 2.0e-6);
    assert!(maker.build_solid_flag());
    assert!(!maker.is_done() && maker.result_shape().is_same(generated));
    assert!(maker.generated(source)?.is_same(generated));
    maker.perform()?;
    assert_eq!(maker.error(), BRepOffsetSimpleStatus::NullInputShape);
    assert!(!maker.is_done() && maker.result_shape().is_same(generated));
    assert!(maker.generated(source)?.is_same(generated));
    Ok(())
}

#[test]
fn solid_wall_path_builds_a_closed_native_result() -> Result<(), rustycad_tkernel::standard::Failure>
{
    let source = plane_face()?;
    let mut maker = BRepOffsetMakeSimpleOffset::with_shape(&source, 1.5);
    maker.set_build_solid_flag(true);
    maker.perform()?;
    assert_eq!(maker.error(), BRepOffsetSimpleStatus::Ok);
    assert!(maker.is_done());
    assert_eq!(
        maker.result_shape().shape_type()?,
        rustycad_tkg3d::top_abs::ShapeEnum::Solid
    );
    let analyzer = BRepCheckAnalyzer::new(maker.result_shape(), true, false, false)?;
    assert!(analyzer.is_valid()?);
    let mut properties = GProps::default();
    BRepGProp::volume_properties(maker.result_shape(), &mut properties, false, false, false)?;
    assert_eq!(properties.mass(), -3.0);
    Ok(())
}

#[test]
fn closed_input_reports_multiple_shells_like_native()
-> Result<(), rustycad_tkernel::standard::Failure> {
    let mut box_maker = BRepPrimAPIMakeBox::from_sizes(1.0, 1.0, 1.0)?;
    let mut maker = BRepOffsetMakeSimpleOffset::with_shape(box_maker.shape()?, 0.1);
    maker.set_build_solid_flag(true);
    maker.perform()?;
    assert_eq!(maker.error(), BRepOffsetSimpleStatus::ErrorInvalidNbShells);
    assert!(!maker.is_done());
    assert_eq!(
        maker.result_shape().shape_type()?,
        rustycad_tkg3d::top_abs::ShapeEnum::Solid
    );
    Ok(())
}

fn plane_face() -> Result<Shape, rustycad_tkernel::standard::Failure> {
    let mut maker =
        BRepBuilderAPIMakeFace::from_plane_bounds(Plane3::default(), (0.0, 1.0, 0.0, 2.0))?;
    Ok(maker.shape()?.clone())
}
