use super::*;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkg3d::geom::GeomRectangularTrimmedSurface;
use rustycad_tkg3d::top_abs::ShapeEnum;

#[test]
fn split_edges_two_patch_planar_face_matches_native_context_replacements() {
    let expected = include_str!("../../../../tools/oracle/tkshhealing_compose_shell.expected");
    assert_eq!(
        split_edges_summary(false),
        native_row(expected, "split_edges ")
    );
}

#[test]
fn split_edges_closed_mode_matches_native_context_replacements() {
    let expected = include_str!("../../../../tools/oracle/tkshhealing_compose_shell.expected");
    assert_eq!(
        split_edges_summary(true),
        native_row(expected, "split_edges_closed ")
    );
}

#[test]
fn split_edges_periodic_cylinder_matches_native_context_replacements() {
    let expected = include_str!("../../../../tools/oracle/tkshhealing_compose_shell.expected");
    let result = split_periodic_face_edges();
    assert_eq!(result, native_row(expected, "split_edges_periodic "));
}

fn split_periodic_face_edges() -> String {
    let cylinder = Cylinder3::new(Axis3::default(), 2.0).unwrap();
    let face = BRepBuilderAPIMakeFace::from_cylinder_bounds(
        cylinder,
        (0.0, std::f64::consts::TAU, 0.0, 2.0),
    )
    .unwrap()
    .face()
    .unwrap();
    let surface: Handle<dyn GeomSurface> =
        Handle::new(GeomCylindricalSurface::from_cylinder(cylinder)).cast();
    let grid = surface_grid(surface);
    let context = Handle::new(ShapeBuildReShape::new());
    let composer = initialized_composer(&face, &context);
    composer
        .init(&grid, &Location::default(), &face, 1.0e-7)
        .unwrap();
    let original = composer.load_wires().unwrap().value(1).unwrap().clone();
    composer.split_edges().unwrap();
    mapped_edge_summary("split_edges_periodic", &original, &context)
}

fn split_edges_summary(closed_mode: bool) -> String {
    let face = face_fixture();
    let context = Handle::new(ShapeBuildReShape::new());
    let composer = initialized_composer(&face, &context);
    let grid = two_trimmed_patch_grid();
    composer
        .init(&grid, &Location::default(), &face, 1.0e-7)
        .unwrap();
    composer.set_closed_mode(closed_mode);
    let original = composer.load_wires().unwrap().value(1).unwrap().clone();
    composer.split_edges().unwrap();
    let prefix = if closed_mode {
        "split_edges_closed"
    } else {
        "split_edges"
    };
    mapped_edge_summary(prefix, &original, &context)
}

fn mapped_edge_summary(
    prefix: &str,
    original: &ShapeFixWireSegment,
    context: &Handle<ShapeBuildReShape>,
) -> String {
    let mut result = String::from(prefix);
    for index in 1..=original.nb_edges() {
        let mapped = context
            .apply(
                &original.edge(index).unwrap().into_shape(),
                ShapeEnum::Shape,
            )
            .unwrap();
        append_mapped_edge(&mut result, &mapped);
    }
    result
}

fn append_mapped_edge(result: &mut String, mapped: &rustycad_tkbrep::topods::Shape) {
    let kind = mapped.shape_type().unwrap();
    result.push_str(&format!(" {}", kind as i32));
    if kind == ShapeEnum::Wire {
        let count = Explorer::new(mapped, ShapeEnum::Edge, ShapeEnum::Shape)
            .unwrap()
            .count();
        result.push_str(&format!(":{count}"));
    }
}

fn two_trimmed_patch_grid() -> Handle<ShapeExtendCompositeSurface> {
    let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    let left =
        GeomRectangularTrimmedSurface::new(&plane, (0.0, 1.0), (0.0, 2.0), (true, true)).unwrap();
    let right =
        GeomRectangularTrimmedSurface::new(&plane, (1.0, 3.0), (0.0, 2.0), (true, true)).unwrap();
    let patches = HArray2::with_value(1, 2, 1, 1, Handle::<dyn GeomSurface>::default()).unwrap();
    patches
        .change_array2()
        .set_value(1, 1, Handle::new(left).cast())
        .unwrap();
    patches
        .change_array2()
        .set_value(2, 1, Handle::new(right).cast())
        .unwrap();
    let grid = Handle::new(
        ShapeExtendCompositeSurface::with_grid(
            &Handle::new(patches),
            ShapeExtendParametrisation::Natural,
        )
        .unwrap(),
    );
    let joints = HArray1::with_value(1, 3, 0.0).unwrap();
    joints.change_array1().set_value(2, 1.0).unwrap();
    joints.change_array1().set_value(3, 3.0).unwrap();
    grid.get()
        .unwrap()
        .set_u_joint_values(&joints.array1())
        .unwrap();
    grid
}

#[test]
fn split_by_line_rectangular_loop_matches_native_intersections_and_codes() {
    let expected = include_str!("../../../../tools/oracle/tkshhealing_compose_shell.expected");
    let composer = initialized_composer(&face_fixture(), &Handle::new(ShapeBuildReShape::new()));
    let segments = composer.load_wires().unwrap();
    let mut wire = segments.value(1).unwrap().clone();
    let line = Line2::new(Point2::new(1.0, 0.0), Direction2::new(0.0, 1.0).unwrap());
    let mut parameters = Sequence::default();
    let mut codes = Sequence::default();
    let mut vertices = Sequence::default();
    let done = composer
        .split_by_line(
            &mut wire,
            line,
            true,
            1,
            &mut parameters,
            &mut codes,
            &mut vertices,
        )
        .unwrap();
    assert!(done);
    assert_eq!(
        split_by_line_summary(
            "split_by_line",
            &wire,
            &parameters,
            &codes,
            &vertices,
            false
        ),
        native_row(expected, "split_by_line ")
    );
}

#[test]
fn split_by_line_closed_mode_on_nonperiodic_face_matches_native() {
    let expected = include_str!("../../../../tools/oracle/tkshhealing_compose_shell.expected");
    let composer = initialized_composer(&face_fixture(), &Handle::new(ShapeBuildReShape::new()));
    composer.set_closed_mode(true);
    let segments = composer.load_wires().unwrap();
    let mut wire = segments.value(1).unwrap().clone();
    let mut parameters = Sequence::default();
    let mut codes = Sequence::default();
    let mut vertices = Sequence::default();
    let line = Line2::new(Point2::new(1.0, 0.0), Direction2::new(0.0, 1.0).unwrap());
    let done = composer
        .split_by_line(
            &mut wire,
            line,
            true,
            1,
            &mut parameters,
            &mut codes,
            &mut vertices,
        )
        .unwrap();
    assert!(done);
    assert_eq!(
        split_by_line_summary(
            "split_by_line",
            &wire,
            &parameters,
            &codes,
            &vertices,
            false
        ),
        native_row(expected, "split_by_line_closed ").replacen(
            "split_by_line_closed",
            "split_by_line",
            1
        )
    );
}

#[test]
fn split_by_line_periodic_cylinder_matches_native_seam_rows() {
    let expected = include_str!("../../../../tools/oracle/tkshhealing_compose_shell.expected");
    for (line_u, row) in [
        (0.0, "split_by_line_periodic "),
        (std::f64::consts::TAU, "split_by_line_periodic_shifted "),
    ] {
        assert_eq!(
            periodic_split_summary(line_u, row),
            native_row(expected, row)
        );
    }
}

fn periodic_split_summary(line_u: f64, row: &str) -> String {
    let (composer, mut wire) = periodic_cylinder_wire();
    let mut parameters = Sequence::default();
    let mut codes = Sequence::default();
    let mut vertices = Sequence::default();
    let line = Line2::new(Point2::new(line_u, 0.0), Direction2::new(0.0, 1.0).unwrap());
    let done = composer
        .split_by_line(
            &mut wire,
            line,
            true,
            1,
            &mut parameters,
            &mut codes,
            &mut vertices,
        )
        .unwrap();
    assert!(done, "native fixture {row} must split");
    split_by_line_summary(row.trim(), &wire, &parameters, &codes, &vertices, true)
}

fn periodic_cylinder_wire() -> (ShapeFixComposeShell, ShapeFixWireSegment) {
    let cylinder = Cylinder3::new(Axis3::default(), 2.0).unwrap();
    let bounds = (0.0, std::f64::consts::TAU, 0.0, 2.0);
    let face = BRepBuilderAPIMakeFace::from_cylinder_bounds(cylinder, bounds)
        .unwrap()
        .face()
        .unwrap();
    let surface: Handle<dyn GeomSurface> =
        Handle::new(GeomCylindricalSurface::from_cylinder(cylinder)).cast();
    let grid = surface_grid(surface);
    let composer = ShapeFixComposeShell::new();
    composer.set_context(&Handle::new(ShapeBuildReShape::new()));
    composer
        .init(&grid, &Location::default(), &face, 1.0e-7)
        .unwrap();
    composer.set_closed_mode(true);
    let wire = composer.load_wires().unwrap().value(1).unwrap().clone();
    (composer, wire)
}

#[test]
fn split_by_line_vertex_segment_matches_native_projection_and_replacement() {
    let expected = include_str!("../../../../tools/oracle/tkshhealing_compose_shell.expected");
    assert_eq!(
        split_vertex_segment_summary(),
        native_row(expected, "split_by_line_vertex ")
    );
}

fn split_vertex_segment_summary() -> String {
    let face = bounded_plane_face();
    let composer = initialized_composer(&face, &Handle::new(ShapeBuildReShape::new()));
    let mut segment = ShapeFixWireSegment::new();
    let vertex = BRepBuilderAPIMakeVertex::new(Point3::new(1.0, 1.0, 0.0))
        .unwrap()
        .vertex()
        .unwrap();
    segment.set_vertex(&vertex);
    let mut parameters = Sequence::default();
    let mut codes = Sequence::default();
    let mut vertices = Sequence::default();
    let line = Line2::new(Point2::new(1.0, 0.0), Direction2::new(0.0, 1.0).unwrap());
    let done = composer
        .split_by_line(
            &mut segment,
            line,
            true,
            1,
            &mut parameters,
            &mut codes,
            &mut vertices,
        )
        .unwrap();
    let point = BRepTool::pnt(&topods::vertex(vertices.value(1).unwrap()).unwrap()).unwrap();
    format!(
        "split_by_line_vertex {} {} {} {} {}:{}",
        done as u8,
        segment.is_vertex() as u8,
        parameters.length(),
        codes.value(1).unwrap(),
        point.x(),
        point.y()
    )
}

fn split_by_line_summary(
    prefix: &str,
    wire: &ShapeFixWireSegment,
    parameters: &Sequence<f64>,
    codes: &Sequence<i32>,
    vertices: &Sequence<rustycad_tkbrep::topods::Shape>,
    include_z: bool,
) -> String {
    let mut summary = format!(
        "{prefix} 1 {} {} {} {}",
        wire.nb_edges(),
        parameters.length(),
        codes.length(),
        vertices.length()
    );
    for index in 1..=parameters.length() {
        let vertex =
            rustycad_tkbrep::topods::vertex(vertices.value(index as usize).unwrap()).unwrap();
        let point = BRepTool::pnt(&vertex).unwrap();
        append_split_vertex(&mut summary, parameters, codes, index, point, include_z);
    }
    summary
}

fn append_split_vertex(
    summary: &mut String,
    parameters: &Sequence<f64>,
    codes: &Sequence<i32>,
    index: i32,
    point: Point3,
    include_z: bool,
) {
    if include_z {
        summary.push_str(&format!(
            " {}:{}:{}:{}:{}",
            parameters.value(index as usize).unwrap(),
            codes.value(index as usize).unwrap(),
            point.x(),
            point.y(),
            point.z()
        ));
    } else {
        summary.push_str(&format!(
            " {}:{}:{}:{}",
            parameters.value(index as usize).unwrap(),
            codes.value(index as usize).unwrap(),
            point.x(),
            point.y()
        ));
    }
}
