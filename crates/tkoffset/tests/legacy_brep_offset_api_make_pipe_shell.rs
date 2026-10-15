//! Exact translations of BRepOffsetAPI_MakePipeShell_Test.cxx at OCCT 3d097a0.
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape, Shell, Solid, Wire};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCurve, GeomPlane, GeomSurface};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gc::MakeArcOfCircle;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkgeomalgo::law::{LawFunction, LawLinear};
use rustycad_tkmath::gp::{Axis1, Axis2, Circle3, Direction3, Plane3, Point3, Transform3, Vector3};
use rustycad_tkoffset::brep_offset_api::BRepOffsetAPIMakePipeShell;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeSweep;
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape,
    BRepBuilderAPIMakeWire, BRepBuilderAPIPipeError, BRepBuilderAPITransform,
};
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;
use rustycad_tktopalgo::brep_gprop::BRepGProp;

/// `wireLength(theWire)`: the total linear length of a wire.
fn wire_length(the_wire: &Wire) -> Result<f64, Failure> {
    let mut a_props = GProps::default();
    BRepGProp::linear_properties(the_wire.as_ref(), &mut a_props, false, false)?;
    Ok(a_props.mass())
}

/// A wire of one circle edge.
fn circle_wire(circle: Circle3) -> Result<Wire, Failure> {
    let edge = BRepBuilderAPIMakeEdge::from_circ(circle)?.edge()?;
    BRepBuilderAPIMakeWire::from_edge(&edge)?.wire()
}

/// A linear scaling law over the curve range from 1 to `end_value`.
fn linear_law(curve: &Handle<dyn GeomCurve>, end_value: f64) -> Result<Handle<dyn LawFunction>, Failure> {
    let c = curve.get().ok_or_else(|| Failure::with_message("Standard_NullObject", "null spine"))?;
    let law = Handle::new(LawLinear::new());
    law.get()
        .ok_or_else(|| Failure::with_message("Standard_NullObject", "null law"))?
        .set(c.first_parameter(), 1.0, c.last_parameter(), end_value);
    Ok(law.cast())
}

/// A built pipe of the profile scaled by the law along the spine, or `None`.
fn scaled_pipe(
    spine: &Wire,
    profile: &Wire,
    law: &Handle<dyn LawFunction>,
) -> Result<Option<BRepOffsetAPIMakePipeShell>, Failure> {
    let mut a_mk_pipe = BRepOffsetAPIMakePipeShell::new(spine)?;
    a_mk_pipe.set_tolerance(1.0e-8, 1.0e-8, 1.0e-6);
    a_mk_pipe.set_law(profile.as_ref(), law, false, false)?;
    a_mk_pipe.build(&ProgressRange::default())?;
    Ok(a_mk_pipe.is_done().then_some(a_mk_pipe))
}

/// An annular face on the plane between the outer and the reversed inner wire.
fn ring_face(plane: Plane3, outer: &Shape, inner: &Shape) -> Result<Option<Shape>, Failure> {
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::from_pln(plane)).cast();
    let mut a_mk_face = BRepBuilderAPIMakeFace::new();
    a_mk_face.init_surface(&surface, false, precision::confusion())?;
    a_mk_face.add(&topods::wire(outer)?)?;
    a_mk_face.add(&topods::wire(&inner.reversed())?)?;
    if !a_mk_face.is_done() {
        return Ok(None);
    }
    Ok(Some(a_mk_face.face()?.into_shape()))
}

/// The first face of a shape, if any.
fn first_face(shape: &Shape) -> Result<Option<Shape>, Failure> {
    let exp = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)?;
    Ok(exp.more().then(|| exp.current().clone()))
}

/// `buildBentTube(...)`: the tube wall solid and the inner gas solid, or `None` when a
/// construction step fails.
fn build_bent_tube(
    the_wall_thickness: f64,
    the_dia1: f64,
    the_dia2: f64,
    the_major_radius: f64,
    the_length: f64,
) -> Result<Option<(Solid, Shape)>, Failure> {
    let a_bend_angle = the_length / the_major_radius;
    if a_bend_angle >= std::f64::consts::PI {
        return Ok(None);
    }
    let a_radius_l = the_dia1 / 2.0;
    let a_radius_r = the_dia2 / 2.0;
    let an_origin = Axis2::new(Point3::new(5000.0, -300.0, 1000.0), Direction3::new(0.0, -1.0, -1.0)?)?;
    let a_circ1_plane = Plane3::new(an_origin.location(), an_origin.direction())?;
    let a_face_circle = Circle3::new(an_origin, a_radius_l)?;
    let an_out_face_circle = Circle3::new(an_origin, a_radius_l + the_wall_thickness)?;
    let a_circ_center = an_origin
        .location()
        .translated(Vector3::from(an_origin.x_direction().xyz()) * the_major_radius);
    let a_circ_axis = Axis1::new(a_circ_center, an_origin.y_direction());
    let a_circ2_plane = a_circ1_plane.rotated(a_circ_axis, a_bend_angle)?;
    let a_spine_axis = Axis2::with_x_direction(a_circ_center, an_origin.y_direction(), -an_origin.x_direction())?;
    let a_spine_circle = Circle3::new(a_spine_axis, the_major_radius)?;
    let a_wire1 = circle_wire(a_face_circle)?;
    let an_out_wire = circle_wire(an_out_face_circle)?;
    let a_spine_curve: Handle<dyn GeomCurve> =
        MakeArcOfCircle::from_parameters(a_spine_circle, 0.0, a_bend_angle, true)?.value()?.cast();
    let a_law1 = linear_law(&a_spine_curve, a_radius_r / a_radius_l)?;
    let a_law2 = linear_law(
        &a_spine_curve,
        (a_radius_r + the_wall_thickness) / (a_radius_l + the_wall_thickness),
    )?;
    let a_mk_edge = BRepBuilderAPIMakeEdge::from_curve(&a_spine_curve)?;
    if !a_mk_edge.is_done() {
        return Ok(None);
    }
    let a_spine_wire = BRepBuilderAPIMakeWire::from_edge(&a_mk_edge.edge()?)?.wire()?;
    let Some(mut a_mk_pipe1) = scaled_pipe(&a_spine_wire, &a_wire1, &a_law1)? else {
        return Ok(None);
    };
    let Some(mut a_mk_pipe2) = scaled_pipe(&a_spine_wire, &an_out_wire, &a_law2)? else {
        return Ok(None);
    };
    // Build opening face at start of bend (annular ring)
    let Some(a_face1) = ring_face(a_circ1_plane, &a_mk_pipe2.first_shape()?, &a_mk_pipe1.first_shape()?)? else {
        return Ok(None);
    };
    // Build opening face at end of bend (annular ring)
    let Some(a_face2) = ring_face(a_circ2_plane, &a_mk_pipe2.last_shape()?, &a_mk_pipe1.last_shape()?)? else {
        return Ok(None);
    };
    // Assemble tube shell from inner pipe face, outer pipe face, and two annular ring faces
    let mut a_tube_shell = Shell::default();
    let a_builder = BRepBuilder::default();
    a_builder.make_shell(&mut a_tube_shell);
    if let Some(face) = first_face(a_mk_pipe1.shape()?)? {
        a_builder.add(&mut a_tube_shell, &face.reversed())?;
    }
    // Extract the gas solid before modifying mkPipe1 to a solid
    a_mk_pipe1.make_solid()?;
    let the_gas_solid = a_mk_pipe1.shape()?.clone();
    if let Some(face) = first_face(a_mk_pipe2.shape()?)? {
        a_builder.add(&mut a_tube_shell, &face)?;
    }
    a_builder.add(&mut a_tube_shell, &a_face1.reversed())?;
    a_builder.add(&mut a_tube_shell, &a_face2)?;
    a_tube_shell.set_closed(BRepTool::is_closed(a_tube_shell.as_ref())?)?;
    let mut the_wall_solid = Solid::default();
    a_builder.make_solid(&mut the_wall_solid);
    a_builder.add(&mut the_wall_solid, a_tube_shell.as_ref())?;
    Ok(Some((the_wall_solid, the_gas_solid)))
}

/// The number of lengths within the relative tolerance of the expected one.
fn count_matching(lengths: &[f64], the_expected: f64, a_tol: f64) -> usize {
    lengths
        .iter()
        .filter(|a_len| (*a_len - the_expected).abs() / the_expected <= a_tol)
        .count()
}

// TEST(BRepOffsetAPI_MakePipeShellTest, Bug332_BentTubeWithScalingLaw)
#[test]
fn bug332_bent_tube_with_scaling_law() -> Result<(), Failure> {
    let a_wall_thickness = 10.0;
    let a_dia1 = 30.0;
    let a_dia2 = 50.0;
    let a_major_radius = 100.0;
    let a_length = 200.0;
    let built = build_bent_tube(a_wall_thickness, a_dia1, a_dia2, a_major_radius, a_length)?;
    let (a_wall_solid, a_gas_solid) = built.expect("buildBentTube construction failed");
    // Validate both solids
    assert!(BRepCheckAnalyzer::new(a_wall_solid.as_ref(), true, false, false)?.is_valid()?, "wallSolid is not valid");
    assert!(BRepCheckAnalyzer::new(&a_gas_solid, true, false, false)?.is_valid()?, "gasSolid is not valid");
    // Collect all wire lengths in wallSolid
    let mut a_wire_lengths = Vec::new();
    let mut an_exp = Explorer::new(a_wall_solid.as_ref(), ShapeEnum::Wire, ShapeEnum::Shape)?;
    while an_exp.more() {
        let a_len = wire_length(&topods::wire(an_exp.current())?)?;
        if a_len > precision::confusion() {
            a_wire_lengths.push(a_len);
        }
        an_exp.next()?;
    }
    let a_pi = std::f64::consts::PI;
    let a_exp_len1 = a_pi * a_dia1;
    let a_exp_len2 = a_pi * (a_dia1 + 2.0 * a_wall_thickness);
    let a_exp_len4 = a_pi * (a_dia2 + 2.0 * a_wall_thickness);
    let a_tol = 0.001;
    assert!(count_matching(&a_wire_lengths, a_exp_len1, a_tol) >= 1, "No wire found with circumference pi*dia1 = {a_exp_len1}");
    // aExpLen2 and aExpLen3 are identical (dia2/2 == dia1/2 + wall), so expect at least 2 matches
    assert!(count_matching(&a_wire_lengths, a_exp_len2, a_tol) >= 2, "Expected 2 wires with circumference pi*50 = {a_exp_len2}");
    assert!(count_matching(&a_wire_lengths, a_exp_len4, a_tol) >= 1, "No wire found with circumference pi*(dia2+2*wall) = {a_exp_len4}");
    Ok(())
}

/// The base circle wire moved by a transformation (not copied).
fn transformed_wire(wire: &Wire, trsf: Transform3) -> Result<Wire, Failure> {
    let mut transform = BRepBuilderAPITransform::with_shape(wire.as_ref(), trsf, false, false)?;
    topods::wire(transform.shape()?)
}

// TEST(BRepOffsetAPI_MakePipeShellTest, Bug24909_2_ClosedCircularSpine)
// The source DRAW test is explicitly incomplete: this configuration may fail with NotDone
// or be accepted by the sweep implementation. The native test skips on both failures
// (GTEST_SKIP); the translation returns there, as the skip does. At the pinned revision
// both kernels throw Standard_OutOfRange from Build (the first skip).
#[test]
fn bug24909_2_closed_circular_spine() -> Result<(), Failure> {
    let a_circle = Circle3::new(Axis2::new(Point3::new(0.0, 0.0, 0.0), Direction3::new(0.0, 0.0, 1.0)?)?, 40.0)?;
    let a_base_wire = circle_wire(a_circle)?;
    let mut a_rotate = Transform3::default();
    a_rotate.set_rotation(
        Axis1::new(Point3::new(0.0, 0.0, 0.0), Direction3::new(0.0, 1.0, 0.0)?),
        std::f64::consts::PI / 2.0,
    )?;
    let a_rotated_wire = transformed_wire(&a_base_wire, a_rotate)?;
    let mut a_translate_up = Transform3::default();
    a_translate_up.set_translation(Vector3::new(0.0, 0.0, 75.0));
    let a_sketch = transformed_wire(&a_rotated_wire, a_translate_up)?;
    let mut a_translate_down = Transform3::default();
    a_translate_down.set_translation(Vector3::new(0.0, 0.0, -75.0));
    let a_sketch001 = transformed_wire(&a_rotated_wire, a_translate_down)?;
    let mut a_translate_spine = Transform3::default();
    a_translate_spine.set_translation(Vector3::new(50.0, 0.0, 0.0));
    let a_sketch002 = transformed_wire(&a_base_wire, a_translate_spine)?;
    let mut a_sweep = BRepOffsetAPIMakePipeShell::new(&a_sketch002)?;
    a_sweep.set_mode_frenet(false)?; // setsweep -CF
    a_sweep.add(a_sketch001.as_ref(), false, false)?;
    a_sweep.add(a_sketch.as_ref(), false, false)?;
    let built = a_sweep.build(&ProgressRange::default());
    if built.is_err() {
        // GTEST_SKIP: TODO OCC24909: the historical closed-spine sweep still reports NotDone.
        return Ok(());
    }
    if !a_sweep.is_done() {
        // GTEST_SKIP: TODO OCC24909: the closed-spine sweep did not complete on this platform.
        return Ok(());
    }
    assert_eq!(a_sweep.get_status(), BRepBuilderAPIPipeError::PipeDone);
    assert!(!a_sweep.shape()?.is_null());
    Ok(())
}
