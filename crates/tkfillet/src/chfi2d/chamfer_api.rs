// Rust translation of ChFi2d_ChamferAPI.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ChFi2d_ChamferAPI`: the chamfer between two linear edges with a common point.
use super::first_two_edges;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{Edge, Wire};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkgeombase::gc::MakeLine;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;

/// `ChFi2d_ChamferAPI`: the two edges, their curves and ranges, and which ends they
/// share.
///
/// ```
/// use rustycad_tkfillet::chfi2d::ChFi2dChamferAPI;
/// let api = ChFi2dChamferAPI::new();
/// # let _ = api;
/// ```
#[derive(Clone, Default)]
pub struct ChFi2dChamferAPI {
    edge1: Edge,
    edge2: Edge,
    curve1: Handle<dyn GeomCurve>,
    curve2: Handle<dyn GeomCurve>,
    start1: f64,
    end1: f64,
    start2: f64,
    end2: f64,
    common_start1: bool,
    common_start2: bool,
}

impl ChFi2dChamferAPI {
    /// `ChFi2d_ChamferAPI()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `ChFi2d_ChamferAPI(theWire)`: a wire of two linear edges.
    pub fn with_wire(wire: &Wire) -> Result<Self, Failure> {
        let mut api = Self::new();
        api.init_wire(wire)?;
        Ok(api)
    }

    /// `ChFi2d_ChamferAPI(theEdge1, theEdge2)`: two linear edges.
    pub fn with_edges(edge1: &Edge, edge2: &Edge) -> Self {
        let mut api = Self::new();
        api.init_edges(edge1, edge2);
        api
    }

    /// `Init(theWire)`: the first two edges of the wire.
    pub fn init_wire(&mut self, wire: &Wire) -> Result<(), Failure> {
        let (e1, e2) = first_two_edges(wire)?;
        self.init_edges(&e1, &e2);
        Ok(())
    }

    /// `Init(theEdge1, theEdge2)`.
    pub fn init_edges(&mut self, edge1: &Edge, edge2: &Edge) {
        self.edge1 = edge1.clone();
        self.edge2 = edge2.clone();
    }

    /// `Perform()`: the curves of the edges and which of their ends are common; always
    /// true.
    pub fn perform(&mut self) -> Result<bool, Failure> {
        (self.curve1, self.start1, self.end1) = BRepTool::curve(&self.edge1)?;
        (self.curve2, self.start2, self.end2) = BRepTool::curve(&self.edge2)?;
        let (c1, c2) = (live(&self.curve1)?, live(&self.curve2)?);
        let tol = precision::confusion();
        // searching for common points
        (self.common_start1, self.common_start2) =
            if c1.value(self.start1)?.is_equal(&c2.value(self.end2)?, tol) {
                (true, false)
            } else if c1.value(self.end1)?.is_equal(&c2.value(self.start2)?, tol) {
                (false, true)
            } else if c1.value(self.end1)?.is_equal(&c2.value(self.end2)?, tol) {
                (false, false)
            } else {
                (true, true)
            };
        Ok(true)
    }

    /// `Result(theEdge1, theEdge2, theLength1, theLength2)`: the chamfer edge at the
    /// lengths from the common point, and the edges cut there; the null edge when an
    /// edge is shorter than its length.
    pub fn result(
        &self,
        edge1: &mut Edge,
        edge2: &mut Edge,
        (length1, length2): (f64, f64),
    ) -> Result<Edge, Failure> {
        if (self.end1 - self.start1).abs() < length1 || (self.end2 - self.start2).abs() < length2 {
            return Ok(Edge::default());
        }
        let common1 = common_parameter((self.start1, self.end1), self.common_start1, length1);
        let common2 = common_parameter((self.start2, self.end2), self.common_start2, length2);
        let (p1, p2) = (
            live(&self.curve1)?.value(common1)?,
            live(&self.curve2)?.value(common2)?,
        );
        // make chamfer edge
        let line: Handle<dyn GeomCurve> = MakeLine::from_points(p1, p2)?.value()?.cast();
        let result = BRepBuilderAPIMakeEdge::from_curve_points(&line, p1, p2)?.edge()?;
        // divide first edge
        let other1 = if self.common_start1 {
            self.end1
        } else {
            self.start1
        };
        *edge1 =
            BRepBuilderAPIMakeEdge::from_curve_params(&self.curve1, common1, other1)?.edge()?;
        // divide second edge
        let other2 = if self.common_start2 {
            self.end2
        } else {
            self.start2
        };
        *edge2 =
            BRepBuilderAPIMakeEdge::from_curve_params(&self.curve2, common2, other2)?.edge()?;
        Ok(result)
    }
}

/// The parameter at `length` from the common end of a curve (its start when
/// `common_start`), inside its range.
fn common_parameter((start, end): (f64, f64), common_start: bool, length: f64) -> f64 {
    let base = if common_start { start } else { end };
    base + if (start > end) ^ common_start {
        length
    } else {
        -length
    }
}

/// The curve behind its handle.
fn live(curve: &Handle<dyn GeomCurve>) -> Result<&dyn GeomCurve, Failure> {
    curve.get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            "ChFi2d_ChamferAPI: null curve; expected an edge with a 3d curve",
        )
    })
}
