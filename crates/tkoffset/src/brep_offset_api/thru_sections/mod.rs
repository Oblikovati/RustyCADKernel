// Rust translation of ModelingAlgorithms/TKOffset/BRepOffsetAPI/BRepOffsetAPI_ThruSections.hxx/.cxx
// (the constructor, Init, the settings and Build) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepOffsetAPI_ThruSections`: a shell or a solid passing through a set of sections
//! (wires, or vertices at the ends), ruled or smoothed by an approximated surface.
mod generated;
mod indices;
mod ruled;
mod smoothed;
mod solid;
mod total_surf;

use rustycad_tkbool::brep_fill::{BRepFillGenerator, ThruSectionErrorStatus};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex, Wire};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeombase::approx::ParametrizationType;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeShape, BRepBuilderAPIMakeShapeBase};
use rustycad_tktopalgo::brep_lib::encode_regularity;

/// The indices of the faces grown from each section edge (`myEdgeNewIndices`).
type EdgeIndices = DataMap<Shape, Vec<i32>, ShapeMapHasher>;
/// The signed index of the edge starting (positive) or ending (negative) at each
/// section vertex (`myVertexIndex`).
type VertexIndices = DataMap<Shape, i32, ShapeMapHasher>;

/// `BRepOffsetAPI_ThruSections`: lofts the added sections; `Build` checks the punctual
/// sections, makes the wires compatible (unless disabled) and builds the ruled or the
/// smoothed shell, closed into a solid on request.
///
/// ```
/// use rustycad_tkmath::gp::{Axis2, Circle3, Direction3, Point3};
/// use rustycad_tkoffset::brep_offset_api::BRepOffsetAPIThruSections;
/// use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeShape, BRepBuilderAPIMakeWire};
/// let circle = |z: f64, r: f64| -> Result<_, rustycad_tkernel::standard::Failure> {
///     let axis = Axis2::new(Point3::new(0.0, 0.0, z), Direction3::new(0.0, 0.0, 1.0)?)?;
///     let edge = BRepBuilderAPIMakeEdge::from_circ(Circle3::new(axis, r)?)?.edge()?;
///     BRepBuilderAPIMakeWire::from_edge(&edge)?.wire()
/// };
/// let mut loft = BRepOffsetAPIThruSections::new(true, true, 1.0e-6);
/// loft.add_wire(&circle(0.0, 2.0)?);
/// loft.add_wire(&circle(3.0, 1.0)?);
/// assert!(!loft.shape()?.is_null());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct BRepOffsetAPIThruSections {
    base: BRepBuilderAPIMakeShapeBase,
    my_input_wires: List<Shape>,
    my_wires: Vec<Shape>,
    my_edge_new_indices: EdgeIndices,
    my_vertex_index: VertexIndices,
    my_nb_edges_in_section: i32,
    my_is_solid: bool,
    my_is_ruled: bool,
    my_w_check: bool,
    my_pres3d: f64,
    my_first: Face,
    my_last: Face,
    my_degen1: bool,
    my_degen2: bool,
    my_edge_face: DataMap<Shape, Shape, ShapeMapHasher>,
    my_continuity: Continuity,
    my_param_type: ParametrizationType,
    my_deg_max: i32,
    my_crit_weights: [f64; 3],
    my_use_smoothing: bool,
    my_mutable_input: bool,
    my_bf_generator: Option<BRepFillGenerator>,
    my_status: ThruSectionErrorStatus,
}

impl Default for BRepOffsetAPIThruSections {
    fn default() -> Self {
        Self::new(false, false, 1.0e-6)
    }
}

impl BRepBuilderAPIMakeShape for BRepOffsetAPIThruSections {
    fn make_shape_base(&self) -> &BRepBuilderAPIMakeShapeBase {
        &self.base
    }

    fn make_shape_base_mut(&mut self) -> &mut BRepBuilderAPIMakeShapeBase {
        &mut self.base
    }

    /// `Build()`: the loft; a failure of the shell construction leaves the algorithm
    /// not done (the status keeps its value, as natively).
    fn build(&mut self, _range: &ProgressRange<'_>) -> Result<(), Failure> {
        self.my_status = ThruSectionErrorStatus::Done;
        self.my_bf_generator = None;
        if wrong_punctual_sections(&self.my_wires)? {
            self.my_status = ThruSectionErrorStatus::WrongUsage;
            return Ok(());
        }
        self.my_nb_edges_in_section = 0;
        if self.my_w_check {
            if !self.make_compatible()? {
                return Ok(());
            }
        } else {
            self.index_unchecked_wires()?;
        }
        if self.create_shell().is_err() || self.my_status != ThruSectionErrorStatus::Done {
            self.base.set_not_done();
            return Ok(());
        }
        encode_regularity(&self.base.my_shape, 1.0e-10)
    }

    /// `Generated(S)`: the faces grown from a section edge, or the longitudinal edges
    /// from a section vertex.
    fn generated(&mut self, shape: &Shape) -> Result<&List<Shape>, Failure> {
        self.base.my_generated = self.generated_shapes(shape)?;
        Ok(&self.base.my_generated)
    }
}

impl BRepOffsetAPIThruSections {
    /// `BRepOffsetAPI_ThruSections(isSolid, ruled, pres3d)` (natively defaulted to a
    /// smoothed shell within 1e-6): checked compatibility, mutable input, chord-length
    /// parameters, degree 8, C2 and the criterion weights (0.4, 0.2, 0.4);
    /// `BRepOffsetAPIThruSections::new(true, false, 1e-6)`.
    pub fn new(is_solid: bool, ruled: bool, pres3d: f64) -> Self {
        Self {
            base: BRepBuilderAPIMakeShapeBase::new(),
            my_input_wires: List::default(),
            my_wires: Vec::new(),
            my_edge_new_indices: EdgeIndices::default(),
            my_vertex_index: VertexIndices::default(),
            my_nb_edges_in_section: 0,
            my_is_solid: is_solid,
            my_is_ruled: ruled,
            my_w_check: true,
            my_pres3d: pres3d,
            my_first: Face::default(),
            my_last: Face::default(),
            my_degen1: false,
            my_degen2: false,
            my_edge_face: DataMap::default(),
            my_continuity: Continuity::C2,
            my_param_type: ParametrizationType::ChordLength,
            my_deg_max: 8,
            my_crit_weights: [0.4, 0.2, 0.4],
            my_use_smoothing: false,
            my_mutable_input: true,
            my_bf_generator: None,
            my_status: ThruSectionErrorStatus::NotDone,
        }
    }

    /// `Init(isSolid, ruled, pres3d)`: the settings again (the sections are kept, and
    /// the degree becomes 6, not the 8 of the constructor, as natively);
    /// `loft.init(true, false, 1e-6)`.
    pub fn init(&mut self, is_solid: bool, ruled: bool, pres3d: f64) {
        self.my_is_solid = is_solid;
        self.my_is_ruled = ruled;
        self.my_pres3d = pres3d;
        self.my_w_check = true;
        self.my_mutable_input = true;
        self.my_param_type = ParametrizationType::ChordLength;
        self.my_deg_max = 6;
        self.my_continuity = Continuity::C2;
        self.my_crit_weights = [0.4, 0.2, 0.4];
        self.my_use_smoothing = false;
        self.my_status = ThruSectionErrorStatus::NotDone;
    }

    /// `AddWire(wire)`: one more section; `loft.add_wire(&wire)`.
    pub fn add_wire(&mut self, wire: &Wire) {
        self.my_wires.push(wire.as_ref().clone());
        self.my_input_wires.append(wire.as_ref().clone());
    }

    /// `AddVertex(aVertex)`: a punctual section, a closed wire of one degenerated
    /// edge on the vertex; `loft.add_vertex(&vertex)?`.
    pub fn add_vertex(&mut self, vertex: &Vertex) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        let mut deg_edge = Edge::default();
        builder.make_edge(&mut deg_edge)?;
        builder.add(&mut deg_edge, &vertex.oriented(Orientation::Forward))?;
        builder.add(&mut deg_edge, &vertex.oriented(Orientation::Reversed))?;
        builder.degenerated(&deg_edge, true)?;
        let mut deg_wire = Wire::default();
        builder.make_wire(&mut deg_wire);
        builder.add(&mut deg_wire, deg_edge.as_ref())?;
        deg_wire.set_closed(true)?;
        self.my_wires.push(deg_wire.as_ref().clone());
        self.my_input_wires.append(deg_wire.into_shape());
        Ok(())
    }

    /// `CheckCompatibility(check)` (natively defaulted to true): whether `Build` makes
    /// the wires compatible first; `loft.check_compatibility(false)`.
    pub fn check_compatibility(&mut self, check: bool) {
        self.my_w_check = check;
    }

    /// `SetSmoothing(UseSmoothing)`: the smoothing approximation; `loft.set_smoothing(true)`.
    pub fn set_smoothing(&mut self, use_smoothing: bool) {
        self.my_use_smoothing = use_smoothing;
    }

    /// `SetParType(ParType)`; `loft.set_par_type(ParametrizationType::Centripetal)`.
    pub fn set_par_type(&mut self, par_type: ParametrizationType) {
        self.my_param_type = par_type;
    }

    /// `SetContinuity(C)`; `loft.set_continuity(Continuity::C1)`.
    pub fn set_continuity(&mut self, continuity: Continuity) {
        self.my_continuity = continuity;
    }

    /// `SetCriteriumWeight(W1, W2, W3)`: the smoothing weights; a negative one sets
    /// the Failed status and keeps the weights; `loft.set_criterium_weight(0.4, 0.2, 0.4)`.
    pub fn set_criterium_weight(&mut self, w1: f64, w2: f64, w3: f64) {
        if w1 < 0. || w2 < 0. || w3 < 0. {
            self.my_status = ThruSectionErrorStatus::Failed;
            return;
        }
        self.my_crit_weights = [w1, w2, w3];
    }

    /// `SetMaxDegree(MaxDeg)`; `loft.set_max_degree(6)`.
    pub fn set_max_degree(&mut self, max_deg: i32) {
        self.my_deg_max = max_deg;
    }

    /// `ParType()`; `loft.par_type()`.
    pub fn par_type(&self) -> ParametrizationType {
        self.my_param_type
    }

    /// `Continuity()`; `loft.continuity()`.
    pub fn continuity(&self) -> Continuity {
        self.my_continuity
    }

    /// `MaxDegree()`; `loft.max_degree()`.
    pub fn max_degree(&self) -> i32 {
        self.my_deg_max
    }

    /// `UseSmoothing()`; `loft.use_smoothing()`.
    pub fn use_smoothing(&self) -> bool {
        self.my_use_smoothing
    }

    /// `CriteriumWeight(W1, W2, W3)`; `let (w1, w2, w3) = loft.criterium_weight();`.
    pub fn criterium_weight(&self) -> (f64, f64, f64) {
        let [w1, w2, w3] = self.my_crit_weights;
        (w1, w2, w3)
    }

    /// `FirstShape()`: the face closing the first section of a solid; `loft.first_shape()`.
    pub fn first_shape(&self) -> &Face {
        &self.my_first
    }

    /// `LastShape()`: the face closing the last section of a solid; `loft.last_shape()`.
    pub fn last_shape(&self) -> &Face {
        &self.my_last
    }

    /// `GeneratedFace(Edge)`: the face grown from a section edge (a null shape for an
    /// unknown edge); `loft.generated_face(&edge)`.
    pub fn generated_face(&self, edge: &Shape) -> Shape {
        self.my_edge_face.find(edge).cloned().unwrap_or_default()
    }

    /// `SetMutableInput(theIsMutableInput)`: whether the input sections may be
    /// modified; `loft.set_mutable_input(false)`.
    pub fn set_mutable_input(&mut self, is_mutable_input: bool) {
        self.my_mutable_input = is_mutable_input;
    }

    /// `IsMutableInput()`; `loft.is_mutable_input()`.
    pub fn is_mutable_input(&self) -> bool {
        self.my_mutable_input
    }

    /// `Wires()`: the input sections; `loft.wires()`.
    pub fn wires(&self) -> &List<Shape> {
        &self.my_input_wires
    }

    /// `GetStatus()`; `loft.get_status()`.
    pub fn get_status(&self) -> ThruSectionErrorStatus {
        self.my_status
    }

    /// The `try` block of `Build`: a ruled shell for two sections or on request, a
    /// smoothed one otherwise.
    fn create_shell(&mut self) -> Result<(), Failure> {
        if self.my_wires.len() == 2 || self.my_is_ruled {
            self.create_ruled()
        } else {
            self.create_smoothed()
        }
    }
}

/// Whether an inner section is punctual, or every section of at most two is.
fn wrong_punctual_sections(wires: &[Shape]) -> Result<bool, Failure> {
    let count = wires.len();
    for wire in wires.iter().take(count.saturating_sub(1)).skip(1) {
        if is_punctual(wire)? {
            return Ok(true);
        }
    }
    if count > 2 {
        return Ok(false);
    }
    for wire in wires {
        if !is_punctual(wire)? {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Whether every edge of the shape is degenerated (true without edge).
fn is_punctual(shape: &Shape) -> Result<bool, Failure> {
    let mut explorer = Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)?;
    while explorer.more() {
        if !BRepTool::degenerated(&topods::edge(explorer.current())?)? {
            return Ok(false);
        }
        explorer.next()?;
    }
    Ok(true)
}
