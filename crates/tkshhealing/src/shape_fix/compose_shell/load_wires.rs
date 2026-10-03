use super::ShapeFixComposeShell;
use crate::shape_analysis::{ShapeAnalysis, ShapeAnalysisEdge, ShapeAnalysisWireOrder};
use crate::shape_extend::ShapeExtendWireData;
use crate::shape_fix::{FixRoot, ShapeFixWire, ShapeFixWireSegment};
use rustycad_tkbrep::{
    brep_tools::ReShape,
    topods::{self, Builder, Face, Shape, ShapeIterator},
};
use rustycad_tkernel::{
    ncollection::Sequence,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::Xy;

pub(super) fn load_wires(
    composer: &ShapeFixComposeShell,
) -> Result<Sequence<ShapeFixWireSegment>, Failure> {
    let face = composer.fields.read().face.clone();
    let context_handle = composer.context();
    let context = context_handle.get().ok_or_else(null_context)?;
    let mut segments = Sequence::default();
    let face_shape = face.clone().into_shape();
    for child in ShapeIterator::new(&face_shape, false, true) {
        load_child(&mut segments, &face, context, &child)?;
    }
    Ok(segments)
}

fn load_child(
    segments: &mut Sequence<ShapeFixWireSegment>,
    face: &Face,
    context: &crate::shape_build::ShapeBuildReShape,
    child: &Shape,
) -> Result<(), Failure> {
    let applied = context.apply(child, ShapeEnum::Shape)?;
    if applied.is_null() {
        return Ok(());
    }
    match applied.shape_type()? {
        ShapeEnum::Wire => load_wire(segments, face, &topods::wire(&applied)?),
        ShapeEnum::Vertex => append_vertex(segments, &applied),
        _ => Ok(()),
    }
}

fn append_vertex(
    segments: &mut Sequence<ShapeFixWireSegment>,
    shape: &Shape,
) -> Result<(), Failure> {
    let mut segment = ShapeFixWireSegment::new();
    segment.set_vertex(&topods::vertex(shape)?);
    segment.set_orientation(shape.orientation());
    segments.append(segment);
    Ok(())
}

fn load_wire(
    segments: &mut Sequence<ShapeFixWireSegment>,
    face: &Face,
    wire: &topods::Wire,
) -> Result<(), Failure> {
    if is_nonmanifold_orientation(wire.orientation()) {
        append_nonmanifold_wire(segments, wire)?;
        return Ok(());
    }
    let (manifold, nonmanifold) = partition_wire(wire)?;
    if nonmanifold.nb_edges() > 0 {
        append_wire_data(segments, &nonmanifold, Orientation::Internal)?;
    }
    append_manifold_wire(segments, face, wire, &manifold)?;
    Ok(())
}

fn is_nonmanifold_orientation(orientation: Orientation) -> bool {
    !matches!(orientation, Orientation::Forward | Orientation::Reversed)
}

fn append_nonmanifold_wire(
    segments: &mut Sequence<ShapeFixWireSegment>,
    wire: &topods::Wire,
) -> Result<(), Failure> {
    let wire_data = Handle::new(ShapeExtendWireData::from_wire(wire, true, false)?);
    if wire_data.nb_edges() > 0 {
        append_wire_data(segments, &wire_data, Orientation::Internal)?;
    }
    Ok(())
}

fn partition_wire(
    wire: &topods::Wire,
) -> Result<(Handle<ShapeExtendWireData>, Handle<ShapeExtendWireData>), Failure> {
    let manifold = Handle::new(ShapeExtendWireData::new());
    let nonmanifold = Handle::new(ShapeExtendWireData::new());
    nonmanifold.set_manifold_mode(false);
    for child in ShapeIterator::new(&wire.clone().into_shape(), true, true) {
        let edge = topods::edge(&child)?;
        let target = if is_nonmanifold_orientation(edge.orientation()) {
            &nonmanifold
        } else {
            &manifold
        };
        target.add_edge(&edge, 0)?;
    }
    Ok((manifold, nonmanifold))
}

fn append_wire_data(
    segments: &mut Sequence<ShapeFixWireSegment>,
    wire_data: &Handle<ShapeExtendWireData>,
    orientation: Orientation,
) -> Result<(), Failure> {
    segments.append(ShapeFixWireSegment::from_wire_data(wire_data, orientation)?);
    Ok(())
}

fn append_manifold_wire(
    segments: &mut Sequence<ShapeFixWireSegment>,
    face: &Face,
    source_wire: &topods::Wire,
    wire_data: &Handle<ShapeExtendWireData>,
) -> Result<(), Failure> {
    if wire_data.nb_edges() == 0 {
        return Ok(());
    }
    let fixer = ShapeFixWire::new();
    fixer.load_wire_data(wire_data);
    let mut orient = 1;
    let surface = rustycad_tkbrep::brep::BRepTool::surface(face)?;
    if surface
        .get()
        .is_some_and(|surface| surface.is_u_periodic() && surface.is_v_periodic())
    {
        orient = reorder_periodic_wire(&fixer, face)?;
    }
    fixer.fix_reorder()?;
    if fixer.status_reorder(crate::shape_extend::ShapeExtendStatus::Done3) {
        orient = -1;
    }
    // OCCT compares the original wire, including its nonmanifold edges, with this manifold part.
    reverse_for_outer_bound(orient, face, source_wire, wire_data)?;
    append_wire_data(segments, wire_data, Orientation::Reversed)
}

fn reorder_periodic_wire(fixer: &ShapeFixWire, face: &Face) -> Result<i32, Failure> {
    let wire_data = fixer.wire_data()?;
    let mut order = ShapeAnalysisWireOrder::with_mode(false, 0.0, false);
    let edge_analyzer = ShapeAnalysisEdge::new();
    for index in 1..=wire_data.nb_edges() {
        let edge = wire_data.edge(index)?;
        let (curve, first, last) = edge_analyzer.pcurve_on_face(&edge, face, true)?;
        if let Some(curve) = curve.get() {
            order.add_2d(to_xy(curve.value(first)?), to_xy(curve.value(last)?));
        }
    }
    order.perform(false)?;
    let orientation = if order.status() < 0 { -1 } else { 1 };
    fixer.fix_reorder_with(&order)?;
    Ok(orientation)
}

fn to_xy(point: rustycad_tkmath::gp::Point2) -> Xy {
    Xy::new(point.x(), point.y())
}

fn reverse_for_outer_bound(
    orientation: i32,
    face: &Face,
    source_wire: &topods::Wire,
    wire_data: &Handle<ShapeExtendWireData>,
) -> Result<(), Failure> {
    if orientation >= 0 {
        return Ok(());
    }
    let before = outer_bound_for_source_wire(face, source_wire)?;
    let wire = wire_data.wire()?;
    let mut copied_face = topods::face(&face.clone().into_shape().empty_copied()?)?;
    Builder.add(&mut copied_face, &wire.clone().into_shape())?;
    let after = ShapeAnalysis::is_outer_bound(&copied_face)?;
    if before != after {
        wire_data.reverse_on_face(&copied_face)?;
    }
    Ok(())
}

fn outer_bound_for_source_wire(face: &Face, wire: &topods::Wire) -> Result<bool, Failure> {
    let mut copied_face = topods::face(&face.clone().into_shape().empty_copied()?)?;
    Builder.add(&mut copied_face, &wire.clone().into_shape())?;
    ShapeAnalysis::is_outer_bound(&copied_face)
}

fn null_context() -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        "ShapeFix_ComposeShell::LoadWires: null context",
    )
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
    use rustycad_tkmath::gp::{Axis3, Plane3, Point3};
    use rustycad_tkmath::toploc::Location;
    use rustycad_tktopalgo::brep_builder_api::{
        BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeWire,
    };

    #[test]
    fn manifold_boundary_load_matches_native_protected_helper() {
        let expected =
            include_str!("../../../../../tools/oracle/tkshhealing_compose_shell.expected");
        let composer = ShapeFixComposeShell::new();
        let context = Handle::new(ShapeBuildReShape::new());
        composer.set_context(&context);
        composer
            .init(
                &one_patch_grid(),
                &Location::default(),
                &square_face(),
                1.0e-7,
            )
            .unwrap();
        let segments = load_wires(&composer).unwrap();
        assert_eq!(
            native_segment_summary(&segments),
            expected
                .lines()
                .find(|row| row.starts_with("load_wires "))
                .unwrap()
        );
    }

    #[test]
    fn edge_segment_side_codes_match_native_protected_helper() {
        let expected =
            include_str!("../../../../../tools/oracle/tkshhealing_compose_shell.expected");
        let composer = ShapeFixComposeShell::new();
        let context = Handle::new(ShapeBuildReShape::new());
        composer.set_context(&context);
        composer
            .init(
                &one_patch_grid(),
                &Location::default(),
                &square_face(),
                1.0e-7,
            )
            .unwrap();
        let segments = load_wires(&composer).unwrap();
        let wire = segments.value(1).unwrap().wire_data().get().unwrap();
        let line = rustycad_tkmath::gp::Line2::new(
            rustycad_tkmath::gp::Point2::new(1.0, 0.0),
            rustycad_tkmath::gp::Direction2::new(0.0, 1.0).unwrap(),
        );
        assert_native_code(
            &composer,
            wire,
            line,
            (0.25, 0.75),
            expected
                .lines()
                .find(|row| row.starts_with("compute_code "))
                .unwrap(),
        );
        assert_native_code(
            &composer,
            wire,
            line,
            (1.25, 1.75),
            expected
                .lines()
                .find(|row| row.starts_with("compute_code_right "))
                .unwrap(),
        );
        let tangent = rustycad_tkmath::gp::Line2::new(
            rustycad_tkmath::gp::Point2::new(0.0, 0.0),
            rustycad_tkmath::gp::Direction2::new(1.0, 0.0).unwrap(),
        );
        assert_native_code(
            &composer,
            wire,
            tangent,
            (0.25, 0.75),
            expected
                .lines()
                .find(|row| row.starts_with("compute_code_tangent "))
                .unwrap(),
        );
        composer.set_closed_mode(true);
        assert_native_code(
            &composer,
            wire,
            line,
            (0.0, 2.0),
            expected
                .lines()
                .find(|row| row.starts_with("compute_code_closed_mode "))
                .unwrap(),
        );
    }

    #[test]
    fn mixed_wire_partitions_internal_edges_before_manifold_edges() {
        let expected =
            include_str!("../../../../../tools/oracle/tkshhealing_compose_shell.expected");
        let composer = ShapeFixComposeShell::new();
        let context = Handle::new(ShapeBuildReShape::new());
        composer.set_context(&context);
        composer
            .init(
                &one_patch_grid(),
                &Location::default(),
                &square_face_with_internal_edge(),
                1.0e-7,
            )
            .unwrap();
        let segments = load_wires(&composer).unwrap();
        assert_eq!(
            mixed_segment_summary(&segments),
            expected
                .lines()
                .find(|row| row.starts_with("load_wires_mixed "))
                .unwrap()
        );
    }

    fn assert_native_code(
        composer: &ShapeFixComposeShell,
        wire: &ShapeExtendWireData,
        line: rustycad_tkmath::gp::Line2,
        range: (f64, f64),
        expected: &str,
    ) {
        let code = composer
            .compute_code(wire, line, 1, 1, range.0, range.1, false)
            .unwrap();
        let label = expected.split_whitespace().next().unwrap();
        assert_eq!(format!("{label} {code}"), expected);
    }

    fn native_segment_summary(segments: &Sequence<ShapeFixWireSegment>) -> String {
        let segment = segments.value(1).unwrap();
        let orientation = if segment.orientation() == Orientation::Reversed {
            "reversed"
        } else {
            "other"
        };
        format!(
            "load_wires {} {} {orientation}",
            segments.length(),
            segment.nb_edges()
        )
    }

    fn mixed_segment_summary(segments: &Sequence<ShapeFixWireSegment>) -> String {
        let first = segments.value(1).unwrap();
        let last = segments.value(2).unwrap();
        format!(
            "load_wires_mixed {} {} {} {} {}",
            segments.length(),
            first.nb_edges(),
            orientation_name(first.orientation()),
            last.nb_edges(),
            orientation_name(last.orientation())
        )
    }

    fn orientation_name(orientation: Orientation) -> &'static str {
        match orientation {
            Orientation::Internal => "internal",
            Orientation::Reversed => "reversed",
            _ => "other",
        }
    }

    fn one_patch_grid() -> Handle<ShapeExtendCompositeSurface> {
        let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
        let cells = HArray2::with_value(1, 1, 1, 1, Handle::<dyn GeomSurface>::default()).unwrap();
        cells.change_array2().set_value(1, 1, surface).unwrap();
        let grid = Handle::new(cells);
        Handle::new(
            ShapeExtendCompositeSurface::with_grid(&grid, ShapeExtendParametrisation::Natural)
                .unwrap(),
        )
    }

    fn square_face() -> Face {
        square_face_with_edge_mode(false)
    }

    fn square_face_with_internal_edge() -> Face {
        square_face_with_edge_mode(true)
    }

    fn square_face_with_edge_mode(internal_first_edge: bool) -> Face {
        let points = [
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(2.0, 0.0, 0.0),
            Point3::new(2.0, 2.0, 0.0),
            Point3::new(0.0, 2.0, 0.0),
        ];
        let mut wire = BRepBuilderAPIMakeWire::new();
        for index in 0..points.len() {
            add_square_edge(
                &mut wire,
                points[index],
                points[(index + 1) % 4],
                internal_first_edge && index == 0,
            );
        }
        BRepBuilderAPIMakeFace::from_plane_wire(Plane3::default(), &wire.wire().unwrap(), true)
            .unwrap()
            .face()
            .unwrap()
    }

    fn add_square_edge(
        wire: &mut BRepBuilderAPIMakeWire,
        start: Point3,
        end: Point3,
        internal: bool,
    ) {
        let mut edge = BRepBuilderAPIMakeEdge::from_points(start, end)
            .unwrap()
            .edge()
            .unwrap();
        if internal {
            edge.set_orientation(Orientation::Internal);
        }
        wire.add_edge(&edge).unwrap();
    }
}
