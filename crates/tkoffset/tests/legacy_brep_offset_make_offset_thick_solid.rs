//! Exact translations of the thick-solid cases of BRepOffset_MakeOffset_Test.cxx at OCCT
//! 3d097a0 (the planar wire offsets are in legacy_brep_offset_make_offset.rs).
use rustycad_tkbo::brep_algo_api::BRepAlgoAPIFuse;
use rustycad_tkbrep::topods::{Shape, Wire};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::geom_abs::JoinType;
use rustycad_tkmath::gp::{Axis2, Circle3, Direction3, Ellipse3, Point3, Vector3};
use rustycad_tkoffset::brep_offset::{BRepOffsetMode, MakeOffsetOptions};
use rustycad_tkoffset::brep_offset_api::{BRepOffsetAPIMakeThickSolid, BRepOffsetAPIThruSections};
use rustycad_tkprim::brep_prim_api::{
    BRepPrimAPIMakeBox, BRepPrimAPIMakeCylinder, BRepPrimAPIMakeSphere,
};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakePolygon, BRepBuilderAPIMakeShape,
    BRepBuilderAPIMakeWire,
};
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;
use std::f64::consts::PI;

/// `MakeCircularWire(theCenter, theNormal, theRadius)`.
fn make_circular_wire(center: Point3, normal: Direction3, radius: f64) -> Result<Wire, Failure> {
    let circ = Circle3::new(Axis2::new(center, normal)?, radius)?;
    let edge_maker = BRepBuilderAPIMakeEdge::from_circ(circ)?;
    if !edge_maker.is_done() {
        return Ok(Wire::default());
    }
    let wire_maker = BRepBuilderAPIMakeWire::from_edge(&edge_maker.edge()?)?;
    if !wire_maker.is_done() {
        return Ok(Wire::default());
    }
    wire_maker.wire()
}

/// `MakeRectangularWire(theCenter, theNormal, theWidth, theHeight)`.
fn make_rectangular_wire(
    center: Point3,
    normal: Direction3,
    width: f64,
    height: f64,
) -> Result<Wire, Failure> {
    // The rectangle centred on the point in the plane of the normal.
    let axis = Axis2::new(center, normal)?;
    let (x_dir, y_dir) = (
        Vector3::from(axis.x_direction()),
        Vector3::from(axis.y_direction()),
    );
    let (half_w, half_h) = (width / 2.0, height / 2.0);
    let p1 = center.translated(x_dir * (-half_w) + y_dir * (-half_h));
    let p2 = center.translated(x_dir * half_w + y_dir * (-half_h));
    let p3 = center.translated(x_dir * half_w + y_dir * half_h);
    let p4 = center.translated(x_dir * (-half_w) + y_dir * half_h);
    let mut wire_maker = BRepBuilderAPIMakeWire::new();
    for (a, b) in [(p1, p2), (p2, p3), (p3, p4), (p4, p1)] {
        wire_maker.add_edge(&BRepBuilderAPIMakeEdge::from_points(a, b)?.edge()?)?;
    }
    if !wire_maker.is_done() {
        return Ok(Wire::default());
    }
    wire_maker.wire()
}

/// `MakeEllipticalWire(theCenter, theNormal, theMajorRadius, theMinorRadius)`.
fn make_elliptical_wire(
    center: Point3,
    normal: Direction3,
    major: f64,
    minor: f64,
) -> Result<Wire, Failure> {
    let ellipse = Ellipse3::new(Axis2::new(center, normal)?, major, minor)?;
    let edge_maker = BRepBuilderAPIMakeEdge::from_elips(ellipse)?;
    if !edge_maker.is_done() {
        return Ok(Wire::default());
    }
    let wire_maker = BRepBuilderAPIMakeWire::from_edge(&edge_maker.edge()?)?;
    if !wire_maker.is_done() {
        return Ok(Wire::default());
    }
    wire_maker.wire()
}

/// `MakePolygonalWire(theCenter, theNormal, theRadius, theNbSides)`.
fn make_polygonal_wire(
    center: Point3,
    normal: Direction3,
    radius: f64,
    nb_sides: i32,
) -> Result<Wire, Failure> {
    let axis = Axis2::new(center, normal)?;
    let (x_dir, y_dir) = (
        Vector3::from(axis.x_direction()),
        Vector3::from(axis.y_direction()),
    );
    let mut polygon = BRepBuilderAPIMakePolygon::new();
    for i in 0..=nb_sides {
        let angle = 2.0 * PI * f64::from(i) / f64::from(nb_sides);
        polygon.add_point(
            center.translated(x_dir * radius * angle.cos() + y_dir * radius * angle.sin()),
        )?;
    }
    polygon.close()?;
    if !polygon.is_done() {
        return Ok(Wire::default());
    }
    polygon.wire()
}

fn z() -> Direction3 {
    Direction3::new(0.0, 0.0, 1.0).expect("z")
}

/// The loft of the wires (solid, ruled or not), asserted done and non-null.
fn loft(wires: &[Wire], ruled: bool) -> Result<Shape, Failure> {
    let mut loft_maker = BRepOffsetAPIThruSections::new(true, ruled, 1.0e-6);
    for w in wires {
        assert!(!w.is_null(), "Failed to create a section wire");
        loft_maker.add_wire(w);
    }
    loft_maker.build(&ProgressRange::default())?;
    assert!(loft_maker.is_done(), "Failed to create loft");
    let shape = loft_maker.shape()?.clone();
    assert!(!shape.is_null(), "Loft shape is null");
    Ok(shape)
}

/// `MakeThickSolidByJoin(S, {}, offset, 1e-3, BRepOffset_Skin, intersection, false,
/// GeomAbs_Intersection)` then `Build()`.
fn thick(
    shape: &Shape,
    offset: f64,
    intersection: bool,
) -> Result<BRepOffsetAPIMakeThickSolid, Failure> {
    let mut thick_maker = BRepOffsetAPIMakeThickSolid::new();
    let options = MakeOffsetOptions {
        mode: BRepOffsetMode::Skin,
        intersection,
        self_inter: false,
        join: JoinType::Intersection,
        thickening: false,
        remove_int_edges: false,
    };
    thick_maker.make_thick_solid_by_join(
        shape,
        &List::default(),
        offset,
        1.0e-3,
        options,
        &ProgressRange::default(),
    )?;
    thick_maker.build(&ProgressRange::default())?;
    Ok(thick_maker)
}

fn is_valid(shape: &Shape) -> Result<bool, Failure> {
    BRepCheckAnalyzer::new(shape, true, false, false)?.is_valid()
}

/// The thick solid shape of a done maker, valid when not null.
fn assert_valid_when_done(thick_maker: &mut BRepOffsetAPIMakeThickSolid) -> Result<(), Failure> {
    if thick_maker.is_done() {
        let thick_shape = thick_maker.shape()?.clone();
        // The shape may be null even when done in some edge cases.
        if !thick_shape.is_null() {
            assert!(is_valid(&thick_shape)?, "Thick solid shape is not valid");
        }
    }
    Ok(())
}

/// The circle of radius 50 at the origin and the 80 x 60 rectangle at height 100.
fn circle_and_rectangle(height: f64) -> Result<[Wire; 2], Failure> {
    Ok([
        make_circular_wire(Point3::new(0., 0., 0.), z(), 50.0)?,
        make_rectangular_wire(Point3::new(0., 0., height), z(), 80.0, 60.0)?,
    ])
}

#[test]
fn thick_solid_circle_to_rectangle_loft() -> Result<(), Failure> {
    let loft_shape = loft(&circle_and_rectangle(100.0)?, false)?;
    // The loft shape is valid.
    assert!(is_valid(&loft_shape)?, "Loft shape is not valid");
    // The thickness operation, where the regression of commit 44be1230391 occurs.
    let mut thick_maker = thick(&loft_shape, 2.0, false)?;
    assert!(
        thick_maker.is_done(),
        "ThickSolid operation failed (mixed connectivity regression)"
    );
    assert_valid_when_done(&mut thick_maker)
}

#[test]
fn thick_solid_circle_to_rectangle_loft_intersection_mode() -> Result<(), Failure> {
    let loft_shape = loft(&circle_and_rectangle(100.0)?, false)?;
    let mut thick_maker = thick(&loft_shape, 2.0, true)?;
    assert!(
        thick_maker.is_done(),
        "ThickSolid operation with intersection mode failed"
    );
    assert_valid_when_done(&mut thick_maker)
}

#[test]
fn thick_solid_simple_box_baseline() -> Result<(), Failure> {
    let wire = make_rectangular_wire(Point3::new(0., 0., 0.), z(), 100.0, 100.0)?;
    let top_wire = make_rectangular_wire(Point3::new(0., 0., 50.), z(), 100.0, 100.0)?;
    let shape = loft(&[wire, top_wire], true)?;
    let mut thick_maker = thick(&shape, 2.0, false)?;
    assert!(
        thick_maker.is_done(),
        "ThickSolid on simple ruled solid failed"
    );
    if thick_maker.is_done() {
        assert!(!thick_maker.shape()?.is_null(), "Thick solid shape is null");
    }
    Ok(())
}

#[test]
fn thick_solid_circle_to_ellipse_loft() -> Result<(), Failure> {
    let wires = [
        make_circular_wire(Point3::new(0., 0., 0.), z(), 50.0)?,
        make_elliptical_wire(Point3::new(0., 0., 100.), z(), 60.0, 40.0)?,
    ];
    let thick_maker = thick(&loft(&wires, false)?, 2.0, false)?;
    // A smooth transition: no mixed connectivity.
    assert!(
        thick_maker.is_done(),
        "ThickSolid on circle-to-ellipse loft failed"
    );
    Ok(())
}

#[test]
fn thick_solid_circle_to_hexagon_loft() -> Result<(), Failure> {
    let wires = [
        make_circular_wire(Point3::new(0., 0., 0.), z(), 50.0)?,
        make_polygonal_wire(Point3::new(0., 0., 100.), z(), 50.0, 6)?,
    ];
    let thick_maker = thick(&loft(&wires, false)?, 2.0, false)?;
    assert!(
        thick_maker.is_done(),
        "ThickSolid on circle-to-hexagon loft failed"
    );
    Ok(())
}

#[test]
fn thick_solid_circle_to_square_loft() -> Result<(), Failure> {
    let wires = [
        make_circular_wire(Point3::new(0., 0., 0.), z(), 50.0)?,
        make_rectangular_wire(Point3::new(0., 0., 100.), z(), 80.0, 80.0)?,
    ];
    let thick_maker = thick(&loft(&wires, false)?, 2.0, false)?;
    assert!(
        thick_maker.is_done(),
        "ThickSolid on circle-to-square loft failed"
    );
    Ok(())
}

#[test]
fn thick_solid_ellipse_to_rectangle_loft() -> Result<(), Failure> {
    let wires = [
        make_elliptical_wire(Point3::new(0., 0., 0.), z(), 60.0, 30.0)?,
        make_rectangular_wire(Point3::new(0., 0., 100.), z(), 100.0, 50.0)?,
    ];
    let thick_maker = thick(&loft(&wires, false)?, 2.0, false)?;
    assert!(
        thick_maker.is_done(),
        "ThickSolid on ellipse-to-rectangle loft failed"
    );
    Ok(())
}

#[test]
fn thick_solid_three_section_loft() -> Result<(), Failure> {
    let wires = [
        make_circular_wire(Point3::new(0., 0., 0.), z(), 40.0)?,
        make_elliptical_wire(Point3::new(0., 0., 50.), z(), 50.0, 35.0)?,
        make_rectangular_wire(Point3::new(0., 0., 100.), z(), 80.0, 60.0)?,
    ];
    let thick_maker = thick(&loft(&wires, false)?, 2.0, false)?;
    assert!(
        thick_maker.is_done(),
        "ThickSolid on three-section loft failed"
    );
    Ok(())
}

#[test]
fn thick_solid_circle_to_rectangle_loft_ruled() -> Result<(), Failure> {
    let thick_maker = thick(&loft(&circle_and_rectangle(100.0)?, true)?, 2.0, false)?;
    assert!(
        thick_maker.is_done(),
        "ThickSolid on ruled circle-to-rectangle loft failed"
    );
    Ok(())
}

#[test]
fn thick_solid_circle_to_rectangle_loft_small_offset() -> Result<(), Failure> {
    let thick_maker = thick(&loft(&circle_and_rectangle(100.0)?, false)?, 0.5, false)?;
    assert!(thick_maker.is_done(), "ThickSolid with small offset failed");
    Ok(())
}

#[test]
fn thick_solid_circle_to_rectangle_loft_large_offset() -> Result<(), Failure> {
    let thick_maker = thick(&loft(&circle_and_rectangle(100.0)?, false)?, 10.0, false)?;
    assert!(thick_maker.is_done(), "ThickSolid with large offset failed");
    Ok(())
}

#[test]
fn thick_solid_circle_to_rectangle_loft_negative_offset() -> Result<(), Failure> {
    let thick_maker = thick(&loft(&circle_and_rectangle(100.0)?, false)?, -2.0, false)?;
    assert!(
        thick_maker.is_done(),
        "ThickSolid with negative offset failed"
    );
    Ok(())
}

#[test]
fn thick_solid_cylinder_baseline() -> Result<(), Failure> {
    let cylinder = BRepPrimAPIMakeCylinder::new(50.0, 100.0).shape()?.clone();
    assert!(!cylinder.is_null(), "Failed to create cylinder");
    let mut thick_maker = thick(&cylinder, 2.0, false)?;
    // Analytic surfaces should always work.
    assert!(thick_maker.is_done(), "ThickSolid on cylinder failed");
    if thick_maker.is_done() {
        assert!(
            is_valid(&thick_maker.shape()?.clone())?,
            "Thick cylinder is not valid"
        );
    }
    Ok(())
}

#[test]
fn thick_solid_sphere_baseline() -> Result<(), Failure> {
    let sphere = BRepPrimAPIMakeSphere::new(50.0)?.shape()?.clone();
    assert!(!sphere.is_null(), "Failed to create sphere");
    let mut thick_maker = thick(&sphere, 2.0, false)?;
    assert!(thick_maker.is_done(), "ThickSolid on sphere failed");
    if thick_maker.is_done() {
        assert!(
            is_valid(&thick_maker.shape()?.clone())?,
            "Thick sphere is not valid"
        );
    }
    Ok(())
}

#[test]
fn thick_solid_fused_box_cylinder() -> Result<(), Failure> {
    let a_box = BRepPrimAPIMakeBox::from_sizes(100.0, 100.0, 50.0)?
        .shape()?
        .clone();
    assert!(!a_box.is_null(), "Failed to create box");
    let axis = Axis2::new(Point3::new(50., 50., 50.), z())?;
    let cyl = BRepPrimAPIMakeCylinder::with_axes(axis, 30.0, 50.0)
        .shape()?
        .clone();
    assert!(!cyl.is_null(), "Failed to create cylinder");
    let fuser = BRepAlgoAPIFuse::from_shapes(&a_box, &cyl)?;
    assert!(fuser.is_done(), "Failed to fuse shapes");
    let fused_shape = fuser.shape().clone();
    let thick_maker = thick(&fused_shape, 2.0, false)?;
    // The fused analytic surfaces should work.
    assert!(
        thick_maker.is_done(),
        "ThickSolid on fused box+cylinder failed"
    );
    Ok(())
}

#[test]
fn thick_solid_circle_to_rectangle_thin_loft() -> Result<(), Failure> {
    let loft_shape = loft(&circle_and_rectangle(10.0)?, false)?;
    // The operation may or may not succeed on very thin geometry, but must not crash.
    let thick_maker = thick(&loft_shape, 1.0, false)?;
    let _ = thick_maker.is_done();
    Ok(())
}

#[test]
fn thick_solid_circle_to_rectangle_tall_loft() -> Result<(), Failure> {
    let thick_maker = thick(&loft(&circle_and_rectangle(500.0)?, false)?, 2.0, false)?;
    assert!(
        thick_maker.is_done(),
        "ThickSolid on tall circle-to-rectangle loft failed"
    );
    Ok(())
}

#[test]
fn thick_solid_circle_to_triangle_loft() -> Result<(), Failure> {
    let wires = [
        make_circular_wire(Point3::new(0., 0., 0.), z(), 50.0)?,
        make_polygonal_wire(Point3::new(0., 0., 100.), z(), 60.0, 3)?,
    ];
    let thick_maker = thick(&loft(&wires, false)?, 2.0, false)?;
    // The triangle has sharp 60-degree corners.
    assert!(
        thick_maker.is_done(),
        "ThickSolid on circle-to-triangle loft failed"
    );
    Ok(())
}

#[test]
fn thick_solid_circle_to_octagon_loft() -> Result<(), Failure> {
    let wires = [
        make_circular_wire(Point3::new(0., 0., 0.), z(), 50.0)?,
        make_polygonal_wire(Point3::new(0., 0., 100.), z(), 55.0, 8)?,
    ];
    let thick_maker = thick(&loft(&wires, false)?, 2.0, false)?;
    assert!(
        thick_maker.is_done(),
        "ThickSolid on circle-to-octagon loft failed"
    );
    Ok(())
}
