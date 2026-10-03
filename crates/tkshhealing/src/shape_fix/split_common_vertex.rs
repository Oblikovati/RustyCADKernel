// Rust translation of ShapeFix/ShapeFix_SplitCommonVertex.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Split exactly equal vertices shared by distinct face wires.
use crate::shape_analysis::ShapeAnalysisEdge;
use crate::shape_build::{ShapeBuildEdge, ShapeBuildReShape};
use crate::shape_extend::ShapeExtendWireData;
use crate::shape_fix::{FixRoot, FixRootData, ShapeFixRoot};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Face, Shape, ShapeIterator, Vertex, Wire};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::message::Msg;
use rustycad_tkernel::standard::{Failure, Handle, SharedState};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;

/// `ShapeFix_SplitCommonVertex`: separates orientation-equal shared vertices in face wires.
///
/// ```
/// use rustycad_tkbrep::brep::BRepBuilder;
/// use rustycad_tkbrep::topods::Face;
/// use rustycad_tkshhealing::shape_fix::ShapeFixSplitCommonVertex;
/// let mut face = Face::default();
/// BRepBuilder::default().make_face(&mut face);
/// let fixer = ShapeFixSplitCommonVertex::new();
/// fixer.init(&face.clone().into_shape())?;
/// fixer.perform()?;
/// assert!(fixer.shape()?.is_same(&face.clone().into_shape()));
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Default)]
pub struct ShapeFixSplitCommonVertex {
    parent: ShapeFixRoot,
    shape: SharedState<Shape>,
}
impl_transient!(ShapeFixSplitCommonVertex, "ShapeFix_SplitCommonVertex", ShapeFixRoot, parent, interfaces: [dyn FixRoot]);

impl ShapeFixSplitCommonVertex {
    /// `ShapeFix_SplitCommonVertex()`: initializes the root precision and an empty shape;
    /// `ShapeFixSplitCommonVertex::new()`.
    pub fn new() -> Self {
        let fixer = Self::default();
        fixer.set_precision(rustycad_tkernel::precision::confusion());
        fixer
    }

    /// `Init(shape)`: retains the input while ensuring a context exists; `fixer.init(&shape)?`.
    pub fn init(&self, shape: &Shape) -> Result<(), Failure> {
        *self.shape.write() = shape.clone();
        let context = self.context();
        if context.is_null() {
            self.set_context(&Handle::new(ShapeBuildReShape::new()));
        }
        let context = self.context();
        let _discarded_apply_result = context.apply(shape, ShapeEnum::Shape)?;
        Ok(())
    }

    /// `Perform()`: replaces matching vertices on the later wire of each face;
    /// `fixer.perform()?`.
    pub fn perform(&self) -> Result<(), Failure> {
        let original = self.shape.read().clone();
        if original.shape_type()? > ShapeEnum::Face {
            return Ok(());
        }
        self.perform_faces(&original)?;
        let result = self.context().apply(&original, ShapeEnum::Shape)?;
        *self.shape.write() = result;
        Ok(())
    }

    /// `Shape()`: the retained input after recorded replacements are applied;
    /// `let result = fixer.shape()?`.
    pub fn shape(&self) -> Result<Shape, Failure> {
        Ok(self.shape.read().clone())
    }

    fn perform_faces(&self, original: &Shape) -> Result<(), Failure> {
        let faces = Explorer::new(original, ShapeEnum::Face, ShapeEnum::Shape)?;
        for face in faces {
            self.split_face_wires(&face)?;
        }
        Ok(())
    }

    fn split_face_wires(&self, input: &Shape) -> Result<(), Failure> {
        let changed_face = self.context().apply(input, ShapeEnum::Shape)?;
        let face = topods::face(&changed_face)?;
        if face.is_null() {
            return Ok(());
        }
        let wires = direct_wires(&face)?;
        if wires.len() < 2 {
            return Ok(());
        }
        let mut replacements = Vec::new();
        for first in 0..wires.len() - 1 {
            let _first_wire_data = ShapeExtendWireData::from_wire(&wires[first], true, true)?;
            for second in first + 1..wires.len() {
                self.compare_wire_pair(&wires[first], &wires[second], &mut replacements)?;
            }
        }
        self.warn_if_replacements_exist(!replacements.is_empty());
        Ok(())
    }

    fn warn_if_replacements_exist(&self, found: bool) {
        if !found {
            return;
        }
        let message = Msg::from_key(&"Fix.SplitCommonVertex.MSG0".into());
        self.send_warning_on_shape(&message);
    }

    fn compare_wire_pair(
        &self,
        first: &Wire,
        second: &Wire,
        replacements: &mut Vec<(Shape, Vertex)>,
    ) -> Result<(), Failure> {
        let second_wire_data = ShapeExtendWireData::from_wire(second, true, true)?;
        let first_vertices = vertices(&first.clone().into_shape())?;
        let second_vertices = vertices(&second.clone().into_shape())?;
        for left in &first_vertices {
            for right in &second_vertices {
                if left.is_equal(right) {
                    let replacement = replacement_for(right, replacements)?;
                    replace_edge_vertices(self, &second_wire_data, right, &replacement)?;
                }
            }
        }
        Ok(())
    }
}

impl FixRoot for ShapeFixSplitCommonVertex {
    fn fix_root_data(&self) -> &FixRootData {
        self.parent.fix_root_data()
    }
}

fn direct_wires(face: &Face) -> Result<Vec<Wire>, Failure> {
    let children = ShapeIterator::new(&face.clone().into_shape(), false, true);
    let mut wires = Vec::new();
    for child in children {
        if child.shape_type()? == ShapeEnum::Wire {
            wires.push(topods::wire(&child)?);
        }
    }
    Ok(wires)
}

fn vertices(shape: &Shape) -> Result<Vec<Vertex>, Failure> {
    Explorer::new(shape, ShapeEnum::Vertex, ShapeEnum::Shape)?
        .map(|vertex| topods::vertex(&vertex))
        .collect()
}

fn replacement_for(
    original: &Vertex,
    replacements: &mut Vec<(Shape, Vertex)>,
) -> Result<Vertex, Failure> {
    if let Some((_, replacement)) = replacements.iter().find(|(key, _)| key.is_same(original)) {
        return Ok(replacement.clone());
    }
    let point: Point3 = BRepTool::pnt(original)?;
    let tolerance = BRepTool::vertex_tolerance(original)?;
    let mut replacement = Vertex::default();
    BRepBuilder::default().make_vertex_at(&mut replacement, point, tolerance)?;
    replacements.push((original.clone().into_shape(), replacement.clone()));
    Ok(replacement)
}

fn replace_edge_vertices(
    fixer: &ShapeFixSplitCommonVertex,
    wire_data: &ShapeExtendWireData,
    shared: &Vertex,
    replacement: &Vertex,
) -> Result<(), Failure> {
    let analysis = ShapeAnalysisEdge::new();
    for index in 1..=wire_data.nb_edges() {
        let edge = wire_data.edge(index)?;
        replace_matching_edge(fixer, &edge, shared, replacement, &analysis)?;
    }
    Ok(())
}

fn replace_matching_edge(
    fixer: &ShapeFixSplitCommonVertex,
    edge: &rustycad_tkbrep::topods::Edge,
    shared: &Vertex,
    replacement: &Vertex,
    analysis: &ShapeAnalysisEdge,
) -> Result<(), Failure> {
    let (mut first, mut last) = (analysis.first_vertex(edge), analysis.last_vertex(edge));
    let first_matches = first.is_equal(shared);
    let last_matches = last.is_equal(shared);
    if !first_matches && !last_matches {
        return Ok(());
    }
    if first_matches {
        first = replacement.clone();
    }
    if last_matches {
        last = replacement.clone();
    }
    replace_edge(fixer, edge, &first, &last)
}

fn replace_edge(
    fixer: &ShapeFixSplitCommonVertex,
    edge: &rustycad_tkbrep::topods::Edge,
    first: &Vertex,
    last: &Vertex,
) -> Result<(), Failure> {
    let copy = ShapeBuildEdge.copy_replace_vertices(edge, first, last)?;
    ReShape::replace(
        &*fixer.context(),
        &edge.clone().into_shape(),
        &copy.into_shape(),
    )
}
