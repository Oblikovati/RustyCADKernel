// Rust translation of ShapeAlgo_AlgoContainer::HomoWires at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `HomoWires`: two wires cut into the same number of edges, the cuts placed in
//! proportion to the parameter ranges of their 3D curves.
use super::ShapeAlgoAlgoContainer;
use crate::shape_extend::ShapeExtendWireData;
use crate::shape_fix::ShapeFixWire;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{self, Edge, ShapeIterator, Wire};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::toploc::Location;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;

/// The native iterator over a wire (`Cook`, `Perry`) with the 3D curve of its current
/// edge (`crv`, `loc`, `first`, `last`) and its `IsToReverse` flag.
struct EdgeCursor {
    edges: Vec<Edge>,
    index: usize,
    curve: Handle<dyn GeomCurve>,
    location: Location,
    first: f64,
    last: f64,
    reverse: bool,
}

impl EdgeCursor {
    fn new(edges: Vec<Edge>) -> Result<Self, Failure> {
        let mut cursor = Self {
            edges,
            index: 0,
            curve: Handle::default(),
            location: Location::default(),
            first: 0.,
            last: 0.,
            reverse: false,
        };
        cursor.load()?;
        Ok(cursor)
    }

    /// `edge = TopoDS::Edge(It.Value())` and `BRep_Tool::Curve(edge, loc, first, last)`;
    /// Standard_NoSuchObject past the end (the native Value() of a finished iterator).
    fn load(&mut self) -> Result<(), Failure> {
        let edge = self
            .edges
            .get(self.index)
            .ok_or_else(|| no_more_edges(self.index))?;
        self.reverse = edge.orientation() == Orientation::Reversed;
        let found = BRepTool::curve_with_location(edge)?;
        (self.curve, self.location) = (found.curve, found.location);
        (self.first, self.last) = (found.first, found.last);
        Ok(())
    }

    fn advance(&mut self) -> Result<(), Failure> {
        self.index += 1;
        self.load()
    }

    fn delta(&self) -> f64 {
        self.last - self.first
    }

    fn is_last(&self) -> bool {
        self.index + 1 >= self.edges.len()
    }

    /// The edge on the whole current range.
    fn whole(&self) -> Result<Edge, Failure> {
        make_edge(&self.curve, self.first, self.last)
    }

    /// The edge on the first `length` of the range in the edge direction (its end for a
    /// reversed edge, "gka BUC60685"), removed from the range.
    fn cut(&mut self, length: f64) -> Result<Edge, Failure> {
        if !self.reverse {
            let edge = make_edge(&self.curve, self.first, self.first + length)?;
            self.first += length;
            return Ok(edge);
        }
        let edge = make_edge(&self.curve, self.last - length, self.last)?;
        self.last -= length;
        Ok(edge)
    }

    /// The made edge moved to the location of the curve and reversed as the original.
    fn placed(&self, mut edge: Edge) -> Result<Edge, Failure> {
        edge.move_by(&self.location, false)?;
        if self.reverse {
            edge.reverse();
        }
        Ok(edge)
    }
}

/// The edges made by one step, and which iterators advance (`iterCook`, `iterPerry`).
struct HomoStep {
    edges: (Edge, Edge),
    advance: (bool, bool),
    created: i32,
}

impl ShapeAlgoAlgoContainer {
    /// `HomoWires(wireIn1, wireIn2, wireOut1, wireOut2, byParam)`: both wires reordered
    /// (ShapeFix_Wire::FixReorder), then cut so that edge k of one spans the same share of
    /// the total 3D parameter range as edge k of the other; `None` is the native false
    /// (after nEdges1 + nEdges2 - 1 edges). `byParam` is unused natively.
    ///
    /// Native quirks (kept): one edge in each wire returns the reordered wires; a wire of
    /// nearly null total range becomes its first edge repeated once per edge of the other.
    /// Example: `let pair = container.homo_wires(&w1, &w2, false)?;`.
    pub fn homo_wires(
        &self,
        wire_in1: &Wire,
        wire_in2: &Wire,
        _by_param: bool,
    ) -> Result<Option<(Wire, Wire)>, Failure> {
        let fix = ShapeFixWire::new();
        let (wire11, wire22) = (reordered(&fix, wire_in1)?, reordered(&fix, wire_in2)?);
        let (edges1, edges2) = (wire_edges(&wire11)?, wire_edges(&wire22)?);
        let (length1, length2) = (total_range(&edges1)?, total_range(&edges2)?);
        let epsilon = precision::p_confusion() * (length1 + length2);
        if edges1.len() == 1 && edges2.len() == 1 {
            return Ok(Some((wire11, wire22)));
        }
        if length1 < epsilon {
            return Ok(Some((repeated_first(&edges1, edges2.len())?, wire22)));
        }
        if length2 < epsilon {
            return Ok(Some((wire11, repeated_first(&edges2, edges1.len())?)));
        }
        let limit = (edges1.len() + edges2.len()) as i32 - 1;
        let cursors = (EdgeCursor::new(edges1)?, EdgeCursor::new(edges2)?);
        split_in_proportion(cursors, length2 / length1, epsilon, limit)
    }
}

fn split_in_proportion(
    (mut cook, mut perry): (EdgeCursor, EdgeCursor),
    ratio: f64,
    epsilon: f64,
    limit: i32,
) -> Result<Option<(Wire, Wire)>, Failure> {
    let (made1, made2) = (ShapeExtendWireData::new(), ShapeExtendWireData::new());
    let mut nb_created = 0;
    while nb_created < limit {
        let step = homo_step(&mut cook, &mut perry, ratio, epsilon)?;
        nb_created += step.created;
        made1.add_edge(&cook.placed(step.edges.0)?, 0)?;
        made2.add_edge(&perry.placed(step.edges.1)?, 0)?;
        if step.advance == (true, true) && cook.is_last() {
            return Ok(Some((made1.wire()?, made2.wire()?)));
        }
        if step.advance.0 {
            cook.advance()?;
        }
        if step.advance.1 {
            perry.advance()?;
        }
    }
    Ok(None)
}

/// One pass of the native loop: the longer edge (in proportion) is cut to the length of
/// the other, or both are kept whole when they match within `epsilon`.
fn homo_step(
    cook: &mut EdgeCursor,
    perry: &mut EdgeCursor,
    ratio: f64,
    epsilon: f64,
) -> Result<HomoStep, Failure> {
    let excess = cook.delta() * ratio - perry.delta();
    if excess > epsilon {
        let edge1 = cook.cut(perry.delta() / ratio)?;
        let step = HomoStep {
            edges: (edge1, perry.whole()?),
            advance: (false, true),
            created: 1,
        };
        return Ok(step);
    }
    if excess.abs() <= epsilon {
        let edges = (cook.whole()?, perry.whole()?);
        return Ok(HomoStep {
            edges,
            advance: (true, true),
            created: 2,
        });
    }
    let edge1 = cook.whole()?;
    let edge2 = perry.cut(cook.delta() * ratio)?;
    Ok(HomoStep {
        edges: (edge1, edge2),
        advance: (true, false),
        created: 1,
    })
}

/// `sfw->Load(wire); sfw->FixReorder(); sfw->Wire()`.
fn reordered(fix: &ShapeFixWire, wire: &Wire) -> Result<Wire, Failure> {
    fix.load(wire)?;
    fix.fix_reorder(false)?;
    fix.wire()
}

fn wire_edges(wire: &Wire) -> Result<Vec<Edge>, Failure> {
    ShapeIterator::new(wire, true, true)
        .map(|child| topods::edge(&child))
        .collect()
}

/// The sum of the 3D parameter ranges (`last - first` of BRep_Tool::Curve).
fn total_range(edges: &[Edge]) -> Result<f64, Failure> {
    let mut length = 0.;
    for edge in edges {
        let found = BRepTool::curve_with_location(edge)?;
        length += found.last - found.first;
    }
    Ok(length)
}

/// The first edge added `count` times (the native loop that never advances `Cook`).
fn repeated_first(edges: &[Edge], count: usize) -> Result<Wire, Failure> {
    let made = ShapeExtendWireData::new();
    for _ in 0..count {
        made.add_edge(edges.first().ok_or_else(|| no_more_edges(0))?, 0)?;
    }
    made.wire()
}

fn make_edge(curve: &Handle<dyn GeomCurve>, first: f64, last: f64) -> Result<Edge, Failure> {
    BRepBuilderAPIMakeEdge::from_curve_params(curve, first, last)?.edge()
}

fn no_more_edges(index: usize) -> Failure {
    Failure::with_type(
        "Standard_NoSuchObject",
        &format!(
            "ShapeAlgo_AlgoContainer::HomoWires: no edge at index {index}; expected a wire with more edges"
        ),
        "",
    )
}
