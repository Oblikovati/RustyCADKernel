// Rust translation of ChFi2d_FilletAPI.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ChFi2d_FilletAPI`: the fillet of two planar edges, by the analytical algorithm for
//! segments and arcs with a common point, else by the iterative one.
use super::{ChFi2dAnaFilletAlgo, ChFi2dFilletAlgo, first_two_edges};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topods::{Edge, Wire};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Plane3, Point3};

/// `ChFi2d_FilletAPI`: both algorithms and which one applies.
///
/// ```
/// use rustycad_tkfillet::chfi2d::ChFi2dFilletAPI;
/// let api = ChFi2dFilletAPI::new();
/// # let _ = api;
/// ```
#[derive(Clone, Default)]
pub struct ChFi2dFilletAPI {
    fillet_algo: ChFi2dFilletAlgo,
    ana_fillet_algo: ChFi2dAnaFilletAlgo,
    is_analytical: bool,
}

impl ChFi2dFilletAPI {
    /// `ChFi2d_FilletAPI()`: use `init_wire` or `init_edges` next.
    pub fn new() -> Self {
        Self::default()
    }

    /// `ChFi2d_FilletAPI(theWire, thePlane)`.
    pub fn with_wire(wire: &Wire, plane: &Plane3) -> Result<Self, Failure> {
        let mut api = Self::new();
        api.init_wire(wire, plane)?;
        Ok(api)
    }

    /// `ChFi2d_FilletAPI(theEdge1, theEdge2, thePlane)`.
    pub fn with_edges(edge1: &Edge, edge2: &Edge, plane: &Plane3) -> Result<Self, Failure> {
        let mut api = Self::new();
        api.init_edges(edge1, edge2, plane)?;
        Ok(api)
    }

    /// `Init(theWire, thePlane)`: the analytical algorithm when the first two edges
    /// allow it (else the choice of a previous Init stays).
    pub fn init_wire(&mut self, wire: &Wire, plane: &Plane3) -> Result<(), Failure> {
        let (e1, e2) = first_two_edges(wire)?;
        if !e1.is_null() && !e2.is_null() {
            self.is_analytical = is_analytical(&e1, &e2)?;
        }
        if self.is_analytical {
            self.ana_fillet_algo.init_wire(wire, plane)
        } else {
            self.fillet_algo.init_wire(wire, plane)
        }
    }

    /// `Init(theEdge1, theEdge2, thePlane)`.
    pub fn init_edges(
        &mut self,
        edge1: &Edge,
        edge2: &Edge,
        plane: &Plane3,
    ) -> Result<(), Failure> {
        self.is_analytical = is_analytical(edge1, edge2)?;
        if self.is_analytical {
            self.ana_fillet_algo.init_edges(edge1, edge2, plane)
        } else {
            self.fillet_algo.init_edges(edge1, edge2, plane)
        }
    }

    /// `Perform(theRadius)`: true when at least one fillet was found.
    pub fn perform(&mut self, radius: f64) -> Result<bool, Failure> {
        if self.is_analytical {
            self.ana_fillet_algo.perform(radius)
        } else {
            self.fillet_algo.perform(radius)
        }
    }

    /// `NbResults(thePoint)`: 1 for the analytical algorithm.
    pub fn nb_results(&mut self, point: Point3) -> Result<i32, Failure> {
        if self.is_analytical {
            return Ok(1);
        }
        self.fillet_algo.nb_results(point)
    }

    /// `Result(thePoint, theEdge1, theEdge2, iSolution)`: the fillet edge (nearest to the
    /// point when `i_solution` is -1) and the edges cut at its ends.
    pub fn result(
        &mut self,
        point: Point3,
        edge1: &mut Edge,
        edge2: &mut Edge,
        i_solution: i32,
    ) -> Result<Edge, Failure> {
        if self.is_analytical {
            return Ok(self.ana_fillet_algo.result(edge1, edge2));
        }
        self.fillet_algo.result(point, edge1, edge2, i_solution)
    }
}

/// `IsAnalytical(theEdge1, theEdge2)`: lines or arcs of circle with a common point.
fn is_analytical(edge1: &Edge, edge2: &Edge) -> Result<bool, Failure> {
    let (ac1, ac2) = (
        BRepAdaptorCurve::with_edge(edge1)?,
        BRepAdaptorCurve::with_edge(edge2)?,
    );
    let simple = |k: CurveType| k == CurveType::Line || k == CurveType::Circle;
    if !simple(ac1.get_type()?) || !simple(ac2.get_type()?) {
        return Ok(false);
    }
    // The edges are lines or arcs of circle; now check whether they have a common point.
    let ends = |ac: &BRepAdaptorCurve| -> Result<[Point3; 2], Failure> {
        Ok([
            ac.value(ac.first_parameter()?)?,
            ac.value(ac.last_parameter()?)?,
        ])
    };
    let ([p11, p12], [p21, p22]) = (ends(&ac1)?, ends(&ac2)?);
    let close = |a: Point3, b: Point3| a.square_distance(&b) < precision::square_confusion();
    Ok(close(p11, p21) || close(p11, p22) || close(p12, p21) || close(p12, p22))
}
