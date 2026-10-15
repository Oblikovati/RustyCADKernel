//! Independent closed-form checks of BRepOffsetAPI_ThruSections: ruled solids whose
//! volumes are known exactly (a conical frustum, a square prism, a square pyramid with a
//! vertex apex), measured by error-bounded integration, with the face count and the
//! validity of each solid.
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Shape, Vertex, Wire};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::gp::{Axis2, Circle3, Direction3, Point3};
use rustycad_tkoffset::brep_offset_api::BRepOffsetAPIThruSections;
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakePolygon, BRepBuilderAPIMakeShape,
    BRepBuilderAPIMakeVertex, BRepBuilderAPIMakeWire,
};
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;
use rustycad_tktopalgo::brep_gprop::BRepGProp;

/// A circle wire of the radius around the Z axis at the height.
fn circle(z: f64, radius: f64) -> Result<Wire, Failure> {
    let axis = Axis2::new(Point3::new(0.0, 0.0, z), Direction3::new(0.0, 0.0, 1.0)?)?;
    let edge = BRepBuilderAPIMakeEdge::from_circ(Circle3::new(axis, radius)?)?.edge()?;
    BRepBuilderAPIMakeWire::from_edge(&edge)?.wire()
}

/// A closed square wire of side `2 h` around the Z axis at the height.
fn square(z: f64, h: f64) -> Result<Wire, Failure> {
    BRepBuilderAPIMakePolygon::from_four_points(
        Point3::new(-h, -h, z),
        Point3::new(h, -h, z),
        Point3::new(h, h, z),
        Point3::new(-h, h, z),
        true,
    )?
    .wire()
}

/// The ruled solid loft of the wires, built and valid.
fn ruled_solid(wires: &[Wire], apex: Option<Vertex>) -> Result<Shape, Failure> {
    let mut loft = BRepOffsetAPIThruSections::new(true, true, 1.0e-6);
    for wire in wires {
        loft.add_wire(wire);
    }
    if let Some(apex) = apex {
        loft.add_vertex(&apex)?;
    }
    let solid = loft.shape()?.clone();
    assert!(BRepCheckAnalyzer::new(&solid, true, false, false)?.is_valid()?);
    Ok(solid)
}

/// Asserts the error-bounded volume of the solid against the closed form.
fn assert_volume(solid: &Shape, exact: f64) -> Result<(), Failure> {
    let mut props = GProps::default();
    let error = BRepGProp::volume_properties_eps(solid, &mut props, 1.0e-9, false, false)?;
    let volume = props.mass();
    let bound = (error.abs() + 1.0e-9) * volume.abs();
    assert!((volume - exact).abs() <= bound, "volume {volume}, exact {exact}, bound {bound}");
    Ok(())
}

/// The number of faces of a shape.
fn face_count(shape: &Shape) -> Result<usize, Failure> {
    Ok(Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)?.count())
}

#[test]
fn ruled_frustum_has_the_closed_form_volume() -> Result<(), Failure> {
    // Radii 2 and 1, height 3: V = pi h (r1^2 + r1 r2 + r2^2) / 3 = 7 pi.
    let solid = ruled_solid(&[circle(0.0, 2.0)?, circle(3.0, 1.0)?], None)?;
    assert_eq!(face_count(&solid)?, 3);
    assert_volume(&solid, 7.0 * std::f64::consts::PI)
}

#[test]
fn ruled_square_prism_has_the_closed_form_volume() -> Result<(), Failure> {
    // Side 2, height 1.5: V = 6.
    let solid = ruled_solid(&[square(0.0, 1.0)?, square(1.5, 1.0)?], None)?;
    assert_eq!(face_count(&solid)?, 6);
    assert_volume(&solid, 6.0)
}

#[test]
fn ruled_pyramid_to_a_vertex_has_the_closed_form_volume() -> Result<(), Failure> {
    // Base side 2, apex height 1.5: V = 4 * 1.5 / 3 = 2.
    let apex = BRepBuilderAPIMakeVertex::new(Point3::new(0.0, 0.0, 1.5))?.vertex()?;
    let solid = ruled_solid(&[square(0.0, 1.0)?], Some(apex))?;
    assert_eq!(face_count(&solid)?, 5);
    assert_volume(&solid, 2.0)
}

#[test]
fn inner_punctual_section_is_wrong_usage() -> Result<(), Failure> {
    let apex = BRepBuilderAPIMakeVertex::new(Point3::new(0.0, 0.0, 1.0))?.vertex()?;
    let mut loft = BRepOffsetAPIThruSections::new(true, false, 1.0e-6);
    loft.add_wire(&square(0.0, 1.0)?);
    loft.add_vertex(&apex)?;
    loft.add_wire(&square(2.0, 1.0)?);
    loft.build(&rustycad_tkernel::message::ProgressRange::default())?;
    assert!(!loft.is_done());
    assert_eq!(
        loft.get_status(),
        rustycad_tkbool::brep_fill::ThruSectionErrorStatus::WrongUsage
    );
    Ok(())
}
