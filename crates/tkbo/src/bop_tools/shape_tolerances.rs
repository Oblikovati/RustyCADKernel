use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topexp::Explorer,
    topods::{Edge, Face, Shape, ShapeIterator, Vertex},
    toptools::IndexedMapOfShape,
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
/// Propagate edge then face tolerances in native pass order; `correct_shape_tolerances(&shape,&avoided,false)?`.
pub fn correct_shape_tolerances(
    shape: &Shape,
    avoided: &IndexedMapOfShape,
    _run_parallel: bool,
) -> Result<(), Failure> {
    for edge in Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)? {
        correct_vertices(&Edge::unchecked(edge), avoided)?;
    }
    for face in Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)? {
        correct_face_children(&Face::unchecked(face), avoided)?;
    }
    Ok(())
}
fn correct_vertices(edge: &Edge, avoided: &IndexedMapOfShape) -> Result<(), Failure> {
    let tolerance = BRepTool::edge_tolerance(edge)?;
    for shape in ShapeIterator::new(edge, true, true) {
        let vertex = Vertex::unchecked(shape);
        if BRepTool::vertex_tolerance(&vertex)? < tolerance {
            update_tolerance(&vertex, tolerance, avoided)?;
        }
    }
    Ok(())
}
fn correct_face_children(face: &Face, avoided: &IndexedMapOfShape) -> Result<(), Failure> {
    let face_tolerance = BRepTool::face_tolerance(face)?;
    let mut edge_tolerance = face_tolerance;
    for child in ShapeIterator::new(face, true, true) {
        if child.shape_type()? == ShapeEnum::Wire {
            update_wire_edges(&child, face_tolerance, &mut edge_tolerance, avoided)?;
            continue;
        }
        let vertex = Vertex::unchecked(child);
        if BRepTool::vertex_tolerance(&vertex)? < edge_tolerance {
            update_tolerance(&vertex, face_tolerance, avoided)?;
        }
    }
    Ok(())
}
fn update_wire_edges(
    wire: &Shape,
    face_tolerance: f64,
    edge_tolerance: &mut f64,
    avoided: &IndexedMapOfShape,
) -> Result<(), Failure> {
    for shape in ShapeIterator::new(wire, true, true) {
        let edge = Edge::unchecked(shape);
        *edge_tolerance = BRepTool::edge_tolerance(&edge)?;
        if *edge_tolerance < face_tolerance {
            update_tolerance(&edge, face_tolerance, avoided)?;
            *edge_tolerance = face_tolerance;
        }
    }
    Ok(())
}
fn update_tolerance(
    shape: &Shape,
    tolerance: f64,
    avoided: &IndexedMapOfShape,
) -> Result<(), Failure> {
    if avoided.contains(shape) {
        return Ok(());
    }
    let builder = BRepBuilder::default();
    match shape.shape_type()? {
        ShapeEnum::Edge => {
            builder.update_edge_tolerance(&Edge::unchecked(shape.clone()), tolerance)
        }
        ShapeEnum::Vertex => {
            builder.update_vertex_tolerance(&Vertex::unchecked(shape.clone()), tolerance)
        }
        _ => Ok(()),
    }
}
