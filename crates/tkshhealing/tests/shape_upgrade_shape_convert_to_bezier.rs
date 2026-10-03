//! Native differential tests for ShapeUpgrade_ShapeConvertToBezier.
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topexp::Explorer,
    topods::{Face, Shape},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dBezierCurve, Geom2dCurve, arrays};
use rustycad_tkg3d::{
    geom::{GeomBezierCurve, GeomBezierSurface},
    top_abs::ShapeEnum,
};
use rustycad_tkmath::gp::{Axis3, Cylinder3, Plane3};
use rustycad_tkshhealing::{
    shape_analysis::ShapeAnalysisEdge,
    shape_upgrade::{ShapeDivider, ShapeUpgradeShapeConvertToBezier},
};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeFace;

#[test]
fn all_modes_match_native_defaults_and_setters() {
    let converter = ShapeUpgradeShapeConvertToBezier::empty();
    assert_eq!(
        mode_values(&converter),
        [
            false, false, false, true, true, true, true, true, true, true
        ]
    );
    assert_eq!(mode_row("defaults", &converter), expected_line(0));
    configure_all_modes(&converter);
    assert_eq!(mode_row("configured", &converter), expected_line(1));
}

#[test]
fn plane_conversion_cases_match_native_topology_and_geometry()
-> Result<(), rustycad_tkernel::standard::Failure> {
    for (index, name) in ["unchanged", "surface_only", "curves3d", "curves2d", "all"]
        .iter()
        .enumerate()
    {
        assert_eq!(conversion_row(name, index)?, expected_line(index + 2));
    }
    Ok(())
}

#[test]
fn cylinder_conversion_preserves_native_circle_policy_and_topology()
-> Result<(), rustycad_tkernel::standard::Failure> {
    let cylinder = cylinder_face()?.into_shape();
    let all_curves = convert_3d(&cylinder, true);
    let pcurves = convert_2d(&cylinder);
    let lines_only = convert_3d(&cylinder, false);
    assert_eq!(
        conversion_summary("cylinder_curves", &all_curves)?,
        expected_line(7)
    );
    assert_eq!(
        conversion_summary("cylinder_pcurves", &pcurves)?,
        expected_line(8)
    );
    assert_eq!(
        conversion_summary("cylinder_line_only", &lines_only)?,
        expected_line(9)
    );
    Ok(())
}

#[test]
fn non_unit_mixed_seam_ranges_match_native_postprocess()
-> Result<(), rustycad_tkernel::standard::Failure> {
    let face = mixed_range_seam_face()?;
    let converter = ShapeUpgradeShapeConvertToBezier::new(face.into_shape());
    converter.set_surface_conversion(true);
    let changed = converter.perform(true)?;
    let result_face = first_face(&converter.result())?;
    let seam = first_seam_edge(&result_face)?;
    assert_eq!(
        mixed_seam_row(changed, &seam, &result_face)?,
        expected_line(11)
    );
    Ok(())
}

#[test]
fn trait_dispatch_runs_the_overridden_inherent_conversion()
-> Result<(), rustycad_tkernel::standard::Failure> {
    let converter = ShapeUpgradeShapeConvertToBezier::new(plane_face()?.into_shape());
    converter.set_surface_conversion(true);
    let policy: &dyn ShapeDivider = &converter;
    assert!(policy.perform(true)?);
    assert_eq!(count_faces(&converter.result()), 1);
    assert!(has_bezier_surface(&converter.result())?);
    Ok(())
}

fn conversion_row(name: &str, mode: usize) -> Result<String, rustycad_tkernel::standard::Failure> {
    let source = plane_face()?.into_shape();
    let converter = ShapeUpgradeShapeConvertToBezier::new(source);
    configure_conversion(&converter, mode);
    let changed = converter.perform(true)?;
    let result = converter.result();
    conversion_summary_with_changed(name, &result, changed)
}

fn conversion_summary(
    name: &str,
    converter: &ShapeUpgradeShapeConvertToBezier,
) -> Result<String, rustycad_tkernel::standard::Failure> {
    let changed = converter.perform(true)?;
    let result = converter.result();
    conversion_summary_with_changed(name, &result, changed)
}

fn conversion_summary_with_changed(
    name: &str,
    result: &Shape,
    changed: bool,
) -> Result<String, rustycad_tkernel::standard::Failure> {
    Ok(format!(
        "{name} changed={} faces={} edges={} curves3d={} curves2d={} bezierSurfaces={}",
        usize::from(changed),
        count_faces(result),
        count(result, ShapeEnum::Edge),
        count_beziers_3d(result)?,
        count_beziers_2d(result)?,
        usize::from(has_bezier_surface(result)?),
    ))
}

fn convert_3d(shape: &Shape, convert_circles: bool) -> ShapeUpgradeShapeConvertToBezier {
    let converter = ShapeUpgradeShapeConvertToBezier::new(shape.clone());
    converter.set_3d_conversion(true);
    converter.set_3d_circle_conversion(convert_circles);
    converter
}

fn convert_2d(shape: &Shape) -> ShapeUpgradeShapeConvertToBezier {
    let converter = ShapeUpgradeShapeConvertToBezier::new(shape.clone());
    converter.set_2d_conversion(true);
    converter
}

fn configure_conversion(converter: &ShapeUpgradeShapeConvertToBezier, mode: usize) {
    converter.set_surface_conversion(mode == 1 || mode == 4);
    converter.set_3d_conversion(mode == 2 || mode == 4);
    converter.set_2d_conversion(mode == 3 || mode == 4);
}

fn configure_all_modes(converter: &ShapeUpgradeShapeConvertToBezier) {
    converter.set_2d_conversion(true);
    converter.set_3d_conversion(true);
    converter.set_surface_conversion(true);
    converter.set_3d_line_conversion(false);
    converter.set_3d_circle_conversion(false);
    converter.set_3d_conic_conversion(false);
    converter.set_plane_mode(false);
    converter.set_revolution_mode(false);
    converter.set_extrusion_mode(false);
    converter.set_bspline_mode(false);
}

fn mode_row(name: &str, converter: &ShapeUpgradeShapeConvertToBezier) -> String {
    let values = mode_values(converter);
    format!(
        "{name} {}",
        values
            .map(u8::from)
            .map(|value| value.to_string())
            .join(" ")
    )
}

fn mode_values(converter: &ShapeUpgradeShapeConvertToBezier) -> [bool; 10] {
    [
        converter.get_2d_conversion(),
        converter.get_3d_conversion(),
        converter.get_surface_conversion(),
        converter.get_3d_line_conversion(),
        converter.get_3d_circle_conversion(),
        converter.get_3d_conic_conversion(),
        converter.get_plane_mode(),
        converter.get_revolution_mode(),
        converter.get_extrusion_mode(),
        converter.get_bspline_mode(),
    ]
}

fn expected_line(index: usize) -> &'static str {
    include_str!("../../../tools/oracle/shape_upgrade_shape_convert_to_bezier.expected")
        .lines()
        .nth(index)
        .unwrap()
}

fn plane_face() -> Result<Face, rustycad_tkernel::standard::Failure> {
    BRepBuilderAPIMakeFace::from_plane_bounds(Plane3::default(), (0., 1., 0., 1.))?.face()
}

fn cylinder_face() -> Result<Face, rustycad_tkernel::standard::Failure> {
    let surface = Cylinder3::new(Axis3::default(), 2.)?;
    BRepBuilderAPIMakeFace::from_cylinder_bounds(surface, (0., std::f64::consts::TAU, 0., 1.))?
        .face()
}

fn mixed_range_seam_face() -> Result<Face, rustycad_tkernel::standard::Failure> {
    let face = BRepBuilderAPIMakeFace::from_cylinder_bounds(
        Cylinder3::new(Axis3::default(), 2.)?,
        (0., std::f64::consts::TAU, 0., 5.),
    )?
    .face()?;
    let seam = first_seam_edge(&face)?;
    install_mixed_seam_curves(&face, &seam)?;
    Ok(face)
}

fn install_mixed_seam_curves(
    face: &Face,
    edge: &rustycad_tkbrep::topods::Edge,
) -> Result<(), rustycad_tkernel::standard::Failure> {
    let analysis = ShapeAnalysisEdge::new();
    let (current, first, last) = analysis.pcurve_on_face(edge, face, false)?;
    let reverse_shape = edge.reversed();
    let reverse_edge = rustycad_tkbrep::topods::edge(&reverse_shape)?;
    let (reverse, _, _) = analysis.pcurve_on_face(&reverse_edge, face, false)?;
    let poles = bezier_poles(&current, first, last)?;
    let bezier = Handle::new(Geom2dBezierCurve::new(&poles)?).cast();
    let builder = BRepBuilder::default();
    builder.update_edge_pcurves_on_face(edge, (&bezier, &reverse), face, 0.)?;
    builder.range_on_face(edge, face, 0.2, 0.8)
}

fn bezier_poles(
    curve: &Handle<dyn Geom2dCurve>,
    first: f64,
    last: f64,
) -> Result<
    rustycad_tkernel::ncollection::Array1<'static, rustycad_tkmath::gp::Point2>,
    rustycad_tkernel::standard::Failure,
> {
    let mut poles = arrays::one_based(2)?;
    arrays::put(&mut poles, 1, curve.value(first)?)?;
    arrays::put(&mut poles, 2, curve.value(last)?)?;
    Ok(poles)
}

fn first_face(shape: &Shape) -> Result<Face, rustycad_tkernel::standard::Failure> {
    let faces = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)?;
    rustycad_tkbrep::topods::face(faces.current())
}

fn first_seam_edge(
    face: &Face,
) -> Result<rustycad_tkbrep::topods::Edge, rustycad_tkernel::standard::Failure> {
    let edges = Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)?;
    for found in edges {
        let mut edge = rustycad_tkbrep::topods::edge(&found)?;
        edge.set_orientation(rustycad_tkg3d::top_abs::Orientation::Forward);
        if BRepTool::is_closed_on_face(&edge, face)? {
            return Ok(edge);
        }
    }
    Err(rustycad_tkernel::standard::Failure::with_type(
        "Standard_NoSuchObject",
        "mixed seam fixture has no closed edge; expected a cylindrical periodic seam",
        "",
    ))
}

fn mixed_seam_row(
    changed: bool,
    edge: &rustycad_tkbrep::topods::Edge,
    face: &Face,
) -> Result<String, rustycad_tkernel::standard::Failure> {
    let analysis = ShapeAnalysisEdge::new();
    let (current, first, last) = analysis.pcurve_on_face(edge, face, false)?;
    let reverse_shape = edge.reversed();
    let reverse = rustycad_tkbrep::topods::edge(&reverse_shape)?;
    let (opposite, first2, last2) = analysis.pcurve_on_face(&reverse, face, false)?;
    format_mixed_seam(changed, &current, (first, last), &opposite, (first2, last2))
}

fn format_mixed_seam(
    changed: bool,
    current: &Handle<dyn Geom2dCurve>,
    current_range: (f64, f64),
    opposite: &Handle<dyn Geom2dCurve>,
    opposite_range: (f64, f64),
) -> Result<String, rustycad_tkernel::standard::Failure> {
    let a = current.value(current_range.0)?;
    let b = current.value(current_range.1)?;
    let c = opposite.value(opposite_range.0)?;
    let d = opposite.value(opposite_range.1)?;
    Ok(format!(
        "mixed_range_seam changed={} currentBezier={} reverseBezier={} range={:.5},{:.5} reverseRange={:.5},{:.5} {}",
        usize::from(changed),
        usize::from(current.cast::<Geom2dBezierCurve>().is_some()),
        usize::from(opposite.cast::<Geom2dBezierCurve>().is_some()),
        current_range.0,
        current_range.1,
        opposite_range.0,
        opposite_range.1,
        endpoint_text(a, b, c, d),
    ))
}

fn endpoint_text(
    current_start: rustycad_tkmath::gp::Point2,
    current_end: rustycad_tkmath::gp::Point2,
    reverse_start: rustycad_tkmath::gp::Point2,
    reverse_end: rustycad_tkmath::gp::Point2,
) -> String {
    format!(
        "currentEnds={:.5},{:.5};{:.5},{:.5} reverseEnds={:.5},{:.5};{:.5},{:.5}",
        current_start.x(),
        current_start.y(),
        current_end.x(),
        current_end.y(),
        reverse_start.x(),
        reverse_start.y(),
        reverse_end.x(),
        reverse_end.y(),
    )
}

fn count_faces(shape: &Shape) -> usize {
    if shape.is_null() {
        0
    } else {
        count(shape, ShapeEnum::Face)
    }
}

fn count(shape: &Shape, kind: ShapeEnum) -> usize {
    if shape.is_null() {
        return 0;
    }
    Explorer::new(shape, kind, ShapeEnum::Shape)
        .unwrap()
        .count()
}

fn count_beziers_3d(shape: &Shape) -> Result<usize, rustycad_tkernel::standard::Failure> {
    if shape.is_null() {
        return Ok(0);
    }
    let mut count = 0;
    let edges = Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)?;
    for found in edges {
        let edge = rustycad_tkbrep::topods::edge(&found)?;
        if BRepTool::curve(&edge)?
            .0
            .cast::<GeomBezierCurve>()
            .is_some()
        {
            count += 1;
        }
    }
    Ok(count)
}

fn count_beziers_2d(shape: &Shape) -> Result<usize, rustycad_tkernel::standard::Failure> {
    if shape.is_null() {
        return Ok(0);
    }
    let mut count = 0;
    let faces = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)?;
    let face = rustycad_tkbrep::topods::face(faces.current())?;
    let edges = Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)?;
    for found in edges {
        let edge = rustycad_tkbrep::topods::edge(&found)?;
        let curve = ShapeAnalysisEdge::new()
            .pcurve_on_face(&edge, &face, false)?
            .0;
        if curve.cast::<Geom2dBezierCurve>().is_some() {
            count += 1;
        }
    }
    Ok(count)
}

fn has_bezier_surface(shape: &Shape) -> Result<bool, rustycad_tkernel::standard::Failure> {
    if shape.is_null() {
        return Ok(false);
    }
    let faces = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)?;
    if !faces.more() {
        return Ok(false);
    }
    let face = rustycad_tkbrep::topods::face(faces.current())?;
    let (surface, _) = BRepTool::surface_with_location(&face)?;
    Ok(!surface.cast::<GeomBezierSurface>().is_null())
}
