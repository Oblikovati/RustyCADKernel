//! Independent closed-form checks of BRepOffsetAPI_MakeThickSolid.
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkoffset::brep_offset::MakeOffsetOptions;
use rustycad_tkoffset::brep_offset_api::BRepOffsetAPIMakeThickSolid;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
use rustycad_tktopalgo::brep_gprop::BRepGProp;

/// The inward offset of the box of side 20 by -1 through its first face keeps sharp
/// inner corners (the concave edges are intersected), so the cavity is the box
/// 18 x 18 x 19 and the wall volume is 20^3 - 18 * 18 * 19 = 1844.
#[test]
fn hollow_box_wall_volume_is_the_closed_form() -> Result<(), Failure> {
    let a_box = BRepPrimAPIMakeBox::from_sizes(20.0, 20.0, 20.0)?.shape()?.clone();
    let face = Explorer::new(&a_box, ShapeEnum::Face, ShapeEnum::Shape)?.current().clone();
    let mut faces_to_remove = List::default();
    faces_to_remove.append(face);
    let mut maker = BRepOffsetAPIMakeThickSolid::new();
    maker.make_thick_solid_by_join(
        &a_box,
        &faces_to_remove,
        -1.0,
        precision::confusion(),
        MakeOffsetOptions::default(),
        &ProgressRange::default(),
    )?;
    maker.build(&ProgressRange::default())?;
    let result = maker.shape()?.clone();
    let mut props = GProps::new();
    BRepGProp::volume_properties(&result, &mut props, false, false, false)?;
    let wall = props.mass();
    assert!((wall - 1844.0).abs() < 1.0e-6 * 1844.0, "wall volume {wall}");
    Ok(())
}
