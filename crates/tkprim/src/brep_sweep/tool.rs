// Rust translation of BRepSweep_Tool.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepSweep_Tool`: the indexation of the sub-shapes of a generating shape.
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

/// `BRepSweep_Tool`: the generating shape and all its sub-shapes, indexed from 1 in
/// `TopExp::MapShapes` order (a shape and its sub-shapes are the same entry whatever
/// their orientation).
///
/// ```
/// use rustycad_tkbrep::brep::BRepBuilder;
/// use rustycad_tkbrep::topods::Vertex;
/// use rustycad_tkmath::gp::Point3;
/// use rustycad_tkprim::brep_sweep::BRepSweepTool;
/// let mut vertex = Vertex::default();
/// BRepBuilder::default().make_vertex_at(&mut vertex, Point3::new(0.0, 0.0, 0.0), 1e-7)?;
/// let tool = BRepSweepTool::new(&vertex);
/// assert_eq!(tool.nb_shapes(), 1);
/// assert_eq!(tool.index(&vertex), 1);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Default)]
pub struct BRepSweepTool {
    my_map: IndexedMapOfShape,
}

impl BRepSweepTool {
    /// `BRepSweep_Tool(aShape)`: indexes `shape` and its sub-shapes;
    /// `BRepSweepTool::new(&shape)`.
    pub fn new(shape: &Shape) -> Self {
        let mut my_map = IndexedMapOfShape::default();
        topexp::map_shapes(shape, &mut my_map, true, true);
        Self { my_map }
    }

    /// `NbShapes()`; `tool.nb_shapes()`.
    pub fn nb_shapes(&self) -> i32 {
        self.my_map.extent() as i32
    }

    /// `Index(aShape)`: the index of `shape`, 0 when it is not a sub-shape;
    /// `tool.index(&shape)`.
    pub fn index(&self, shape: &Shape) -> i32 {
        self.my_map.find_index(shape) as i32
    }

    /// `Shape(anIndex)`: the sub-shape of index `index`; Standard_OutOfRange outside
    /// 1..NbShapes; `tool.shape(1)?`.
    pub fn shape(&self, index: i32) -> Result<Shape, Failure> {
        if index < 1 {
            return Err(Failure::out_of_range(&format!(
                "BRepSweep_Tool::Shape: index {index} outside 1..{}",
                self.nb_shapes()
            )));
        }
        Ok(self.my_map.find_key(index as usize)?.clone())
    }

    /// `Type(aShape)`; `tool.kind(&shape)?`.
    pub fn kind(&self, shape: &Shape) -> Result<ShapeEnum, Failure> {
        shape.shape_type()
    }

    /// `Orientation(aShape)`; `tool.orientation(&shape)`.
    pub fn orientation(&self, shape: &Shape) -> Orientation {
        shape.orientation()
    }

    /// `SetOrientation(aShape, Or)`: sets the orientation of `shape`;
    /// `tool.set_orientation(&mut shape, Orientation::Forward)`.
    pub fn set_orientation(&self, shape: &mut Shape, orientation: Orientation) {
        shape.set_orientation(orientation);
    }
}
