//! BRepFill_AdvancedEvolved: the evolved solid of a profile swept along a planar spine.
use rustycad_tkbool::brep_fill::{AdvancedEvolvedStatus, BRepFillAdvancedEvolved};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Face, Shape, Wire};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakePolygon;

/// The polyline wire through the points (closed when asked).
fn polyline(points: &[(f64, f64, f64)], closed: bool) -> Result<Wire, Failure> {
    let mut polygon = BRepBuilderAPIMakePolygon::new();
    for &(x, y, z) in points {
        polygon.add_point(Point3::new(x, y, z))?;
    }
    if closed {
        polygon.close()?;
    }
    polygon.wire()
}

/// The square spine of side 4 on z = 0.
fn square() -> Result<Wire, Failure> {
    polyline(
        &[(0., 0., 0.), (4., 0., 0.), (4., 4., 0.), (0., 4., 0.)],
        true,
    )
}

/// The count of the sub-shapes of a kind.
fn count(shape: &Shape, kind: ShapeEnum) -> Result<usize, Failure> {
    Ok(Explorer::new(shape, kind, ShapeEnum::Shape)?.count())
}

#[test]
fn a_vertical_profile_along_a_square_gives_a_solid() -> Result<(), Failure> {
    let mut ae = BRepFillAdvancedEvolved::new();
    ae.set_parallel_mode(false);
    ae.perform(
        &square()?,
        &polyline(&[(0., 0., 0.), (0., 0., 1.)], false)?,
        1e-7,
        true,
    )?;
    let mut code: u32 = 0;
    assert!(ae.is_done(Some(&mut code)));
    assert_eq!(code, AdvancedEvolvedStatus::Ok as u32);
    assert_eq!(count(ae.shape(), ShapeEnum::Solid)?, 1);
    Ok(())
}

#[test]
fn the_shell_mode_drops_the_lids() -> Result<(), Failure> {
    let profile = polyline(&[(0., 0., 0.), (0., 0., 1.)], false)?;
    let mut solid = BRepFillAdvancedEvolved::new();
    solid.perform(&square()?, &profile, 1e-7, true)?;
    let mut shell = BRepFillAdvancedEvolved::new();
    shell.perform(&square()?, &profile, 1e-7, false)?;
    assert!(shell.is_done(None));
    assert_eq!(shell.shape().shape_type()?, ShapeEnum::Shell);
    let solid_faces = count(solid.shape(), ShapeEnum::Face)?;
    assert!(count(shell.shape(), ShapeEnum::Face)? < solid_faces);
    Ok(())
}

#[test]
fn a_closed_profile_needs_no_lids() -> Result<(), Failure> {
    let mut ae = BRepFillAdvancedEvolved::new();
    let triangle = polyline(&[(0., 0., 0.), (0.5, 0., 0.), (0., 0., 1.)], true)?;
    ae.perform(&square()?, &triangle, 1e-7, true)?;
    assert!(ae.is_done(None));
    // As natively (bf_advanced_evolved): the swept triangle is cut into four solids at
    // the corners, kept as a compound.
    assert_eq!(count(ae.shape(), ShapeEnum::Solid)?, 4);
    Ok(())
}

#[test]
fn nothing_is_done_before_perform() -> Result<(), Failure> {
    let ae = BRepFillAdvancedEvolved::new();
    let mut code: u32 = 7;
    assert!(!ae.is_done(Some(&mut code)));
    assert_eq!(code, AdvancedEvolvedStatus::Empty as u32);
    assert!(ae.shape().is_null());
    // No lid without lids.
    assert!(!ae.is_lid(&Face::default(), &IndexedMapOfShape::default())?);
    Ok(())
}
