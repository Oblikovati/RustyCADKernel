//! `BOPAlgo_WireSplitter`: splits the edges of a [`WireEdgeSet`] into closed wires.
use super::alerts::alert;
use super::{AlertNullInputShapes, Options, WireEdgeSet};
use crate::bop_tools::{ConnexityBlock, make_connexity_blocks_from_shapes};
use crate::int_tools::Context;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{Shape, Wire};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;

#[path = "wire_splitter/angles.rs"]
mod angles;
#[path = "wire_splitter/edge_info.rs"]
mod edge_info;
#[path = "wire_splitter/path.rs"]
mod path;
#[path = "wire_splitter/split_block.rs"]
mod split_block;
pub use edge_info::EdgeInfo;

/// Builds the wires of a face from its split edges: regular connexity blocks become one
/// wire each, the others are split by the smallest clockwise angle at their vertices.
///
/// ```
/// use rustycad_tkbo::bop_algo::{WireEdgeSet, WireSplitter};
/// let mut splitter = WireSplitter::new();
/// splitter.set_wes(WireEdgeSet::new());
/// splitter.perform().unwrap();
/// assert!(splitter.wes().unwrap().shapes().is_empty());
/// ```
#[derive(Default)]
pub struct WireSplitter {
    options: Options,
    wes: Option<WireEdgeSet>,
    blocks: Vec<ConnexityBlock>,
    context: Handle<Context>,
}

impl WireSplitter {
    /// A splitter with no edge set (`BOPAlgo_WireSplitter()`); `WireSplitter::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Hands the edge set to split; the built wires are added to it (`SetWES`).
    /// `splitter.set_wes(set)`.
    pub fn set_wes(&mut self, wes: WireEdgeSet) {
        self.wes = Some(wes);
    }

    /// The edge set, with the built wires after `perform` (`WES`); `splitter.wes()`.
    pub fn wes(&self) -> Option<&WireEdgeSet> {
        self.wes.as_ref()
    }

    /// Takes the edge set back out of the splitter; `splitter.take_wes()`.
    pub fn take_wes(&mut self) -> Option<WireEdgeSet> {
        self.wes.take()
    }

    /// Shares a geometric context (`SetContext`); `splitter.set_context(context)`.
    pub fn set_context(&mut self, context: Handle<Context>) {
        self.context = context;
    }

    /// The geometric context, created by `perform` when none was set (`Context`).
    /// `splitter.context()`.
    pub fn context(&self) -> &Handle<Context> {
        &self.context
    }

    /// The report and run options; `splitter.options().has_errors()`.
    pub fn options(&self) -> &Options {
        &self.options
    }

    /// Mutable report and run options; `splitter.options_mut().set_run_parallel(true)`.
    pub fn options_mut(&mut self) -> &mut Options {
        &mut self.options
    }

    /// Splits the start edges of the set into wires (`Perform`); `splitter.perform()?`.
    pub fn perform(&mut self) -> Result<(), Failure> {
        self.options.clear();
        self.check_data();
        if self.options.has_errors() {
            return Ok(());
        }
        if self.context.is_null() {
            self.context = Handle::new(Context::default());
        }
        let start: Vec<Shape> = match &self.wes {
            Some(wes) => wes.start_elements().iter().cloned().collect(),
            None => return Ok(()),
        };
        self.blocks.clear();
        make_connexity_blocks_from_shapes(
            &start,
            ShapeEnum::Vertex,
            ShapeEnum::Edge,
            &mut self.blocks,
        )?;
        self.make_wires()
    }

    /// Reports `AlertNullInputShapes` when no edge set was given (`CheckData`).
    fn check_data(&mut self) {
        if self.wes.is_none() {
            self.options.add_error(&alert(AlertNullInputShapes));
        }
    }

    /// Turns each connexity block into wires (`MakeWires`): a regular block is one wire, the
    /// others go through [`WireSplitter::split_block`].
    fn make_wires(&mut self) -> Result<(), Failure> {
        let Some(wes) = self.wes.as_mut() else {
            return Ok(());
        };
        let face = wes.face().clone();
        let mut irregular: Vec<ConnexityBlock> = Vec::new();
        for block in self.blocks.iter_mut() {
            if block.is_regular() {
                let wire = Self::make_wire(block.shapes())?;
                wes.add_shape(&wire.into_shape());
            } else {
                irregular.push(block.clone());
            }
        }
        for block in irregular.iter_mut() {
            Self::split_block(&face, block, &self.context)?;
        }
        for block in &irregular {
            for wire in block.loops().iter() {
                wes.add_shape(wire);
            }
        }
        Ok(())
    }

    /// A wire made of `edges`, flagged closed when it has no free vertex (`MakeWire`).
    /// `WireSplitter::make_wire(&edges)?`.
    pub fn make_wire(edges: &List<Shape>) -> Result<Wire, Failure> {
        let builder = BRepBuilder::default();
        let mut wire = Wire::default();
        builder.make_wire(&mut wire);
        let mut shape = wire.into_shape();
        for edge in edges.iter() {
            builder.add(&mut shape, edge)?;
        }
        shape.set_closed(BRepTool::is_closed(&shape)?)?;
        Ok(Wire::unchecked(shape))
    }
}
