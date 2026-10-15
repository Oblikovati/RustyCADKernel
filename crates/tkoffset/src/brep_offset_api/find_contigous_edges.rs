// Rust translation of ModelingAlgorithms/TKOffset/BRepOffsetAPI/BRepOffsetAPI_FindContigousEdges.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepOffsetAPI_FindContigousEdges`: the contiguous edges of shapes, found by a
//! `BRepBuilderAPI_Sewing` that analyses without sewing.
use rustycad_tkbrep::topods::{Edge, Shape};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPISewing;
use std::io::Write;

/// `BRepOffsetAPI_FindContigousEdges`: adds shapes, then `Perform` finds the pairs of
/// edges within the tolerance.
///
/// ```
/// use rustycad_tkoffset::brep_offset_api::BRepOffsetAPIFindContigousEdges;
/// let finder = BRepOffsetAPIFindContigousEdges::new(1.0e-6, true);
/// assert_eq!(finder.nb_contigous_edges(), 0);
/// ```
pub struct BRepOffsetAPIFindContigousEdges {
    my_sewing: BRepBuilderAPISewing,
}

impl Default for BRepOffsetAPIFindContigousEdges {
    fn default() -> Self {
        Self::new(1.0e-06, true)
    }
}

impl BRepOffsetAPIFindContigousEdges {
    /// `BRepOffsetAPI_FindContigousEdges(tolerance, option)` (natively defaulted to
    /// 1e-6 and the analysis of degenerated shapes);
    /// `BRepOffsetAPIFindContigousEdges::new(1e-6, true)`.
    pub fn new(tolerance: f64, option: bool) -> Self {
        let mut finder = Self {
            my_sewing: BRepBuilderAPISewing::default(),
        };
        finder.init(tolerance, option);
        finder
    }

    /// `Init(tolerance, option)`: a sewing that analyses (with `option`) and cuts but
    /// does not sew; `finder.init(1e-6, true)`.
    pub fn init(&mut self, tolerance: f64, option: bool) {
        self.my_sewing.init(tolerance, option, false, true, false);
    }

    /// `Add(shape)`: a shape to analyse; `finder.add(&shape)?`.
    pub fn add(&mut self, shape: &Shape) -> Result<(), Failure> {
        self.my_sewing.add(shape)
    }

    /// `Perform()`: the analysis; `finder.perform()?`.
    pub fn perform(&mut self) -> Result<(), Failure> {
        self.my_sewing.perform(&ProgressRange::default())
    }

    /// `NbContigousEdges()`; `finder.nb_contigous_edges()`.
    pub fn nb_contigous_edges(&self) -> i32 {
        self.my_sewing.nb_contigous_edges()
    }

    /// `ContigousEdge(index)`: Standard_OutOfRange outside `0..=NbContigousEdges()`;
    /// `finder.contigous_edge(1)?`.
    pub fn contigous_edge(&self, index: i32) -> Result<Edge, Failure> {
        self.check_index(index, self.nb_contigous_edges(), "ContigousEdge")?;
        self.my_sewing.contigous_edge(index)
    }

    /// `ContigousEdgeCouple(index)`: the edges merged into a contiguous edge;
    /// `finder.contigous_edge_couple(1)?`.
    pub fn contigous_edge_couple(&self, index: i32) -> Result<&List<Shape>, Failure> {
        self.check_index(index, self.nb_contigous_edges(), "ContigousEdgeCouple")?;
        self.my_sewing.contigous_edge_couple(index)
    }

    /// `SectionToBoundary(section)`: the boundary edge of a section; Standard_NoSuchObject
    /// for an unbound section; `finder.section_to_boundary(&edge)?`.
    pub fn section_to_boundary(&self, section: &Edge) -> Result<Edge, Failure> {
        if !self.my_sewing.is_section_bound(section) {
            return Err(Failure::with_message(
                "Standard_NoSuchObject",
                "BRepOffsetAPI_FindContigousEdges::SectionToBoundary",
            ));
        }
        self.my_sewing.section_to_boundary(section)
    }

    /// `NbDegeneratedShapes()`; `finder.nb_degenerated_shapes()`.
    pub fn nb_degenerated_shapes(&self) -> i32 {
        self.my_sewing.nb_degenerated_shapes()
    }

    /// `DegeneratedShape(index)`: Standard_OutOfRange outside
    /// `0..=NbDegeneratedShapes()`; `finder.degenerated_shape(1)?`.
    pub fn degenerated_shape(&self, index: i32) -> Result<&Shape, Failure> {
        self.check_index(index, self.nb_degenerated_shapes(), "DegereratedShape")?;
        self.my_sewing.degenerated_shape(index)
    }

    /// `IsDegenerated(shape)`; `finder.is_degenerated(&shape)?`.
    pub fn is_degenerated(&self, shape: &Shape) -> Result<bool, Failure> {
        self.my_sewing.is_degenerated(shape)
    }

    /// `IsModified(shape)`; `finder.is_modified(&shape)`.
    pub fn is_modified(&self, shape: &Shape) -> bool {
        self.my_sewing.is_modified(shape)
    }

    /// `Modified(shape)`: Standard_NoSuchObject for an unmodified shape;
    /// `finder.modified(&shape)?`.
    pub fn modified(&self, shape: &Shape) -> Result<Shape, Failure> {
        if !self.is_modified(shape) {
            return Err(Failure::with_message(
                "Standard_NoSuchObject",
                "BRepOffsetAPI_FindContigousEdges::Modified",
            ));
        }
        Ok(self.my_sewing.modified(shape))
    }

    /// `Dump()`: the counts of the analysis, written to `out` (std::cout natively);
    /// `finder.dump(&mut std::io::stdout())?`.
    pub fn dump(&self, out: &mut dyn Write) -> Result<(), Failure> {
        self.my_sewing.dump(out)
    }

    /// The native `Standard_OutOfRange_Raise_if(index < 0 || index > count, name)`.
    fn check_index(&self, index: i32, count: i32, name: &str) -> Result<(), Failure> {
        if index < 0 || index > count {
            return Err(Failure::with_message(
                "Standard_OutOfRange",
                &format!("BRepOffsetAPI_FindContigousEdges::{name}"),
            ));
        }
        Ok(())
    }
}
