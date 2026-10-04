//! Differential fixture for a shared conical apex across sewn offset faces.
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::{brep::BRepTool, topods};
use rustycad_tkernel::{message::ProgressRange, standard::Handle};
use rustycad_tkg3d::geom::{GeomConicalSurface, GeomSurface};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::geom_abs::JoinType;
use rustycad_tkmath::gp::{Axis3, Direction3, Point3};
use rustycad_tkoffset::brep_offset::{BRepOffsetMakeOffset, BRepOffsetMode, BRepOffsetOffsetFace};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeFace, BRepBuilderAPISewing};
use rustycad_tktopalgo::{brep_check::BRepCheckAnalyzer, brep_gprop::BRepGProp};

#[test]
fn sewn_half_cones_form_one_shared_source_apex() -> Result<(), rustycad_tkernel::standard::Failure>
{
    let input = sewn_half_cones()?;
    assert_eq!(count(&input, ShapeEnum::Face), 2);
    assert_eq!(count(&input, ShapeEnum::Edge), 8);
    assert_eq!(count_degenerated(&input), 2);
    Ok(())
}

#[test]
fn each_sewn_cone_half_has_a_regular_offset_image()
-> Result<(), rustycad_tkernel::standard::Failure> {
    let input = sewn_half_cones()?;
    let images = offset_face_images(&input)?;
    assert_eq!(images.len(), 2);
    assert!(
        images
            .iter()
            .all(|image| count(&image.face().clone().into_shape(), ShapeEnum::Edge) == 4)
    );
    Ok(())
}

#[test]
fn standard_offset_groups_sewn_cone_apices_across_faces()
-> Result<(), rustycad_tkernel::standard::Failure> {
    let input = sewn_half_cones()?;
    let mut maker = BRepOffsetMakeOffset::new();
    maker.initialize(
        &input,
        1.0,
        1.0e-7,
        BRepOffsetMode::Skin,
        false,
        false,
        JoinType::Arc,
        false,
        false,
    );
    maker.make_offset_shape()?;
    assert!(maker.is_done());
    assert_eq!(count(maker.shape(), ShapeEnum::Face), 3);
    assert_eq!(count(maker.shape(), ShapeEnum::Edge), 13);
    assert_eq!(count_degenerated(maker.shape()), 1);
    assert!(BRepCheckAnalyzer::new(maker.shape(), true, false, false)?.is_valid()?);
    assert_native_area(maker.shape())?;
    Ok(())
}

fn assert_native_area(shape: &topods::Shape) -> Result<(), rustycad_tkernel::standard::Failure> {
    let mut properties = GProps::default();
    BRepGProp::surface_properties(shape, &mut properties, false, false)?;
    assert!((properties.mass() - 209.02697939643053).abs() < 1.0e-10);
    Ok(())
}

fn offset_face_images(
    shape: &topods::Shape,
) -> Result<Vec<BRepOffsetOffsetFace>, rustycad_tkernel::standard::Failure> {
    Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)?
        .map(|item| BRepOffsetOffsetFace::from_face(&topods::Face::unchecked(item), 1.0))
        .collect()
}

fn sewn_half_cones() -> Result<topods::Shape, rustycad_tkernel::standard::Failure> {
    let axis = Axis3::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0)?,
        Direction3::new(1.0, 0.0, 0.0)?,
    )?;
    let surface: Handle<dyn GeomSurface> =
        Handle::new(GeomConicalSurface::new(axis, 0.5, 0.0)?).cast();
    let first = BRepBuilderAPIMakeFace::from_surface_bounds(
        &surface,
        (0.0, std::f64::consts::PI, 0.0, 10.0),
        1.0e-7,
    )?;
    let second = BRepBuilderAPIMakeFace::from_surface_bounds(
        &surface,
        (std::f64::consts::PI, 2.0 * std::f64::consts::PI, 0.0, 10.0),
        1.0e-7,
    )?;
    let mut sewing = BRepBuilderAPISewing::new(1.0e-6, true, true, true, false);
    sewing.add(&first.face()?.into_shape())?;
    sewing.add(&second.face()?.into_shape())?;
    sewing.perform(&ProgressRange::default())?;
    Ok(sewing.sewed_shape().clone())
}

fn count(shape: &topods::Shape, kind: ShapeEnum) -> usize {
    Explorer::new(shape, kind, ShapeEnum::Shape)
        .unwrap()
        .count()
}

fn count_degenerated(shape: &topods::Shape) -> usize {
    Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)
        .unwrap()
        .filter(|shape| BRepTool::degenerated(&topods::Edge::unchecked(shape.clone())).unwrap())
        .count()
}
