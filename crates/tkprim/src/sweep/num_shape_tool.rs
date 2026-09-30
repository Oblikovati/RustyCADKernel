// Rust translation of Sweep_NumShapeTool.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Sweep_NumShapeTool`: the indexation of the sub-shapes of a directing shape.
use super::SweepNumShape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

/// `Sweep_NumShapeTool`: indexes the directing shape (index 1) and its vertices
/// (indices 2 and 3; a closed edge has one vertex, of index 2).
///
/// ```
/// use rustycad_tkg3d::top_abs::ShapeEnum;
/// use rustycad_tkprim::sweep::{SweepNumShape, SweepNumShapeTool};
/// let tool = SweepNumShapeTool::new(&SweepNumShape::new(2, ShapeEnum::Edge));
/// assert_eq!(tool.nb_shapes(), 3);
/// assert_eq!(tool.index(&tool.last_vertex()?), 3);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Copy, Debug)]
pub struct SweepNumShapeTool {
    my_num_shape: SweepNumShape,
}

/// The Standard_ConstructionError of `FirstVertex` and `LastVertex` on an edge
/// infinite at that end.
fn infinite_shape(member: &str, shape: &SweepNumShape) -> Failure {
    Failure::with_message(
        "Standard_ConstructionError",
        &format!(
            "infinite Shape: Sweep_NumShapeTool::{member} on an edge of {} vertices infinite \
             at that end (begin {}, end {}); expected a finite end",
            shape.index(),
            shape.beg_infinite(),
            shape.end_infinite()
        ),
    )
}

impl SweepNumShapeTool {
    /// `Sweep_NumShapeTool(aShape)`; `SweepNumShapeTool::new(&edge)`.
    pub fn new(shape: &SweepNumShape) -> Self {
        Self {
            my_num_shape: *shape,
        }
    }

    /// `NbShapes()`: the edge and its vertices (one vertex fewer when closed), 1 for a
    /// vertex; `tool.nb_shapes()`.
    pub fn nb_shapes(&self) -> i32 {
        if self.my_num_shape.kind() != ShapeEnum::Edge {
            return 1;
        }
        if self.my_num_shape.closed() {
            return self.my_num_shape.index();
        }
        self.my_num_shape.index() + 1
    }

    /// `Index(aShape)`: 1 for an edge, 2 for a vertex of a closed edge, the vertex
    /// index plus 1 otherwise; `tool.index(&vertex)`.
    pub fn index(&self, shape: &SweepNumShape) -> i32 {
        if shape.kind() == ShapeEnum::Edge {
            return 1;
        }
        if shape.closed() {
            return 2;
        }
        shape.index() + 1
    }

    /// `Shape(anIndex)`: the directing shape for 1, the vertex of index `anIndex - 1`
    /// otherwise (no range check, as natively); `tool.shape(2)`.
    pub fn shape(&self, index: i32) -> SweepNumShape {
        if index == 1 {
            return self.my_num_shape;
        }
        self.vertex(index - 1)
    }

    /// `Type(aShape)`; `tool.kind(&shape)`.
    pub fn kind(&self, shape: &SweepNumShape) -> ShapeEnum {
        shape.kind()
    }

    /// `Orientation(aShape)`; `tool.orientation(&shape)`.
    pub fn orientation(&self, shape: &SweepNumShape) -> Orientation {
        shape.orientation()
    }

    /// `HasFirstVertex()`: false only for an edge infinite at its beginning;
    /// `tool.has_first_vertex()`.
    pub fn has_first_vertex(&self) -> bool {
        self.my_num_shape.kind() != ShapeEnum::Edge || !self.my_num_shape.beg_infinite()
    }

    /// `HasLastVertex()`: false only for an edge infinite at its end;
    /// `tool.has_last_vertex()`.
    pub fn has_last_vertex(&self) -> bool {
        self.my_num_shape.kind() != ShapeEnum::Edge || !self.my_num_shape.end_infinite()
    }

    /// `FirstVertex()`: the vertex of index 1 of an edge, the shape itself otherwise;
    /// Standard_ConstructionError when the edge is infinite at its beginning;
    /// `tool.first_vertex()?`.
    pub fn first_vertex(&self) -> Result<SweepNumShape, Failure> {
        if self.my_num_shape.kind() != ShapeEnum::Edge {
            return Ok(self.my_num_shape);
        }
        if !self.has_first_vertex() {
            return Err(infinite_shape("FirstVertex", &self.my_num_shape));
        }
        Ok(self.vertex(1))
    }

    /// `LastVertex()`: the vertex of index `NbShapes() - 1` of an edge, the shape
    /// itself otherwise; Standard_ConstructionError when the edge is infinite at its
    /// end; `tool.last_vertex()?`.
    pub fn last_vertex(&self) -> Result<SweepNumShape, Failure> {
        if self.my_num_shape.kind() != ShapeEnum::Edge {
            return Ok(self.my_num_shape);
        }
        if !self.has_last_vertex() {
            return Err(infinite_shape("LastVertex", &self.my_num_shape));
        }
        // Built directly, not through Shape(): a closed edge of one vertex (or none) has
        // NbShapes() - 1 == 0 (or -1), which Shape() would not map to a vertex.
        Ok(self.vertex(self.nb_shapes() - 1))
    }

    /// The vertex of index `index` of the directing edge.
    fn vertex(&self, index: i32) -> SweepNumShape {
        SweepNumShape::with_flags(
            index,
            ShapeEnum::Vertex,
            self.my_num_shape.closed(),
            false,
            false,
        )
    }
}
