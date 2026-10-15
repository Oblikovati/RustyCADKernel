// Rust translation of ModelingAlgorithms/TKFeat/BRepFeat/BRepFeat_Builder.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepFeat_Builder`: the local Boolean operation of the features. A Boolean operation
//! of the basis and the tool (its `Prepare`, `FillIn3DParts` and
//! `CheckArgsForOpenSolid` overridden) gives the parts of the tool; the result is then
//! rebuilt with the kept parts only.
mod rebuild;

use crate::errors::no_such_object;
use rustycad_tkbo::bop_algo::{
    BOPAlgoBOP, BOPAlgoBuilder, BOPAlgoPaveFiller, BopHooks, BuilderHooks, Operation,
    SameDomainShapes, build_bop_shape, fill_images_solids_with, perform_bop_internal1,
};
use rustycad_tkbo::bop_ds::DataStructure;
use rustycad_tkbo::bop_tools::ShapeSet;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{Shape, ShapeIterator};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::ncollection::{List, Map};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// `BRepFeat_Builder`: the local Boolean operation of a basis shape and a tool.
///
/// ```
/// use rustycad_tkfeat::brep_feat::BRepFeatBuilder;
/// let builder = BRepFeatBuilder::new();
/// assert!(builder.shape().is_null());
/// ```
pub struct BRepFeatBuilder {
    bop: BOPAlgoBOP,
    shapes: MapOfShape,
    removed: MapOfShape,
    fuse: i32,
}

impl Default for BRepFeatBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl BRepFeatBuilder {
    /// `BRepFeat_Builder()`: cleared; `BRepFeatBuilder::new()`.
    pub fn new() -> Self {
        let mut builder = Self {
            bop: BOPAlgoBOP::new(),
            shapes: MapOfShape::default(),
            removed: MapOfShape::default(),
            fuse: 0,
        };
        builder.clear();
        builder
    }

    /// `Clear()`: the kept and removed parts, then the Boolean operation;
    /// `builder.clear()`.
    pub fn clear(&mut self) {
        self.shapes.clear(false);
        self.removed.clear(false);
        self.bop.clear();
    }

    /// `Init(theShape)`: the basis alone; `builder.init(&shape)`.
    pub fn init(&mut self, the_shape: &Shape) {
        self.clear();
        self.bop.add_argument(the_shape);
    }

    /// `Init(theShape, theTool)`: the basis and the tool; `builder.init_with_tool(&s, &t)`.
    pub fn init_with_tool(&mut self, the_shape: &Shape, the_tool: &Shape) {
        self.clear();
        self.bop.add_argument(the_shape);
        self.bop.add_tool(the_tool);
    }

    /// `AddTool(theShape)` (`BOPAlgo_ToolsProvider`): one more tool;
    /// `builder.add_tool(&tool)`.
    pub fn add_tool(&mut self, the_shape: &Shape) {
        self.bop.add_tool(the_shape);
    }

    /// The arguments of the Boolean operation (`myArguments`, the basis first);
    /// `builder.arguments()`.
    pub fn arguments(&self) -> &List<Shape> {
        self.bop.arguments()
    }

    /// `SetOperation(theFuse)`: FUSE when `the_fuse` is not 0, else CUT;
    /// `builder.set_operation(1)`.
    pub fn set_operation(&mut self, the_fuse: i32) {
        self.fuse = the_fuse;
        self.bop.set_operation(if self.fuse != 0 {
            Operation::Fuse
        } else {
            Operation::Cut
        });
    }

    /// `SetOperation(theFuse, theFlag)`: with `the_flag` (no second part of the tool
    /// selected), CUT21 for a fuse and COMMON for a cut, else as
    /// [`Self::set_operation`]; `builder.set_operation_with_flag(1, true)`.
    pub fn set_operation_with_flag(&mut self, the_fuse: i32, the_flag: bool) {
        self.fuse = the_fuse;
        let operation = match (the_flag, self.fuse != 0) {
            (false, true) => Operation::Fuse,
            (false, false) => Operation::Cut,
            (true, true) => Operation::Cut21,
            (true, false) => Operation::Common,
        };
        self.bop.set_operation(operation);
    }

    /// `Perform()`: the Boolean operation with the overridden members;
    /// `builder.perform()`.
    pub fn perform(&mut self) {
        BOPAlgoBOP::perform_hooks(self);
    }

    /// `Shape()`: the result; `builder.shape()`.
    pub fn shape(&self) -> &Shape {
        self.bop.shape()
    }

    /// `HasErrors()`; `builder.has_errors()`.
    pub fn has_errors(&self) -> bool {
        self.bop.options().has_errors()
    }

    /// `Modified(S)`: the history of the Boolean operation; `builder.modified(&s)?`.
    pub fn modified(&self, s: &Shape) -> Result<Vec<Shape>, Failure> {
        self.bop.modified(s)
    }

    /// The Boolean operation (`BOPAlgo_BOP` base); `builder.boolean_operation()`.
    pub fn boolean_operation(&self) -> &BOPAlgoBOP {
        &self.bop
    }

    /// `PartsOfTool(theLT)`: the solids of the result; `builder.parts_of_tool(&mut l)?`.
    pub fn parts_of_tool(&self, the_lt: &mut List<Shape>) -> Result<(), Failure> {
        the_lt.clear();
        for s in Explorer::new(self.bop.shape(), ShapeEnum::Solid, ShapeEnum::Shape)? {
            the_lt.append(s);
        }
        Ok(())
    }

    /// `KeepParts(theIm)`: keeps each part; `builder.keep_parts(&parts)?`.
    pub fn keep_parts(&mut self, the_im: &List<Shape>) -> Result<(), Failure> {
        for s in the_im.iter() {
            self.keep_part(s)?;
        }
        Ok(())
    }

    /// `KeepPart(theS)`: the part and its sub-shapes are kept; `builder.keep_part(&s)?`.
    pub fn keep_part(&mut self, the_part: &Shape) -> Result<(), Failure> {
        topexp::map_shapes_into_map(the_part, &mut self.shapes, true, true);
        Ok(())
    }

    /// The first argument (the basis) and the first tool.
    fn basis_and_tool(&self) -> Result<(Shape, Shape), Failure> {
        let basis = self
            .bop
            .arguments()
            .first()
            .cloned()
            .ok_or_else(|| no_such_object("BRepFeat_Builder", "no basis shape"))?;
        let tool = self
            .bop
            .tools()
            .first()
            .cloned()
            .ok_or_else(|| no_such_object("BRepFeat_Builder", "no tool"))?;
        Ok((basis, tool))
    }

    /// `FillRemoved()`: the images of the basis solids are forgotten, and the images of
    /// the tool not kept (with their sub-shapes) are removed; `builder.fill_removed()?`.
    pub fn fill_removed(&mut self) -> Result<(), Failure> {
        let (args0, args1) = self.basis_and_tool()?;
        for s in Explorer::new(&args0, ShapeEnum::Solid, ShapeEnum::Shape)? {
            self.bop.images_mut().unbind(&s);
        }
        let Some(images) = self.bop.images().seek(&args1) else {
            return Ok(());
        };
        let images: Vec<Shape> = images.iter().cloned().collect();
        let mut removed = std::mem::take(&mut self.removed);
        for s in &images {
            self.fill_removed_of(s, &mut removed);
        }
        self.removed = removed;
        Ok(())
    }

    /// `FillRemoved(theS, theM)`: `s` and its sub-shapes not kept are added to `m`;
    /// `builder.fill_removed_of(&s, &mut m)`.
    pub fn fill_removed_of(&self, s: &Shape, m: &mut MapOfShape) {
        if self.shapes.contains(s) {
            return;
        }
        m.add(s.clone());
        for child in ShapeIterator::new(s, true, true) {
            self.fill_removed_of(&child, m);
        }
    }

    /// `CheckSolidImages()`: the images of the basis solids equal to an image of the
    /// tool are replaced by it (same domain); `builder.check_solid_images()?`.
    pub fn check_solid_images(&mut self) -> Result<(), Failure> {
        let (args0, args1) = self.basis_and_tool()?;
        let mut mst: Map<ShapeSet> = Map::default();
        let tool_images = self.bop.images().find(&args1)?.clone();
        for sol_im in tool_images.iter() {
            let mut st = ShapeSet::new();
            st.add(sol_im, ShapeEnum::Face)?;
            mst.add(st);
        }
        // Native quirk kept: the new list grows over the solids of the basis.
        let mut ls_im_new = List::default();
        for solid in Explorer::new(&args0, ShapeEnum::Solid, ShapeEnum::Shape)? {
            let Some(images) = self.bop.images().seek(&solid).cloned() else {
                continue;
            };
            for sol_im in images.iter() {
                let mut st = ShapeSet::new();
                st.add(sol_im, ShapeEnum::Face)?;
                let flag_sd = mst.contains(&st);
                let sx = mst.added(st).shape().clone();
                ls_im_new.append(sx.clone());
                if flag_sd {
                    self.bop.shapes_sd_mut().bind(sol_im.clone(), sx);
                }
            }
            if let Some(l) = self.bop.images_mut().change_seek(&solid) {
                *l = ls_im_new.clone();
            }
        }
        Ok(())
    }

    /// `PerformResult()`: without kept part the Boolean result; else the result rebuilt
    /// with the kept parts of the tool: faces, shells, solids (the basis solids equal to
    /// tool parts merged), compounds and the shape; `builder.perform_result()?`.
    pub fn perform_result(&mut self) -> Result<(), Failure> {
        self.bop.set_operation(if self.fuse != 0 {
            Operation::Fuse
        } else {
            Operation::Cut
        });
        let mut filler = self.bop.take_pave_filler();
        let result = match filler.as_mut().and_then(|f| f.data_structure_mut()) {
            Some(structure) => self.perform_result_on(structure),
            None => Err(no_such_object(
                "BRepFeat_Builder::PerformResult",
                "no data structure; expected a performed builder",
            )),
        };
        self.bop.restore_pave_filler(filler);
        result
    }

    fn perform_result_on(&mut self, structure: &mut DataStructure) -> Result<(), Failure> {
        if self.shapes.is_empty() {
            return build_bop_shape(self, structure);
        }
        BuilderHooks::prepare(self)?;
        self.rebuild_faces(structure)?;
        self.bop
            .fill_images_containers(ShapeEnum::Shell, structure)?;
        if self.has_errors() {
            return Ok(());
        }
        fill_images_solids_with(self, structure)?;
        if self.has_errors() {
            return Ok(());
        }
        self.check_solid_images()?;
        self.bop
            .build_result_of_arguments(ShapeEnum::Solid, structure)?;
        if self.has_errors() {
            return Ok(());
        }
        self.bop.fill_images_compounds(structure)?;
        if self.has_errors() {
            return Ok(());
        }
        self.bop
            .build_result_of_arguments(ShapeEnum::Compound, structure)?;
        if self.has_errors() {
            return Ok(());
        }
        build_bop_shape(self, structure)
    }
}

impl BuilderHooks for BRepFeatBuilder {
    fn general_fuse(&mut self) -> &mut BOPAlgoBuilder {
        &mut self.bop
    }

    fn build_result(&mut self, kind: ShapeEnum, structure: &DataStructure) -> Result<(), Failure> {
        self.bop.build_result_of_arguments(kind, structure)
    }

    /// `Prepare()`: the report cleared, an empty compound, then `FillRemoved`.
    fn prepare(&mut self) -> Result<(), Failure> {
        self.bop.options().clear();
        BOPAlgoBuilder::prepare(&mut self.bop);
        self.fill_removed()
    }

    /// `FillIn3DParts(theDraftSolids)`: the report cleared, the base classification,
    /// then the removed faces taken out of the IN parts of the solids.
    fn fill_in_3d_parts(
        &mut self,
        structure: &mut DataStructure,
        draft_solids: &mut SameDomainShapes,
    ) -> Result<(), Failure> {
        self.bop.options().clear();
        BOPAlgoBuilder::fill_in_3d_parts(&mut self.bop, structure, draft_solids)?;
        // Clear the IN parts of the solids from the removed faces
        let removed = &self.removed;
        let in_parts = self.bop.in_parts_mut();
        for (_, list) in in_parts.items_mut() {
            list.retain(|s| !removed.contains(s));
        }
        Ok(())
    }

    fn perform_internal1(&mut self, filler: &mut BOPAlgoPaveFiller) -> Result<(), Failure> {
        perform_bop_internal1(self, filler)
    }
}

impl BopHooks for BRepFeatBuilder {
    fn boolean(&mut self) -> &mut BOPAlgoBOP {
        &mut self.bop
    }

    /// `CheckArgsForOpenSolid()`: false, so that the splits of the solids always make the
    /// result.
    fn check_args_for_open_solid(&mut self, _structure: &DataStructure) -> Result<bool, Failure> {
        Ok(false)
    }
}
