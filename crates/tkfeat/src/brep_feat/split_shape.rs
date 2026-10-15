// Rust translation of ModelingAlgorithms/TKFeat/BRepFeat/BRepFeat_SplitShape.cxx and
// .lxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepFeat_SplitShape`: the faces of a shape split by wires and edges lying on them
//! (`LocOpe_Spliter` over `LocOpe_WiresOnShape`), as a shape builder with the faces on
//! the left and right of the splitting wires.
use crate::errors::null_object;
use crate::loc_ope::{Spliter, WiresOnShape};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Compound, Edge, Face, Shape, Wire};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::ncollection::{List, Sequence};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeShape, BRepBuilderAPIMakeShapeBase};

/// `BRepFeat_SplitShape`: the splitting of a shape by wires on its faces.
pub struct BRepFeatSplitShape {
    base: BRepBuilderAPIMakeShapeBase,
    s_shape: Spliter,
    w_on_shape: Option<WiresOnShape>,
    right: List<Shape>,
}

impl Default for BRepFeatSplitShape {
    fn default() -> Self {
        Self::new()
    }
}

impl BRepFeatSplitShape {
    /// `BRepFeat_SplitShape()`: no shape (the wires added only after `init`);
    /// `BRepFeatSplitShape::new()`.
    pub fn new() -> Self {
        Self {
            base: BRepBuilderAPIMakeShapeBase::new(),
            s_shape: Spliter::new(),
            w_on_shape: None,
            right: List::default(),
        }
    }

    /// `BRepFeat_SplitShape(S)`: the shape to split; `BRepFeatSplitShape::with_shape(&s)`.
    pub fn with_shape(s: &Shape) -> Self {
        Self {
            base: BRepBuilderAPIMakeShapeBase::new(),
            s_shape: Spliter::with_shape(s),
            w_on_shape: Some(WiresOnShape::new(s)),
            right: List::default(),
        }
    }

    /// `Init(S)`: the shape to split, the wires forgotten; `split.init(&s)`.
    pub fn init(&mut self, s: &Shape) {
        self.s_shape.init(s);
        match &mut self.w_on_shape {
            Some(w) => w.init(s),
            None => self.w_on_shape = Some(WiresOnShape::new(s)),
        }
    }

    /// The wires on the shape (Standard_NullObject before a shape is given).
    fn wires(&mut self) -> Result<&mut WiresOnShape, Failure> {
        self.w_on_shape
            .as_mut()
            .ok_or_else(|| null_object("BRepFeat_SplitShape", "no shape to split"))
    }

    /// `SetCheckInterior(ToCheckInterior)`: whether the splitting edges inside the faces
    /// are checked; `split.set_check_interior(false)?`.
    pub fn set_check_interior(&mut self, to_check_interior: bool) -> Result<(), Failure> {
        self.wires()?.set_check_interior(to_check_interior);
        Ok(())
    }

    /// `Add(theEdges)`: the splitting edges bound to the faces or edges they lie on;
    /// false when one cannot be placed; `split.add_edges(&edges)?`.
    pub fn add_edges(&mut self, the_edges: &Sequence<Shape>) -> Result<bool, Failure> {
        self.wires()?.add(the_edges)
    }

    /// `Add(W, F)`: the wire `w` splits the face `f`; `split.add_wire(&w, &f)?`.
    pub fn add_wire(&mut self, w: &Wire, f: &Face) -> Result<(), Failure> {
        self.wires()?.bind_wire(w, f)
    }

    /// `Add(E, F)`: the edge `e` splits the face `f`; `split.add_edge(&e, &f)?`.
    pub fn add_edge(&mut self, e: &Edge, f: &Face) -> Result<(), Failure> {
        self.wires()?.bind_edge(e, f)
    }

    /// `Add(Comp, F)`: the edges of `comp` split the face `f`;
    /// `split.add_compound(&comp, &f)?`.
    pub fn add_compound(&mut self, comp: &Compound, f: &Face) -> Result<(), Failure> {
        self.wires()?.bind_compound(comp, f)
    }

    /// `Add(E, EOn)`: the edge `e` lies on the edge `eon` of the shape;
    /// `split.add_edge_on_edge(&e, &eon)?`.
    pub fn add_edge_on_edge(&mut self, e: &Edge, eon: &Edge) -> Result<(), Failure> {
        self.wires()?.bind_edge_on_edge(e, eon);
        Ok(())
    }

    /// `DirectLeft()`: the faces on the left of the splitting wires;
    /// `split.direct_left()?`.
    pub fn direct_left(&self) -> Result<&List<Shape>, Failure> {
        self.s_shape.direct_left()
    }

    /// `Left()`: the faces on the left, with those connected to them;
    /// `split.left()?`.
    pub fn left(&self) -> Result<&List<Shape>, Failure> {
        self.s_shape.left()
    }

    /// `Right()`: the faces of the result not on the left (computed once after a build);
    /// `split.right()?`.
    pub fn right(&mut self) -> Result<&List<Shape>, Failure> {
        if self.right.is_empty() {
            let mut a_map_of_left = MapOfShape::default();
            for f in self.s_shape.left()?.iter() {
                a_map_of_left.add(f.clone());
            }
            for a_face in Explorer::new(&self.base.my_shape, ShapeEnum::Face, ShapeEnum::Shape)? {
                if !a_map_of_left.contains(&a_face) {
                    self.right.append(a_face);
                }
            }
        }
        Ok(&self.right)
    }
}

impl BRepBuilderAPIMakeShape for BRepFeatSplitShape {
    fn make_shape_base(&self) -> &BRepBuilderAPIMakeShapeBase {
        &self.base
    }

    fn make_shape_base_mut(&mut self) -> &mut BRepBuilderAPIMakeShapeBase {
        &mut self.base
    }

    /// `Build()`: the faces split; the result is the split shape.
    fn build(&mut self, _range: &ProgressRange<'_>) -> Result<(), Failure> {
        let w = self
            .w_on_shape
            .as_mut()
            .ok_or_else(|| null_object("BRepFeat_SplitShape::Build", "no shape to split"))?;
        self.s_shape.perform(w)?;
        if self.s_shape.is_done() {
            self.base.set_done();
            self.base.my_shape = self.s_shape.resulting_shape()?.clone();
            self.right.clear();
        }
        Ok(())
    }

    /// `IsDeleted(F)`: whether the shape has no descendant.
    fn is_deleted(&mut self, f: &Shape) -> Result<bool, Failure> {
        Ok(self.s_shape.descendant_shapes(f)?.is_empty())
    }

    /// `Modified(F)`: the descendants of the shape.
    fn modified(&mut self, f: &Shape) -> Result<&List<Shape>, Failure> {
        self.base.my_generated = self.s_shape.descendant_shapes(f)?;
        Ok(&self.base.my_generated)
    }
}
