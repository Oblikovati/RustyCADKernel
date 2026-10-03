#[path = "split_geometry.rs"]
mod geometry;
#[path = "split_wire_faces.rs"]
mod wire_faces;

use super::ShapeFixFixSmallFace;
use crate::shape_analysis::{SplittingVertexEdges, SplittingVertexParameters};
use crate::shape_build::ShapeBuildReShape;
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use crate::shape_fix::FixRoot;
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Builder, Compound, Face, Shape};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;

pub(super) fn fix_split_face(fixer: &ShapeFixFixSmallFace) -> Result<Shape, Failure> {
    let input = fixer.input.read().clone();
    if input.is_null() {
        return Ok(input);
    }
    let done = if split_container_supported(&input)? {
        scan_split_faces(fixer, &input)?
    } else {
        false
    };
    let output = apply_context(fixer, &input)?;
    *fixer.input.write() = output.clone();
    *fixer.result.write() = output.clone();
    if done {
        *fixer.status.write() = ShapeExtend::encode_status(ShapeExtendStatus::Done3);
    }
    Ok(output)
}

pub(super) fn split_one_face(
    fixer: &ShapeFixFixSmallFace,
    face: &mut Face,
    split_faces: &mut Compound,
) -> Result<bool, Failure> {
    let applied = apply_context(fixer, &face.clone().into_shape())?;
    *face = topods::face(&applied)?;
    let Some(vertex) = find_split_vertex(fixer, face)? else {
        return Ok(false);
    };
    let Some(boundary) = geometry::find_and_split_boundary(fixer, face, &vertex)? else {
        return Ok(false);
    };
    complete_recursive_split(fixer, face, boundary, split_faces)
}

fn complete_recursive_split(
    fixer: &ShapeFixFixSmallFace,
    face: &Face,
    boundary: geometry::SplitBoundary,
    split_faces: &mut Compound,
) -> Result<bool, Failure> {
    let Some((first, second)) = wire_faces::split_face_wires(face, &boundary)? else {
        return Ok(false);
    };
    append_recursive_split(fixer, first, split_faces)?;
    append_recursive_split(fixer, second, split_faces)?;
    Ok(true)
}

fn find_split_vertex(
    fixer: &ShapeFixFixSmallFace,
    face: &Face,
) -> Result<Option<rustycad_tkbrep::topods::Vertex>, Failure> {
    let (found, all_vertices) = detect_split_vertices(fixer, face)?;
    if found == 0 {
        return Ok(None);
    }
    let vertices = Explorer::new(
        &all_vertices.into_shape(),
        ShapeEnum::Vertex,
        ShapeEnum::Shape,
    )?;
    Ok(Some(topods::vertex(vertices.current())?))
}

fn detect_split_vertices(
    fixer: &ShapeFixFixSmallFace,
    face: &Face,
) -> Result<(i32, Compound), Failure> {
    let mut edges = SplittingVertexEdges::default();
    let mut parameters = SplittingVertexParameters::default();
    let mut vertices = Compound::default();
    Builder.make_compound(&mut vertices);
    let found = fixer.analyzer.write().check_splitting_vertices(
        face,
        &mut edges,
        &mut parameters,
        &mut vertices,
    )?;
    Ok((found, vertices))
}

fn append_recursive_split(
    fixer: &ShapeFixFixSmallFace,
    mut face: Face,
    output: &mut Compound,
) -> Result<(), Failure> {
    if !split_one_face(fixer, &mut face, output)? {
        Builder.add(&mut output.clone().into_shape(), &face.into_shape())?;
    }
    Ok(())
}

fn split_container_supported(shape: &Shape) -> Result<bool, Failure> {
    Ok(matches!(
        shape.shape_type()?,
        ShapeEnum::Compound
            | ShapeEnum::CompSolid
            | ShapeEnum::Solid
            | ShapeEnum::Shell
            | ShapeEnum::Face
    ))
}

fn scan_split_faces(fixer: &ShapeFixFixSmallFace, shape: &Shape) -> Result<bool, Failure> {
    let context = context(fixer)?;
    let mut faces = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)?;
    let mut done = false;
    while faces.more() {
        done |= split_candidate_face(fixer, context.get().unwrap(), faces.current())?;
        faces.next()?;
    }
    Ok(done)
}

fn split_candidate_face(
    fixer: &ShapeFixFixSmallFace,
    context: &ShapeBuildReShape,
    source: &Shape,
) -> Result<bool, Failure> {
    let mut face = topods::face(source)?;
    let mut output = Compound::default();
    Builder.make_compound(&mut output);
    if !split_one_face(fixer, &mut face, &mut output)? {
        return Ok(false);
    }
    ReShape::replace(context, &face.clone().into_shape(), &output.into_shape())?;
    Ok(true)
}

fn apply_context(fixer: &ShapeFixFixSmallFace, shape: &Shape) -> Result<Shape, Failure> {
    let context = context(fixer)?;
    ReShape::apply(context.get().unwrap(), shape, ShapeEnum::Shape)
}

fn context(fixer: &ShapeFixFixSmallFace) -> Result<Handle<ShapeBuildReShape>, Failure> {
    let context = fixer.context();
    if context.get().is_some() {
        return Ok(context);
    }
    Err(Failure::new(
        "ShapeFix_FixSmallFace requires Init(shape) before splitting",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkbrep::brep::BRepBuilder;
    use rustycad_tkbrep::topexp::Explorer;
    use rustycad_tkbrep::topods::{Builder, Edge};
    use rustycad_tkg3d::top_abs::ShapeEnum;
    use rustycad_tkmath::gp::{Plane3, Point3};
    use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace};

    #[test]
    fn initialized_splitter_matches_native_two_face_result() {
        assert_native_split_output();
        let face = fixture_face();
        assert_detector_output(&face);
        assert_direct_split_output(face.clone());
        assert_aggregate_split_output(face);
    }

    #[test]
    fn initialized_splitter_recursively_splits_zigzag_into_three_faces() {
        assert_native_split_output();
        let face = zigzag_fixture_face();
        let fixer = ShapeFixFixSmallFace::new(face.clone().into_shape());
        fixer.analyzer.write().set_tolerance(1.0e-4);
        let mut output = Compound::default();
        Builder.make_compound(&mut output);
        assert!(
            fixer
                .split_one_face(&mut face.clone(), &mut output)
                .unwrap()
        );
        let shape = output.into_shape();
        assert_eq!(count_faces(&shape).unwrap(), 3);
        assert_eq!(shape_counts(&shape).unwrap(), (13, 26));
    }

    fn assert_native_split_output() {
        let fixture =
            include_str!("../../../../../tools/oracle/tkshhealing_fix_small_face_split.expected");
        let rows = fixture.lines().collect::<Vec<_>>();
        assert_eq!(
            rows,
            [
                "detected 2 1",
                "1 2 6 12",
                "aggregate 2 0 4 6 12",
                "zigzag 1 3 13 26"
            ]
        );
    }

    fn assert_detector_output(face: &Face) {
        let fixer = ShapeFixFixSmallFace::new(face.clone().into_shape());
        fixer.analyzer.write().set_tolerance(1.0e-4);
        let (mut edges, mut parameters, mut vertices) = split_outputs();
        let found = fixer
            .analyzer
            .write()
            .check_splitting_vertices(face, &mut edges, &mut parameters, &mut vertices)
            .unwrap();
        assert_eq!((found, edges.len(), parameters.len()), (2, 1, 1));
    }

    fn assert_direct_split_output(mut face: Face) {
        let fixer = ShapeFixFixSmallFace::new(face.clone().into_shape());
        fixer.analyzer.write().set_tolerance(1.0e-4);
        let mut output = Compound::default();
        Builder.make_compound(&mut output);
        assert!(fixer.split_one_face(&mut face, &mut output).unwrap());
        let shape = output.into_shape();
        assert_eq!(count_faces(&shape).unwrap(), 2);
        assert_eq!(shape_counts(&shape).unwrap(), (6, 12));
    }

    fn assert_aggregate_split_output(face: Face) {
        let fixer = ShapeFixFixSmallFace::new(face.clone().into_shape());
        fixer.analyzer.write().set_tolerance(1.0e-4);
        let result = fixer.fix_split_face(&face.clone().into_shape()).unwrap();
        assert_eq!(result.shape_type().unwrap(), ShapeEnum::Compound);
        assert_eq!(count_faces(&result).unwrap(), 2);
        assert_eq!(shape_counts(&result).unwrap(), (6, 12));
        assert!(fixer.status(ShapeExtendStatus::Done3));
    }

    fn split_outputs() -> (SplittingVertexEdges, SplittingVertexParameters, Compound) {
        let mut vertices = Compound::default();
        Builder.make_compound(&mut vertices);
        (
            SplittingVertexEdges::default(),
            SplittingVertexParameters::default(),
            vertices,
        )
    }

    fn fixture_face() -> Face {
        let vertices = boundary_vertices();
        let edges = [
            make_edge(&vertices[0], &vertices[1]),
            make_edge(&vertices[1], &vertices[2]),
            make_edge(&vertices[2], &vertices[0]),
        ];
        face_from_edges(edges)
    }

    fn zigzag_fixture_face() -> Face {
        let points = [
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(4.0, 0.0, 0.0),
            Point3::new(4.0, 2.0, 0.0),
            Point3::new(3.0, 1.0e-5, 0.0),
            Point3::new(2.0, 2.0, 0.0),
            Point3::new(1.0, 1.0e-5, 0.0),
            Point3::new(0.0, 2.0, 0.0),
        ];
        polygon_face(&points)
    }

    fn polygon_face(points: &[Point3]) -> Face {
        let vertices = polygon_vertices(points);
        let edges = polygon_edges(&vertices);
        face_from_polygon_edges(edges)
    }

    fn polygon_vertices(points: &[Point3]) -> Vec<rustycad_tkbrep::topods::Vertex> {
        points.iter().map(|point| make_vertex(*point)).collect()
    }

    fn make_vertex(point: Point3) -> rustycad_tkbrep::topods::Vertex {
        let mut vertex = rustycad_tkbrep::topods::Vertex::default();
        BRepBuilder::default()
            .make_vertex_at(&mut vertex, point, 1.0e-4)
            .unwrap();
        vertex
    }

    fn polygon_edges(vertices: &[rustycad_tkbrep::topods::Vertex]) -> Vec<Edge> {
        (0..vertices.len())
            .map(|index| make_edge(&vertices[index], &vertices[(index + 1) % vertices.len()]))
            .collect()
    }

    fn face_from_polygon_edges(edges: Vec<Edge>) -> Face {
        let mut wire = rustycad_tkbrep::topods::Wire::default();
        Builder.make_wire(&mut wire);
        let mut shape = wire.into_shape();
        for edge in edges {
            Builder.add(&mut shape, &edge.into_shape()).unwrap();
        }
        let wire = rustycad_tkbrep::topods::wire(&shape).unwrap();
        BRepBuilderAPIMakeFace::from_plane_wire(Plane3::default(), &wire, true)
            .unwrap()
            .face()
            .unwrap()
    }

    fn boundary_vertices() -> [rustycad_tkbrep::topods::Vertex; 3] {
        let builder = BRepBuilder::default();
        let mut vertices = [
            rustycad_tkbrep::topods::Vertex::default(),
            rustycad_tkbrep::topods::Vertex::default(),
            rustycad_tkbrep::topods::Vertex::default(),
        ];
        let points = [
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.5, 1.0e-5, 0.0),
        ];
        for (vertex, point) in vertices.iter_mut().zip(points) {
            builder.make_vertex_at(vertex, point, 1.0e-4).unwrap();
        }
        vertices
    }

    fn make_edge(
        first: &rustycad_tkbrep::topods::Vertex,
        last: &rustycad_tkbrep::topods::Vertex,
    ) -> Edge {
        BRepBuilderAPIMakeEdge::from_vertices(first, last)
            .unwrap()
            .edge()
            .unwrap()
    }

    fn face_from_edges(edges: [Edge; 3]) -> Face {
        let mut wire = rustycad_tkbrep::topods::Wire::default();
        Builder.make_wire(&mut wire);
        let mut shape = wire.into_shape();
        for edge in edges {
            Builder.add(&mut shape, &edge.into_shape()).unwrap();
        }
        let wire = rustycad_tkbrep::topods::wire(&shape).unwrap();
        BRepBuilderAPIMakeFace::from_plane_wire(Plane3::default(), &wire, true)
            .unwrap()
            .face()
            .unwrap()
    }

    fn count_faces(shape: &Shape) -> Result<usize, Failure> {
        let mut explorer = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)?;
        let mut count = 0;
        while explorer.more() {
            count += 1;
            explorer.next()?;
        }
        Ok(count)
    }

    fn shape_counts(shape: &Shape) -> Result<(usize, usize), Failure> {
        let edges = count_shapes(shape, ShapeEnum::Edge)?;
        let vertices = count_shapes(shape, ShapeEnum::Vertex)?;
        Ok((edges, vertices))
    }

    fn count_shapes(shape: &Shape, kind: ShapeEnum) -> Result<usize, Failure> {
        let mut explorer = Explorer::new(shape, kind, ShapeEnum::Shape)?;
        let mut count = 0;
        while explorer.more() {
            count += 1;
            explorer.next()?;
        }
        Ok(count)
    }
}
