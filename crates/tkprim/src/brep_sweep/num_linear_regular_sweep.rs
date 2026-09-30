// Rust translation of BRepSweep_NumLinearRegularSweep.hxx/.cxx (the data, the virtual
// members and the accessors) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepSweep_NumLinearRegularSweep`: the generic sweep of a generating shape along a
//! directing edge, as a trait over a base struct (docs/prim-design.md section 3). The
//! construction of the grid of shapes is in `sweep_shape.rs`.
use super::sweep_shape;
use super::{BRepSweepBuilder, BRepSweepTool};
use crate::sweep::{SweepNumShape, SweepNumShapeTool};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::{Array2, ArrayError};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

/// The Standard_OutOfRange of a grid access (an index 0 for a shape that is not a
/// sub-shape of the generating shape, natively an NCollection_Array2 range check).
pub(crate) fn grid_failure(error: ArrayError) -> Failure {
    Failure::out_of_range(&format!(
        "BRepSweep_NumLinearRegularSweep: {error:?}; expected a sub-shape of the generating shape"
    ))
}

/// The protected data of `BRepSweep_NumLinearRegularSweep`: the builder, the
/// generating shape and directing edge with their tools, and the grid of built shapes
/// indexed by (generating index, directing index), with its built and used flags.
///
/// ```
/// use rustycad_tkbrep::brep::BRepBuilder;
/// use rustycad_tkbrep::topods::Shape;
/// use rustycad_tkg3d::top_abs::ShapeEnum;
/// use rustycad_tkprim::brep_sweep::{BRepSweepBuilder, BRepSweepNumLinearRegularSweepBase};
/// use rustycad_tkprim::sweep::SweepNumShape;
/// let builder = BRepSweepBuilder::new(BRepBuilder::default());
/// let base = BRepSweepNumLinearRegularSweepBase::new(builder, &Shape::default(), SweepNumShape::new(2, ShapeEnum::Edge))?;
/// assert_eq!(base.my_dir_shape_tool.nb_shapes(), 3);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct BRepSweepNumLinearRegularSweepBase {
    /// `myBuilder`.
    pub my_builder: BRepSweepBuilder,
    /// `myGenShape`: the generating shape.
    pub my_gen_shape: Shape,
    /// `myDirWire`: the directing edge.
    pub my_dir_wire: SweepNumShape,
    /// `myGenShapeTool`.
    pub my_gen_shape_tool: BRepSweepTool,
    /// `myDirShapeTool`.
    pub my_dir_shape_tool: SweepNumShapeTool,
    /// `myShapes`: the built shapes.
    pub my_shapes: Array2<'static, Shape>,
    /// `myBuiltShapes`: whether each shape is built.
    pub my_built_shapes: Array2<'static, bool>,
    /// `myUsedShapes`: whether each shape is used in its container.
    pub my_used_shapes: Array2<'static, bool>,
}

impl BRepSweepNumLinearRegularSweepBase {
    /// `BRepSweep_NumLinearRegularSweep(aBuilder, aGenShape, aDirWire)` (protected): the
    /// grid of NbShapes(generating) x NbShapes(directing) empty, unbuilt, unused shapes;
    /// `BRepSweepNumLinearRegularSweepBase::new(builder, &shape, edge)?`.
    pub fn new(
        builder: BRepSweepBuilder,
        gen_shape: &Shape,
        dir_wire: SweepNumShape,
    ) -> Result<Self, Failure> {
        let gen_tool = BRepSweepTool::new(gen_shape);
        let dir_tool = SweepNumShapeTool::new(&dir_wire);
        let (rows, cols) = (gen_tool.nb_shapes(), dir_tool.nb_shapes());
        Ok(Self {
            my_builder: builder,
            my_gen_shape: gen_shape.clone(),
            my_dir_wire: dir_wire,
            my_gen_shape_tool: gen_tool,
            my_dir_shape_tool: dir_tool,
            my_shapes: Array2::with_bounds(1, rows, 1, cols).map_err(grid_failure)?,
            my_built_shapes: Array2::with_bounds(1, rows, 1, cols).map_err(grid_failure)?,
            my_used_shapes: Array2::with_bounds(1, rows, 1, cols).map_err(grid_failure)?,
        })
    }

    /// `myShapes(i, j)`: the shape of the grid cell; Standard_OutOfRange outside it.
    pub fn cell(&self, i: i32, j: i32) -> Result<&Shape, Failure> {
        self.my_shapes.value(i, j).map_err(grid_failure)
    }

    /// `myShapes(i, j) = shape`.
    pub fn set_cell(&mut self, i: i32, j: i32, shape: Shape) -> Result<(), Failure> {
        self.my_shapes.set_value(i, j, shape).map_err(grid_failure)
    }

    /// `myBuiltShapes(i, j)`.
    pub fn built(&self, i: i32, j: i32) -> Result<bool, Failure> {
        self.my_built_shapes
            .value(i, j)
            .copied()
            .map_err(grid_failure)
    }

    /// `myBuiltShapes(i, j) = value`.
    pub fn set_built(&mut self, i: i32, j: i32, value: bool) -> Result<(), Failure> {
        self.my_built_shapes
            .set_value(i, j, value)
            .map_err(grid_failure)
    }

    /// `myUsedShapes(i, j)`.
    pub fn used(&self, i: i32, j: i32) -> Result<bool, Failure> {
        self.my_used_shapes
            .value(i, j)
            .copied()
            .map_err(grid_failure)
    }

    /// `myUsedShapes(i, j) = value`.
    pub fn set_used(&mut self, i: i32, j: i32, value: bool) -> Result<(), Failure> {
        self.my_used_shapes
            .set_value(i, j, value)
            .map_err(grid_failure)
    }
}

/// The address of a new shape in the generic algorithm: the generating shape, its
/// generating sub-shape (or the same shape) and the directing shape.
pub type SweepAddress<'a> = (&'a Shape, &'a Shape, &'a SweepNumShape);

/// `BRepSweep_NumLinearRegularSweep` (docs/prim-design.md section 3): a grid of shapes
/// addressed by a generating sub-shape and a directing shape (a vertex or the edge).
/// Implementors provide the base accessors and the geometric members; the shape
/// accessors are provided and never overridden (non-virtual natively).
pub trait BRepSweepNumLinearRegularSweep {
    /// The embedded base data.
    fn sweep_base(&self) -> &BRepSweepNumLinearRegularSweepBase;

    /// The embedded base data, to write.
    fn sweep_base_mut(&mut self) -> &mut BRepSweepNumLinearRegularSweepBase;

    /// `MakeEmptyVertex(aGenV, aDirV)`: the vertex addressed by `[gen_v, dir_v]`, with
    /// its geometry but no sub-shape; `sweep.make_empty_vertex(&v, &dir_v)?`.
    fn make_empty_vertex(&mut self, gen_v: &Shape, dir_v: &SweepNumShape)
    -> Result<Shape, Failure>;

    /// `MakeEmptyDirectingEdge(aGenV, aDirE)`: the edge addressed by `[gen_v, dir_e]`;
    /// `sweep.make_empty_directing_edge(&v, &dir_e)?`.
    fn make_empty_directing_edge(
        &mut self,
        gen_v: &Shape,
        dir_e: &SweepNumShape,
    ) -> Result<Shape, Failure>;

    /// `MakeEmptyGeneratingEdge(aGenE, aDirV)`: the edge addressed by `[gen_e, dir_v]`;
    /// `sweep.make_empty_generating_edge(&e, &dir_v)?`.
    fn make_empty_generating_edge(
        &mut self,
        gen_e: &Shape,
        dir_v: &SweepNumShape,
    ) -> Result<Shape, Failure>;

    /// `SetParameters(aNewFace, aNewVertex, aGenF, aGenV, aDirV)`: the parameters of the
    /// new vertex on the new face; `sweep.set_parameters(&f, &v, (&gf, &gv, &dv))?`.
    fn set_parameters(
        &mut self,
        new_face: &Shape,
        new_vertex: &Shape,
        address: SweepAddress<'_>,
    ) -> Result<(), Failure>;

    /// `SetDirectingParameter(aNewEdge, aNewVertex, aGenV, aDirE, aDirV)`: the parameter
    /// of the new vertex on the new directing edge;
    /// `sweep.set_directing_parameter(&e, &v, &gv, &de, &dv)?`.
    fn set_directing_parameter(
        &mut self,
        new_edge: &Shape,
        new_vertex: &Shape,
        gen_v: &Shape,
        dir_e: &SweepNumShape,
        dir_v: &SweepNumShape,
    ) -> Result<(), Failure>;

    /// `SetGeneratingParameter(aNewEdge, aNewVertex, aGenE, aGenV, aDirV)`: the parameter
    /// of the new vertex on the new generating edge;
    /// `sweep.set_generating_parameter(&e, &v, (&ge, &gv, &dv))?`.
    fn set_generating_parameter(
        &mut self,
        new_edge: &Shape,
        new_vertex: &Shape,
        address: SweepAddress<'_>,
    ) -> Result<(), Failure>;

    /// `MakeEmptyFace(aGenS, aDirS)`: the face addressed by `[gen_s, dir_s]` (a
    /// generating face and a directing vertex, or a generating edge and the edge);
    /// `sweep.make_empty_face(&s, &dir_s)?`.
    fn make_empty_face(&mut self, gen_s: &Shape, dir_s: &SweepNumShape) -> Result<Shape, Failure>;

    /// `SetPCurve(aNewFace, aNewEdge, aGenF, aGenE, aDirV, orien)`: the p-curve of a new
    /// edge on a new cap face; `sweep.set_pcurve(&f, &e, (&gf, &ge, &dv), orien)?`.
    fn set_pcurve(
        &mut self,
        new_face: &Shape,
        new_edge: &Shape,
        address: SweepAddress<'_>,
        orien: Orientation,
    ) -> Result<(), Failure>;

    /// `SetGeneratingPCurve(aNewFace, aNewEdge, aGenE, aDirE, aDirV, orien)`: the p-curve
    /// of a new generating edge on a new side face;
    /// `sweep.set_generating_pcurve(&f, &e, &ge, (&de, &dv), orien)?`.
    fn set_generating_pcurve(
        &mut self,
        new_face: &Shape,
        new_edge: &Shape,
        gen_e: &Shape,
        dirs: (&SweepNumShape, &SweepNumShape),
        orien: Orientation,
    ) -> Result<(), Failure>;

    /// `SetDirectingPCurve(aNewFace, aNewEdge, aGenE, aGenV, aDirE, orien)`: the p-curve
    /// of a new directing edge on a new side face;
    /// `sweep.set_directing_pcurve(&f, &e, (&ge, &gv, &de), orien)?`.
    fn set_directing_pcurve(
        &mut self,
        new_face: &Shape,
        new_edge: &Shape,
        address: SweepAddress<'_>,
        orien: Orientation,
    ) -> Result<(), Failure>;

    /// `DirectSolid(aGenS, aDirS)`: the orientation of the shell in the solid swept from
    /// the face `gen_s` (REVERSED when swept along the normal);
    /// `sweep.direct_solid(&face, &dir_s)?`.
    fn direct_solid(
        &mut self,
        gen_s: &Shape,
        dir_s: &SweepNumShape,
    ) -> Result<Orientation, Failure>;

    /// `GGDShapeIsToAdd(aNewShape, aNewSubShape, aGenS, aSubGenS, aDirS)`: whether the
    /// new sub-shape of a generating sub-shape goes into the new shape;
    /// `sweep.ggd_shape_is_to_add(&s, &sub, (&gs, &sgs, &ds))?`.
    fn ggd_shape_is_to_add(
        &self,
        new_shape: &Shape,
        new_sub_shape: &Shape,
        address: SweepAddress<'_>,
    ) -> Result<bool, Failure>;

    /// `GDDShapeIsToAdd(aNewShape, aNewSubShape, aGenS, aDirS, aSubDirS)`: whether the
    /// new sub-shape of a directing sub-shape goes into the new shape;
    /// `sweep.gdd_shape_is_to_add(&s, &sub, &gs, (&ds, &sds))?`.
    fn gdd_shape_is_to_add(
        &self,
        new_shape: &Shape,
        new_sub_shape: &Shape,
        gen_s: &Shape,
        dirs: (&SweepNumShape, &SweepNumShape),
    ) -> Result<bool, Failure>;

    /// `SeparatedWires(aNewShape, aNewSubShape, aGenS, aSubGenS, aDirS)`: whether the
    /// new face is made of independent closed wires;
    /// `sweep.separated_wires(&s, &sub, (&gs, &sgs, &ds))?`.
    fn separated_wires(
        &self,
        new_shape: &Shape,
        new_sub_shape: &Shape,
        address: SweepAddress<'_>,
    ) -> Result<bool, Failure>;

    /// `SplitShell(aNewShape)`: the compound of the independent closed shells of a new
    /// shell; natively a compound holding the shell itself;
    /// `sweep.split_shell(&shell)?`.
    fn split_shell(&self, new_shape: &Shape) -> Result<Shape, Failure> {
        let builder = self.sweep_base().my_builder;
        let mut compound = Shape::default();
        builder.make_compound(&mut compound)?;
        builder.add(&mut compound, new_shape)?;
        Ok(compound)
    }

    /// `SetContinuity(aGenS, aDirS)`: propagates the continuity of the generating wire
    /// on the new edges and faces; `sweep.set_continuity(&edge, &dir_s)?`.
    fn set_continuity(&mut self, gen_s: &Shape, dir_s: &SweepNumShape) -> Result<(), Failure>;

    /// `HasShape(aGenS, aDirS)`: whether `[gen_s, dir_s]` addresses a shape (a
    /// degenerated or seam edge, for example, does not); `sweep.has_shape(&s, &ds)?`.
    fn has_shape(&self, gen_s: &Shape, dir_s: &SweepNumShape) -> Result<bool, Failure>;

    /// `IsInvariant(aGenS)`: whether the sweep does not move `gen_s`;
    /// `sweep.is_invariant(&s)?`.
    fn is_invariant(&self, gen_s: &Shape) -> Result<bool, Failure>;

    /// `Shape(aGenS, aDirS)`: the shape addressed by `[gen_s, dir_s]`, built once;
    /// Standard_OutOfRange for a shape that is not a generating sub-shape;
    /// `sweep.shape_at(&s, &ds)?`.
    fn shape_at(&mut self, gen_s: &Shape, dir_s: &SweepNumShape) -> Result<Shape, Failure> {
        sweep_shape::shape_at(self, gen_s, dir_s)
    }

    /// `Shape(aGenS)`: the shape generated by `gen_s` along the directing edge, null
    /// when `gen_s` is foreign or generates nothing; `sweep.shape_of(&s)?`.
    fn shape_of(&mut self, gen_s: &Shape) -> Result<Shape, Failure> {
        let dir_wire = self.sweep_base().my_dir_wire;
        if self.sweep_base().my_gen_shape_tool.index(gen_s) != 0
            && self.has_shape(gen_s, &dir_wire)?
        {
            return self.shape_at(gen_s, &dir_wire);
        }
        Ok(Shape::default())
    }

    /// `Shape()`: the shape generated by the generating shape; `sweep.shape()?`.
    fn shape(&mut self) -> Result<Shape, Failure> {
        let (gen_shape, dir_wire) = (
            self.sweep_base().my_gen_shape.clone(),
            self.sweep_base().my_dir_wire,
        );
        if self.has_shape(&gen_shape, &dir_wire)? {
            return self.shape_at(&gen_shape, &dir_wire);
        }
        Ok(Shape::default())
    }

    /// `IsUsed(aGenS)`: whether a shape generated by `gen_s` is used in the result;
    /// `sweep.is_used(&s)?`.
    fn is_used(&self, gen_s: &Shape) -> Result<bool, Failure> {
        sweep_shape::is_used(self, gen_s)
    }

    /// `GenIsUsed(theS)`: whether the shape generated by `gen_s` at the first directing
    /// vertex is used in the result; `sweep.gen_is_used(&s)?`.
    fn gen_is_used(&self, gen_s: &Shape) -> Result<bool, Failure> {
        let base = self.sweep_base();
        let i_gen = base.my_gen_shape_tool.index(gen_s);
        if i_gen == 0 {
            return Ok(false);
        }
        if i_gen == 1 {
            return base.built(i_gen, 1);
        }
        Ok(base.built(i_gen, 1)? && base.used(i_gen, 1)?)
    }

    /// `FirstShape()`: the shape at the first directing vertex, null without one;
    /// `sweep.first_shape()?`.
    fn first_shape(&mut self) -> Result<Shape, Failure> {
        let gen_shape = self.sweep_base().my_gen_shape.clone();
        self.first_shape_of(&gen_shape)
    }

    /// `LastShape()`: the shape at the last directing vertex, null without one;
    /// `sweep.last_shape()?`.
    fn last_shape(&mut self) -> Result<Shape, Failure> {
        let gen_shape = self.sweep_base().my_gen_shape.clone();
        self.last_shape_of(&gen_shape)
    }

    /// `FirstShape(aGenS)`: the shape of `gen_s` at the first directing vertex;
    /// Standard_OutOfRange for a foreign shape; `sweep.first_shape_of(&s)?`.
    fn first_shape_of(&mut self, gen_s: &Shape) -> Result<Shape, Failure> {
        let tool = self.sweep_base().my_dir_shape_tool;
        if !tool.has_first_vertex() {
            return Ok(Shape::default());
        }
        let vertex = tool.first_vertex()?;
        if self.has_shape(gen_s, &vertex)? {
            return self.shape_at(gen_s, &vertex);
        }
        Ok(Shape::default())
    }

    /// `LastShape(aGenS)`: the shape of `gen_s` at the last directing vertex;
    /// Standard_OutOfRange for a foreign shape; `sweep.last_shape_of(&s)?`.
    fn last_shape_of(&mut self, gen_s: &Shape) -> Result<Shape, Failure> {
        let tool = self.sweep_base().my_dir_shape_tool;
        if !tool.has_last_vertex() {
            return Ok(Shape::default());
        }
        let vertex = tool.last_vertex()?;
        if self.has_shape(gen_s, &vertex)? {
            return self.shape_at(gen_s, &vertex);
        }
        Ok(Shape::default())
    }

    /// `Closed()`: whether the directing edge is closed; `sweep.closed()`.
    fn closed(&self) -> bool {
        self.sweep_base().my_dir_wire.closed()
    }
}

/// The shape type of `shape` (Standard_NullObject for a null shape).
pub(crate) fn kind_of(shape: &Shape) -> Result<ShapeEnum, Failure> {
    shape.shape_type()
}
