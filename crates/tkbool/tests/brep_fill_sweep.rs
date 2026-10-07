//! BRepFill_Sweep: the shell or wire of a section law swept along a location law, with
//! the corners of the right and round transitions trimmed by BRepFill_TrimShellCorner.
use rustycad_tkbool::brep_fill::{
    BRepFillEdge3DLaw, BRepFillLocationLaw, BRepFillSectionLaw, BRepFillShapeLaw, BRepFillSweep,
    SweepBuildOptions, SweepHistory, SweepTablesMap, TransitionStyle,
};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Shape, Wire};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeomalgo::geom_fill::{
    GeomFillCorrectedFrenet, GeomFillCurveAndTrihedron, GeomFillLocationLaw, GeomFillTrihedronLaw,
};
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakePolygon, BRepBuilderAPIMakeVertex};

/// The polyline through the points (closed on request).
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

/// The location law of the path with the corrected Frenet trihedron.
fn location(path: &Wire) -> Result<Handle<dyn BRepFillLocationLaw>, Failure> {
    let tri: Handle<dyn GeomFillTrihedronLaw> = Handle::new(GeomFillCorrectedFrenet::new()).cast();
    let model: Handle<dyn GeomFillLocationLaw> =
        Handle::new(GeomFillCurveAndTrihedron::new(&tri)).cast();
    Ok(Handle::new(BRepFillEdge3DLaw::new(path, &model)?).cast())
}

/// The square section of side 0.2 around the origin of the local frame.
fn square_section() -> Result<Handle<dyn BRepFillSectionLaw>, Failure> {
    let square = polyline(
        &[
            (-0.1, -0.1, 0.),
            (0.1, -0.1, 0.),
            (0.1, 0.1, 0.),
            (-0.1, 0.1, 0.),
        ],
        true,
    )?;
    Ok(Handle::new(BRepFillShapeLaw::with_wire(&square, true)?).cast())
}

/// The count of the sub-shapes of a kind.
fn count(shape: &Shape, kind: ShapeEnum) -> Result<usize, Failure> {
    Ok(Explorer::new(shape, kind, ShapeEnum::Shape)?.count())
}

/// The sweep built with the transition.
fn built(
    sec: &Handle<dyn BRepFillSectionLaw>,
    path: &Wire,
    transition: TransitionStyle,
) -> Result<BRepFillSweep, Failure> {
    let mut sweep = BRepFillSweep::new(sec, &location(path)?, true);
    let (mut reversed, mut tapes, mut rails) = (
        MapOfShape::default(),
        SweepTablesMap::default(),
        SweepTablesMap::default(),
    );
    let mut hist = SweepHistory {
        reversed_edges: &mut reversed,
        tapes: &mut tapes,
        rails: &mut rails,
    };
    let options = SweepBuildOptions {
        transition,
        ..SweepBuildOptions::default()
    };
    sweep.build(&mut hist, &options)?;
    Ok(sweep)
}

/// The bounds of a table of the sweep.
fn bounds(table: &Handle<rustycad_tkernel::ncollection::HArray2<Shape>>) -> Option<[i32; 4]> {
    let t = table.get()?.array2();
    Some([t.lower_row(), t.upper_row(), t.lower_col(), t.upper_col()])
}

#[test]
fn a_new_sweep_is_not_done_and_has_no_tables() -> Result<(), Failure> {
    let sweep = BRepFillSweep::new(
        &square_section()?,
        &location(&polyline(&[(0., 0., 0.), (0., 0., 1.)], false)?)?,
        true,
    );
    assert!(!sweep.is_done());
    assert!(sweep.shape().is_null());
    assert!(
        sweep.sub_shape().is_null() && sweep.inter_faces().is_null() && sweep.sections().is_null()
    );
    assert_eq!(
        sweep.tape(1).map_err(|e| e.exception_type().to_string()),
        Err("Standard_NullObject".to_string())
    );
    Ok(())
}

#[test]
fn a_square_along_a_segment_makes_a_tube_of_four_faces() -> Result<(), Failure> {
    let sweep = built(
        &square_section()?,
        &polyline(&[(0., 0., 0.), (0., 0., 3.)], false)?,
        TransitionStyle::Modified,
    )?;
    assert!(sweep.is_done());
    let shell = sweep.shape();
    assert_eq!(shell.shape_type()?, ShapeEnum::Shell);
    assert_eq!(count(&shell, ShapeEnum::Face)?, 4);
    assert_eq!(bounds(&sweep.sub_shape()), Some([1, 4, 1, 1]));
    assert_eq!(bounds(&sweep.inter_faces()), Some([1, 5, 1, 1]));
    assert_eq!(bounds(&sweep.sections()), Some([1, 4, 1, 2]));
    for i in 1..=4 {
        assert_eq!(count(&sweep.tape(i)?, ShapeEnum::Face)?, 1);
    }
    assert_eq!(
        sweep.tape(5).map_err(|e| e.exception_type().to_string()),
        Err("Standard_OutOfRange".to_string())
    );
    // A planar sweep is exact.
    assert!(sweep.error_on_surface() < 1e-9);
    Ok(())
}

#[test]
fn a_vertex_section_sweeps_to_a_wire() -> Result<(), Failure> {
    let vertex = BRepBuilderAPIMakeVertex::new(Point3::new(0.1, 0., 0.))?.vertex()?;
    let sec: Handle<dyn BRepFillSectionLaw> =
        Handle::new(BRepFillShapeLaw::with_vertex(&vertex, true)?).cast();
    let path = polyline(&[(0., 0., 0.), (0., 0., 2.), (1., 0., 3.)], false)?;
    let sweep = built(&sec, &path, TransitionStyle::Modified)?;
    assert!(sweep.is_done());
    assert_eq!(sweep.shape().shape_type()?, ShapeEnum::Wire);
    assert_eq!(count(&sweep.shape(), ShapeEnum::Edge)?, 2);
    assert_eq!(bounds(&sweep.sub_shape()), Some([1, 1, 1, 2]));
    Ok(())
}

#[test]
fn the_right_and_round_corners_of_a_bent_path_are_trimmed() -> Result<(), Failure> {
    let path = polyline(&[(0., 0., 0.), (0., 0., 2.), (1., 0., 3.)], false)?;
    let modified = built(&square_section()?, &path, TransitionStyle::Modified)?;
    let right = built(&square_section()?, &path, TransitionStyle::Right)?;
    let round = built(&square_section()?, &path, TransitionStyle::Round)?;
    for sweep in [&modified, &right, &round] {
        assert!(sweep.is_done());
        assert_eq!(bounds(&sweep.sub_shape()), Some([1, 4, 1, 2]));
    }
    assert_eq!(count(&modified.shape(), ShapeEnum::Face)?, 8);
    assert_eq!(count(&right.shape(), ShapeEnum::Face)?, 8);
    // The round corner adds the filling faces of revolution.
    assert!(count(&round.shape(), ShapeEnum::Face)? > 8);
    Ok(())
}

#[test]
fn the_angular_control_is_clamped() -> Result<(), Failure> {
    let path = polyline(&[(0., 0., 0.), (0., 0., 2.), (1., 0., 3.)], false)?;
    let mut sweep = BRepFillSweep::new(&square_section()?, &location(&path)?, true);
    // An angle above 6.28 or below Precision::Angular() is clamped (no observable state
    // but the corner choice): with the largest angle the right corner stays right.
    sweep.set_angular_control(0., 100.);
    sweep.set_tolerance(1e-4, 1.0, 1e-5, 1e-2);
    sweep.set_force_approx_c1(false);
    let (mut reversed, mut tapes, mut rails) = (
        MapOfShape::default(),
        SweepTablesMap::default(),
        SweepTablesMap::default(),
    );
    let mut hist = SweepHistory {
        reversed_edges: &mut reversed,
        tapes: &mut tapes,
        rails: &mut rails,
    };
    let options = SweepBuildOptions {
        transition: TransitionStyle::Right,
        ..SweepBuildOptions::default()
    };
    sweep.build(&mut hist, &options)?;
    assert!(sweep.is_done());
    assert_eq!(count(&sweep.shape(), ShapeEnum::Face)?, 8);
    Ok(())
}
