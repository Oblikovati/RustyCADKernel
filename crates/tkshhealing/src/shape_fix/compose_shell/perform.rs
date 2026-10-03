//! Full ComposeShell orchestration from native `Perform`.
use super::dispatch_wires::{PatchLoopGroup, dispatch_wires};
use super::{ShapeFixComposeShell, ShapeFixWireSegment};
use crate::shape_build::ShapeBuildReShape;
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use crate::shape_fix::{FixRoot, face::make_faces_on_patch::make_faces_on_patch};
use rustycad_tkbrep::topods::{Builder, Face, Shape, Shell};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::Failure;

pub(super) fn perform(composer: &ShapeFixComposeShell) -> Result<bool, Failure> {
    initialize_perform(composer);
    let mut segments = composer.load_wires()?;
    if segments.is_empty() {
        mark_status(composer, ShapeExtendStatus::Fail6);
        return Ok(false);
    }
    split_and_collect(composer, &mut segments)?;
    let faces = compose_patch_faces(composer, &mut segments)?;
    store_result(composer, faces)?;
    mark_status(composer, ShapeExtendStatus::Done1);
    Ok(true)
}

fn initialize_perform(composer: &ShapeFixComposeShell) {
    let mut fields = composer.fields.write();
    fields.status = ShapeExtend::encode_status(ShapeExtendStatus::Ok);
    fields.invert_edge_status = false;
}

fn split_and_collect(
    composer: &ShapeFixComposeShell,
    segments: &mut Sequence<ShapeFixWireSegment>,
) -> Result<(), Failure> {
    super::split_by_grid::split_by_grid(composer, segments)?;
    composer.break_wires(segments)?;
    let mut loops = Sequence::default();
    composer.collect_wires(&mut loops, segments)?;
    *segments = loops;
    Ok(())
}

fn compose_patch_faces(
    composer: &ShapeFixComposeShell,
    segments: &mut Sequence<ShapeFixWireSegment>,
) -> Result<Vec<Face>, Failure> {
    let groups = dispatch_wires(composer, segments)?;
    let (_, original, location, _) = composer.collection_inputs();
    let context = composer.context();
    let invert_edge_status = composer.fields.read().invert_edge_status;
    assemble_patch_groups(groups, &original, &location, &context, invert_edge_status)
}

fn assemble_patch_groups(
    groups: Vec<PatchLoopGroup>,
    original_face: &Face,
    location: &rustycad_tkmath::toploc::Location,
    context: &rustycad_tkernel::standard::Handle<ShapeBuildReShape>,
    invert_edge_status: bool,
) -> Result<Vec<Face>, Failure> {
    let mut faces = Vec::new();
    for group in groups {
        faces.extend(assemble_patch_group(
            group,
            original_face,
            location,
            context,
            invert_edge_status,
        )?);
    }
    Ok(faces)
}

fn assemble_patch_group(
    mut group: PatchLoopGroup,
    original_face: &Face,
    location: &rustycad_tkmath::toploc::Location,
    context: &rustycad_tkernel::standard::Handle<ShapeBuildReShape>,
    invert_edge_status: bool,
) -> Result<Vec<Face>, Failure> {
    let mut loops = Sequence::default();
    for shape in group.loops.drain(..) {
        loops.append(shape);
    }
    make_faces_on_patch(
        &group.surface,
        &mut loops,
        original_face,
        location,
        context,
        invert_edge_status,
    )
}

fn store_result(composer: &ShapeFixComposeShell, faces: Vec<Face>) -> Result<(), Failure> {
    let mut result = if faces.len() == 1 {
        faces[0].clone().into_shape()
    } else {
        make_result_shell(faces)?
    };
    result.set_orientation(composer.fields.read().orientation);
    composer.fields.write().result = result;
    Ok(())
}

fn make_result_shell(faces: Vec<Face>) -> Result<Shape, Failure> {
    let mut shell = Shell::default();
    Builder.make_shell(&mut shell);
    for face in faces {
        Builder.add(&mut shell, &face.clone().into_shape())?;
    }
    Ok(shell.into_shape())
}

fn mark_status(composer: &ShapeFixComposeShell, status: ShapeExtendStatus) {
    composer.fields.write().status |= ShapeExtend::encode_status(status);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shape_build::ShapeBuildReShape;
    use crate::shape_extend::{ShapeExtendCompositeSurface, ShapeExtendParametrisation};
    use crate::shape_fix::FixRoot;
    use rustycad_tkernel::ncollection::HArray2;
    use rustycad_tkernel::standard::Handle;
    use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
    use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
    use rustycad_tkmath::gp::{Axis3, Cylinder3, Plane3, Point3};
    use rustycad_tkmath::toploc::Location;
    use rustycad_tktopalgo::brep_builder_api::{
        BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeWire,
    };

    #[test]
    fn perform_planar_face_matches_native_output_and_done_status() {
        let composer = initialized_composer(&square_face());
        let done = composer.perform().unwrap();
        assert_native_result(&composer, done, "perform_plane_face");
        assert_eq!(composer.result().shape_type().unwrap(), ShapeEnum::Face);
        assert_eq!(composer.result().orientation(), Orientation::Forward);
    }

    #[test]
    fn perform_two_patches_matches_native_shell_and_done_status() {
        let composer = initialized_two_patch_composer();
        let done = composer.perform().unwrap();
        assert_native_result(&composer, done, "perform_two_patches");
    }

    #[test]
    fn perform_without_wire_segments_sets_fail6_and_keeps_null_result() {
        let composer = initialized_composer(&empty_plane_face());
        let done = composer.perform().unwrap();
        assert!(!done);
        assert!(composer.result().is_null());
        assert!(composer.status(ShapeExtendStatus::Fail6));
        assert!(!composer.status(ShapeExtendStatus::Done1));
        assert_no_wire_oracle_row(&composer);
    }

    #[test]
    fn perform_assigns_a_contained_loop_as_a_hole_like_native() {
        let composer = initialized_composer(&nested_face());
        composer.set_closed_mode(true);
        assert!(composer.perform().unwrap());
        let result = composer.result();
        assert_nested_face_matches_oracle(&result);
    }

    #[test]
    fn perform_periodic_cylinder_matches_native_open_and_closed_paths() {
        assert_periodic_cylinder_result(false, "perform_periodic_cylinder_open");
        assert_periodic_cylinder_result(true, "perform_periodic_cylinder_closed");
    }

    #[test]
    fn perform_periodic_cylinder_with_trimmed_patches_matches_native_pcurves() {
        let composer = initialized_trimmed_cylinder_composer();
        let done = composer.perform().unwrap();
        assert_native_result(&composer, done, "perform_periodic_cylinder_patches");
    }

    fn assert_periodic_cylinder_result(closed: bool, label: &str) {
        let composer = initialized_cylinder_composer(closed);
        let done = composer.perform().unwrap();
        assert_native_result(&composer, done, label);
    }

    fn initialized_cylinder_composer(closed: bool) -> ShapeFixComposeShell {
        let cylinder = Cylinder3::new(Axis3::default(), 2.0).unwrap();
        let face = BRepBuilderAPIMakeFace::from_cylinder_bounds(
            cylinder,
            (0.0, std::f64::consts::TAU, 0.0, 2.0),
        )
        .unwrap()
        .face()
        .unwrap();
        let surface: Handle<dyn GeomSurface> = Handle::new(
            rustycad_tkg3d::geom::GeomCylindricalSurface::from_cylinder(cylinder),
        )
        .cast();
        let composer = ShapeFixComposeShell::new();
        composer.set_context(&Handle::new(ShapeBuildReShape::new()));
        composer
            .init(
                &composite_grid(1, vec![surface]),
                &Location::default(),
                &face,
                1.0e-7,
            )
            .unwrap();
        composer.set_closed_mode(closed);
        composer
    }

    fn initialized_trimmed_cylinder_composer() -> ShapeFixComposeShell {
        let cylinder = Cylinder3::new(Axis3::default(), 2.0).unwrap();
        let face = BRepBuilderAPIMakeFace::from_cylinder_bounds(
            cylinder,
            (0.0, std::f64::consts::TAU, 0.0, 2.0),
        )
        .unwrap()
        .face()
        .unwrap();
        let basis: Handle<dyn GeomSurface> = Handle::new(
            rustycad_tkg3d::geom::GeomCylindricalSurface::from_cylinder(cylinder),
        )
        .cast();
        let first = trimmed_cylinder(&basis, (0.0, std::f64::consts::PI));
        let second = trimmed_cylinder(&basis, (std::f64::consts::PI, std::f64::consts::TAU));
        let composer = ShapeFixComposeShell::new();
        composer.set_context(&Handle::new(ShapeBuildReShape::new()));
        composer
            .init(
                &composite_grid(2, vec![first, second]),
                &Location::default(),
                &face,
                1.0e-7,
            )
            .unwrap();
        composer.set_closed_mode(true);
        composer
    }

    fn trimmed_cylinder(basis: &Handle<dyn GeomSurface>, u: (f64, f64)) -> Handle<dyn GeomSurface> {
        Handle::new(
            rustycad_tkg3d::geom::GeomRectangularTrimmedSurface::new(
                basis,
                u,
                (0.0, 2.0),
                (true, true),
            )
            .unwrap(),
        )
        .cast()
    }

    fn assert_nested_face_matches_oracle(result: &Shape) {
        let expected =
            include_str!("../../../../../tools/oracle/shape_fix_compose_shell_make_faces.expected");
        let wires = rustycad_tkbrep::topods::ShapeIterator::new(result, false, false)
            .filter(|shape| shape.shape_type().ok() == Some(ShapeEnum::Wire))
            .count();
        let row = format!("nested_patch changed=1 faces=1 wires={wires}");
        assert_eq!(row, expected.trim());
    }

    fn assert_no_wire_oracle_row(composer: &ShapeFixComposeShell) {
        let expected =
            include_str!("../../../../../tools/oracle/tkshhealing_compose_shell.expected");
        let row = format!(
            "perform_no_wires {} {} {} {}",
            0,
            i32::from(composer.result().is_null()),
            i32::from(composer.status(ShapeExtendStatus::Fail6)),
            i32::from(composer.status(ShapeExtendStatus::Done1)),
        );
        assert_eq!(
            row,
            expected
                .lines()
                .find(|line| line.starts_with("perform_no_wires "))
                .unwrap()
        );
    }

    fn assert_native_result(composer: &ShapeFixComposeShell, done: bool, label: &str) {
        let expected =
            include_str!("../../../../../tools/oracle/tkshhealing_compose_shell.expected");
        let result = composer.result();
        let mut row = format!(
            "{label} {} {} {} {} {}",
            i32::from(done),
            result.shape_type().unwrap() as i32,
            result.orientation() as i32,
            result.nb_children(),
            i32::from(composer.status(ShapeExtendStatus::Done1)),
        );
        if label == "perform_periodic_cylinder_open" || label == "perform_periodic_cylinder_closed"
        {
            let seam = periodic_seam_values(&result).unwrap();
            row.push_str(&format!(" {} {} {} {}", seam.0, seam.1, seam.2, seam.3));
        }
        if label == "perform_two_patches" || label == "perform_periodic_cylinder_patches" {
            let bounds = pcurve_bounds(&result).unwrap();
            row.push_str(&format!(
                " {} {} {} {}",
                bounds.0, bounds.1, bounds.2, bounds.3
            ));
        }
        assert_eq!(
            row,
            expected
                .lines()
                .find(|line| line.starts_with(label))
                .unwrap()
        );
    }

    fn periodic_seam_values(result: &Shape) -> Result<(i64, i64, i64, i64), Failure> {
        let face = rustycad_tkbrep::topods::face(result)?;
        for wire in rustycad_tkbrep::topods::ShapeIterator::new(result, false, false) {
            for child in rustycad_tkbrep::topods::ShapeIterator::new(&wire, false, false) {
                if child.shape_type()? != ShapeEnum::Edge {
                    continue;
                }
                let edge = rustycad_tkbrep::topods::edge(&child)?;
                if !rustycad_tkbrep::brep::BRepTool::is_closed_on_face(&edge, &face)? {
                    continue;
                }
                return seam_endpoint_values(&edge, &face);
            }
        }
        Err(Failure::with_message(
            "Standard_NoSuchObject",
            "periodic result had no seam edge; expected closed edge on face",
        ))
    }

    fn seam_endpoint_values(
        edge: &rustycad_tkbrep::topods::Edge,
        face: &Face,
    ) -> Result<(i64, i64, i64, i64), Failure> {
        let analyzer = crate::shape_analysis::ShapeAnalysisEdge::new();
        let (first, first_parameter, _) = analyzer.pcurve_on_face(edge, face, false)?;
        let reversed = rustycad_tkbrep::topods::edge(&edge.reversed())?;
        let (second, second_parameter, _) = analyzer.pcurve_on_face(&reversed, face, false)?;
        let first_point = first.value(first_parameter)?;
        let second_point = second.value(second_parameter)?;
        Ok((
            scaled(first_point.x()),
            scaled(first_point.y()),
            scaled(second_point.x()),
            scaled(second_point.y()),
        ))
    }

    fn scaled(value: f64) -> i64 {
        (value * 1.0e6).round() as i64
    }

    fn pcurve_bounds(result: &Shape) -> Result<(i64, i64, i64, i64), Failure> {
        let mut bounds = (
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
        );
        let analyzer = crate::shape_analysis::ShapeAnalysisEdge::new();
        for face in result_faces(result) {
            update_face_pcurve_bounds(&face, &analyzer, &mut bounds)?;
        }
        Ok((
            scaled(bounds.0),
            scaled(bounds.1),
            scaled(bounds.2),
            scaled(bounds.3),
        ))
    }

    fn result_faces(result: &Shape) -> Vec<Face> {
        if result.shape_type().ok() == Some(ShapeEnum::Face) {
            return rustycad_tkbrep::topods::face(result).into_iter().collect();
        }
        rustycad_tkbrep::topods::ShapeIterator::new(result, false, false)
            .filter_map(|shape| rustycad_tkbrep::topods::face(&shape).ok())
            .collect()
    }

    fn update_face_pcurve_bounds(
        face: &Face,
        analyzer: &crate::shape_analysis::ShapeAnalysisEdge,
        bounds: &mut (f64, f64, f64, f64),
    ) -> Result<(), Failure> {
        for wire in
            rustycad_tkbrep::topods::ShapeIterator::new(&face.clone().into_shape(), false, false)
        {
            update_wire_pcurve_bounds(&wire, face, analyzer, bounds)?;
        }
        Ok(())
    }

    fn update_wire_pcurve_bounds(
        wire: &Shape,
        face: &Face,
        analyzer: &crate::shape_analysis::ShapeAnalysisEdge,
        bounds: &mut (f64, f64, f64, f64),
    ) -> Result<(), Failure> {
        for edge_shape in rustycad_tkbrep::topods::ShapeIterator::new(wire, false, false) {
            update_edge_pcurve_bounds(&edge_shape, face, analyzer, bounds)?;
        }
        Ok(())
    }

    fn update_edge_pcurve_bounds(
        edge_shape: &Shape,
        face: &Face,
        analyzer: &crate::shape_analysis::ShapeAnalysisEdge,
        bounds: &mut (f64, f64, f64, f64),
    ) -> Result<(), Failure> {
        if edge_shape.shape_type()? != ShapeEnum::Edge {
            return Ok(());
        }
        let edge = rustycad_tkbrep::topods::edge(edge_shape)?;
        let (curve, first, last) = analyzer.pcurve_on_face(&edge, face, false)?;
        if curve.is_null() {
            return Ok(());
        }
        update_point_bound(&curve.value(first)?, bounds);
        update_point_bound(&curve.value(last)?, bounds);
        Ok(())
    }

    fn update_point_bound(point: &rustycad_tkmath::gp::Point2, bounds: &mut (f64, f64, f64, f64)) {
        bounds.0 = bounds.0.min(point.x());
        bounds.1 = bounds.1.max(point.x());
        bounds.2 = bounds.2.min(point.y());
        bounds.3 = bounds.3.max(point.y());
    }

    fn initialized_composer(face: &Face) -> ShapeFixComposeShell {
        let composer = ShapeFixComposeShell::new();
        composer.set_context(&Handle::new(ShapeBuildReShape::new()));
        composer
            .init(&one_patch_grid(), &Location::default(), face, 1.0e-7)
            .unwrap();
        composer
    }

    fn initialized_two_patch_composer() -> ShapeFixComposeShell {
        let composer = ShapeFixComposeShell::new();
        composer.set_context(&Handle::new(ShapeBuildReShape::new()));
        composer
            .init(
                &two_patch_grid(),
                &Location::default(),
                &square_face(),
                1.0e-7,
            )
            .unwrap();
        composer
    }

    fn one_patch_grid() -> Handle<ShapeExtendCompositeSurface> {
        let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
        composite_grid(1, vec![surface])
    }

    fn two_patch_grid() -> Handle<ShapeExtendCompositeSurface> {
        let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
        let left = trimmed_plane(&plane, (0.0, 1.0));
        let right = trimmed_plane(&plane, (1.0, 3.0));
        composite_grid(2, vec![left, right])
    }

    fn trimmed_plane(plane: &Handle<dyn GeomSurface>, u: (f64, f64)) -> Handle<dyn GeomSurface> {
        Handle::new(
            rustycad_tkg3d::geom::GeomRectangularTrimmedSurface::new(
                plane,
                u,
                (0.0, 2.0),
                (true, true),
            )
            .unwrap(),
        )
        .cast()
    }

    fn composite_grid(
        columns: i32,
        surfaces: Vec<Handle<dyn GeomSurface>>,
    ) -> Handle<ShapeExtendCompositeSurface> {
        let cells =
            HArray2::with_value(1, columns, 1, 1, Handle::<dyn GeomSurface>::default()).unwrap();
        for (index, surface) in surfaces.into_iter().enumerate() {
            cells
                .change_array2()
                .set_value(index as i32 + 1, 1, surface)
                .unwrap();
        }
        Handle::new(
            ShapeExtendCompositeSurface::with_grid(
                &Handle::new(cells),
                ShapeExtendParametrisation::Natural,
            )
            .unwrap(),
        )
    }

    fn square_face() -> Face {
        let wire = square_wire_bounds((0.0, 0.0), (2.0, 2.0));
        BRepBuilderAPIMakeFace::from_plane_wire(Plane3::default(), &wire, true)
            .unwrap()
            .face()
            .unwrap()
    }

    fn empty_plane_face() -> Face {
        BRepBuilderAPIMakeFace::from_plane(Plane3::default())
            .unwrap()
            .face()
            .unwrap()
    }

    fn nested_face() -> Face {
        let outer = square_wire_bounds((0.0, 0.0), (5.0, 5.0));
        let mut face = BRepBuilderAPIMakeFace::from_plane_wire(Plane3::default(), &outer, true)
            .unwrap()
            .face()
            .unwrap();
        let hole = square_wire_bounds((1.0, 1.0), (2.0, 2.0));
        Builder.add(&mut face, &hole.reversed()).unwrap();
        face
    }

    fn square_wire_bounds(
        (x_min, y_min): (f64, f64),
        (x_max, y_max): (f64, f64),
    ) -> rustycad_tkbrep::topods::Wire {
        let points = [
            Point3::new(x_min, y_min, 0.0),
            Point3::new(x_max, y_min, 0.0),
            Point3::new(x_max, y_max, 0.0),
            Point3::new(x_min, y_max, 0.0),
        ];
        let mut wire = BRepBuilderAPIMakeWire::new();
        for index in 0..points.len() {
            let edge = BRepBuilderAPIMakeEdge::from_points(
                points[index],
                points[(index + 1) % points.len()],
            )
            .unwrap()
            .edge()
            .unwrap();
            wire.add_edge(&edge).unwrap();
        }
        wire.wire().unwrap()
    }
}
