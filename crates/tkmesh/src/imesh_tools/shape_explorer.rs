//! IMeshTools_ShapeExplorer preserves native free-edge then sorted-face order.
use super::ShapeVisitor;
use crate::imesh_data::{
    MeshShape,
    shape::{ShapeState, mesh_shape_members},
};
use rustycad_tkbrep::{
    brep::BRepTool,
    topexp::Explorer,
    topods::{self, Shape},
    toptools::MapOfShape,
};
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Handle, StandardTransient},
};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::toploc::Location;
/// Native shape traversal; `ShapeExplorer::new(&shape).accept(&visitor)?`.
pub struct ShapeExplorer {
    transient: StandardTransient,
    shape: ShapeState,
}
impl_transient!(ShapeExplorer, "IMeshTools_ShapeExplorer", dyn MeshShape, transient, interfaces: [dyn MeshShape]);
mesh_shape_members!(ShapeExplorer);
impl ShapeExplorer {
    /// Keep the input topology; `ShapeExplorer::new(&shape)`.
    pub fn new(shape: &Shape) -> Self {
        Self {
            transient: StandardTransient::default(),
            shape: ShapeState::new(shape),
        }
    }
    /// Visit geometric free edges and unique geometric faces in native order.
    /// Example: `explorer.accept(&visitor)?`.
    pub fn accept(&self, visitor: &Handle<dyn ShapeVisitor>) -> Result<(), Failure> {
        let visitor = visitor.get().ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "visitor is null; expected ShapeVisitor handle",
            )
        })?;
        let shape = self.get_shape();
        visit_edges(visitor, &shape, true, ShapeEnum::Face)?;
        visit_faces(visitor, &self.get_shape())
    }
}
fn visit_edges(
    visitor: &dyn ShapeVisitor,
    shape: &Shape,
    reset_location: bool,
    avoid: ShapeEnum,
) -> Result<(), Failure> {
    let mut explorer = Explorer::new(shape, ShapeEnum::Edge, avoid)?;
    while explorer.more() {
        let edge = topods::edge(explorer.current())?;
        if !BRepTool::is_geometric_edge(&edge)? {
            explorer.next()?;
            continue;
        }
        let located = if reset_location {
            topods::edge(&edge.located(&Location::default(), false)?)?
        } else {
            edge
        };
        visitor.visit_edge(&located)?;
        explorer.next()?;
    }
    Ok(())
}
fn visit_faces(visitor: &dyn ShapeVisitor, shape: &Shape) -> Result<(), Failure> {
    let faces = rustycad_tktopalgo::brep_lib::reverse_sort_faces(shape)?;
    let mut visited = MapOfShape::default();
    for shape in faces.iter() {
        let unlocated = shape.located(&Location::default(), false)?;
        if !visited.add(unlocated) {
            continue;
        }
        let face = topods::face(shape)?;
        if !BRepTool::is_geometric_face(&face)? {
            continue;
        }
        visit_edges(visitor, &face, false, ShapeEnum::Shape)?;
        visitor.visit_face(&topods::face(&face.oriented(Orientation::Forward))?)?;
    }
    Ok(())
}
