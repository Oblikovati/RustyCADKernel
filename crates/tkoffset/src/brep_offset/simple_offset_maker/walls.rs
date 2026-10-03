use super::BRepOffsetSimpleStatus;
use super::build_simple_offset_wall;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::{Modifier, Quilt, ReShape};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Compound, Edge, Shape, Shell, Vertex};
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkshhealing::shape_analysis::ShapeAnalysisFreeBounds;
use rustycad_tkshhealing::shape_build::ShapeBuildReShape;
use rustycad_tkshhealing::shape_fix::ShapeFixEdge;

pub(super) fn make_solid(
    source: &Shape,
    offset: &Shape,
    modifier: &Modifier,
    reshape: &Handle<ShapeBuildReShape>,
    wall_edges: &mut DataMap<Vertex, Edge>,
) -> Result<Result<Shell, BRepOffsetSimpleStatus>, Failure> {
    let walls = match build_wall_faces(source, modifier, wall_edges)? {
        Ok(walls) => walls,
        Err(status) => return Ok(Err(status)),
    };
    fix_wall_edge_parameters(&walls, reshape)?;
    let compound = combine_faces(source, offset, &walls)?;
    let shells = quilt_shells(&compound, reshape)?;
    sole_closed_shell(&shells)
}

fn build_wall_faces(
    source: &Shape,
    modifier: &Modifier,
    wall_edges: &mut DataMap<Vertex, Edge>,
) -> Result<Result<Compound, BRepOffsetSimpleStatus>, Failure> {
    let free_bounds = ShapeAnalysisFreeBounds::from_shape(source, false, true, false)?;
    let builder = BRepBuilder::default();
    let mut walls = Compound::default();
    builder.make_compound(&mut walls);
    let wires = free_bounds.closed_wires().clone().into_shape();
    for edge in Explorer::new(&wires, ShapeEnum::Edge, ShapeEnum::Shape)? {
        if !add_wall(&edge, modifier, wall_edges, &builder, &mut walls)? {
            return Ok(Err(BRepOffsetSimpleStatus::ErrorWallFaceComputation));
        }
    }
    Ok(Ok(walls))
}

fn add_wall(
    source_edge: &Shape,
    modifier: &Modifier,
    wall_edges: &mut DataMap<Vertex, Edge>,
    builder: &BRepBuilder,
    walls: &mut Compound,
) -> Result<bool, Failure> {
    let source_edge = Edge::unchecked(source_edge.clone());
    let offset_edge = Edge::unchecked(modifier.modified_shape(&source_edge)?);
    let wall = build_simple_offset_wall(&source_edge, &offset_edge, wall_edges)?;
    if wall.is_null() {
        return Ok(false);
    }
    builder.add(walls, &wall.into_shape())?;
    Ok(true)
}

fn fix_wall_edge_parameters(
    walls: &Compound,
    reshape: &Handle<ShapeBuildReShape>,
) -> Result<(), Failure> {
    let shape = walls.clone().into_shape();
    let mut fixer = ShapeFixEdge::new();
    fixer.set_context(reshape);
    for edge in Explorer::new(&shape, ShapeEnum::Edge, ShapeEnum::Shape)? {
        fixer.fix_same_parameter(&Edge::unchecked(edge), 0.0)?;
    }
    Ok(())
}

fn combine_faces(source: &Shape, offset: &Shape, walls: &Compound) -> Result<Compound, Failure> {
    let builder = BRepBuilder::default();
    let mut faces = Compound::default();
    builder.make_compound(&mut faces);
    add_faces(source, &builder, &mut faces)?;
    add_faces(offset, &builder, &mut faces)?;
    add_faces(&walls.clone().into_shape(), &builder, &mut faces)?;
    Ok(faces)
}

fn add_faces(shape: &Shape, builder: &BRepBuilder, faces: &mut Compound) -> Result<(), Failure> {
    for face in Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)? {
        builder.add(faces, &face)?;
    }
    Ok(())
}

fn quilt_shells(
    compound: &Compound,
    reshape: &Handle<ShapeBuildReShape>,
) -> Result<Shape, Failure> {
    let compound = reshape
        .get()
        .expect("reshape handle is initialized")
        .apply(&compound.clone().into_shape(), ShapeEnum::Shape)?;
    let mut quilt = Quilt::new();
    quilt.add(&compound)?;
    quilt.shells()
}

fn sole_closed_shell(shells: &Shape) -> Result<Result<Shell, BRepOffsetSimpleStatus>, Failure> {
    let mut result = Shell::default();
    for shell in Explorer::new(shells, ShapeEnum::Shell, ShapeEnum::Shape)? {
        if !result.is_null() {
            return Ok(Err(BRepOffsetSimpleStatus::ErrorInvalidNbShells));
        }
        result = Shell::unchecked(shell);
    }
    if result.is_null() {
        return Ok(Err(BRepOffsetSimpleStatus::ErrorNonClosedShell));
    }
    if !BRepTool::is_closed(&result)? {
        return Ok(Err(BRepOffsetSimpleStatus::ErrorNonClosedShell));
    }
    Ok(Ok(result))
}
