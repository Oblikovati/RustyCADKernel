// Rust translation of ChFi2d_FilletAlgo.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ChFi2d_FilletAlgo`: the iterative fillet of two planar edges: the fillet function
//! (distance from the center of a circle tangent to the first curve to the second curve,
//! minus the radius) sampled along the first curve, and its roots by a recursive Newton
//! method.
mod fill;
mod newton;
mod point;
mod result;

pub use point::FilletPoint;

use super::first_two_edges;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topods::{Edge, Wire};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkgeombase::geom_proj_lib;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::Plane3;

/// `ChFi2d_FilletAlgo`: the edges (the more complicated or parametrically shorter one
/// first), the plane, their curves in it and ranges, the radius, the roots found with
/// the side of the fillet, and the recursion depth.
///
/// ```
/// use rustycad_tkfillet::chfi2d::ChFi2dFilletAlgo;
/// let algo = ChFi2dFilletAlgo::new();
/// # let _ = algo;
/// ```
#[derive(Clone, Default)]
pub struct ChFi2dFilletAlgo {
    edge1: Edge,
    edge2: Edge,
    plane: Handle<GeomPlane>,
    curve1: Handle<dyn Geom2dCurve>,
    curve2: Handle<dyn Geom2dCurve>,
    start1: f64,
    end1: f64,
    start2: f64,
    end2: f64,
    radius: f64,
    result_params: Vec<f64>,
    result_orientation: Vec<i32>,
    start_side: bool,
    edges_exchanged: bool,
    degree_of_recursion: i32,
}

impl ChFi2dFilletAlgo {
    /// `ChFi2d_FilletAlgo()`: use `init_wire` or `init_edges` next.
    pub fn new() -> Self {
        Self::default()
    }

    /// `ChFi2d_FilletAlgo(theWire, thePlane)`.
    pub fn with_wire(wire: &Wire, plane: &Plane3) -> Result<Self, Failure> {
        let mut algo = Self::new();
        algo.init_wire(wire, plane)?;
        Ok(algo)
    }

    /// `ChFi2d_FilletAlgo(theEdge1, theEdge2, thePlane)`.
    pub fn with_edges(edge1: &Edge, edge2: &Edge, plane: &Plane3) -> Result<Self, Failure> {
        let mut algo = Self {
            edge1: edge1.clone(),
            edge2: edge2.clone(),
            ..Self::new()
        };
        algo.init_edges(edge1, edge2, plane)?;
        Ok(algo)
    }

    /// `Init(theWire, thePlane)`: the first two edges of the wire;
    /// Standard_ConstructionError without two edges.
    pub fn init_wire(&mut self, wire: &Wire, plane: &Plane3) -> Result<(), Failure> {
        let (e1, e2) = first_two_edges(wire)?;
        if e1.is_null() || e2.is_null() {
            return Err(Failure::with_message(
                "Standard_ConstructionError",
                "ChFi2d_FilletAlgo: The fillet algorithms expects a wire consisting of two edges.",
            ));
        }
        self.init_edges(&e1, &e2, plane)
    }

    /// `Init(theEdge1, theEdge2, thePlane)`: the first curve must be more complicated
    /// (then parametrically shorter); the curves are projected on the plane.
    pub fn init_edges(
        &mut self,
        edge1: &Edge,
        edge2: &Edge,
        plane: &Plane3,
    ) -> Result<(), Failure> {
        self.plane = Handle::new(GeomPlane::from_pln(*plane));
        let (kind1, kind2) = (curve_type(edge1)?, curve_type(edge2)?);
        // first curve must be more complicated
        self.edges_exchanged = (kind1 as i32) < (kind2 as i32);
        (self.edge1, self.edge2) = if self.edges_exchanged {
            (edge2.clone(), edge1.clone())
        } else {
            (edge1.clone(), edge2.clone())
        };
        (self.curve1, self.start1, self.end1) = self.projected(&self.edge1.clone())?;
        (self.curve2, self.start2, self.end2) = self.projected(&self.edge2.clone())?;
        self.end1 = unwrapped_end(&self.curve1, self.start1, self.end1)?;
        self.end2 = unwrapped_end(&self.curve2, self.start2, self.end2)?;
        // first curve must be parametrically shorter
        if kind1 == kind2 && self.end2 - self.start2 < self.end1 - self.start1 {
            self.swap_edges();
        }
        Ok(())
    }

    /// The curve of an edge in the plane (GeomProjLib::Curve2d) and its range.
    fn projected(&self, edge: &Edge) -> Result<(Handle<dyn Geom2dCurve>, f64, f64), Failure> {
        let (curve, first, last) = BRepTool::curve(edge)?;
        let surface: Handle<dyn GeomSurface> = self.plane.clone().cast();
        let c2d = geom_proj_lib::curve2d_range(&curve, (first, last), &surface)?;
        Ok((c2d, first, last))
    }

    /// The edges, curves and ranges swapped.
    fn swap_edges(&mut self) {
        std::mem::swap(&mut self.edge1, &mut self.edge2);
        std::mem::swap(&mut self.curve1, &mut self.curve2);
        std::mem::swap(&mut self.start1, &mut self.start2);
        std::mem::swap(&mut self.end1, &mut self.end2);
        self.edges_exchanged = true;
    }

    /// `Perform(theRadius)`: the roots of the fillet function on both sides of the first
    /// curve; true when at least one was found. `algo.perform(1.0)?`.
    pub fn perform(&mut self, radius: f64) -> Result<bool, Failure> {
        self.degree_of_recursion = 0;
        self.result_params.clear();
        self.result_orientation.clear();
        let nb_steps = self.nb_steps()?;
        self.radius = radius;
        let step = (self.end1 - self.start1) / nb_steps;
        let dstep = 1.0e-4 * step;
        self.start_side = false;
        for _cycle in 0..2 {
            self.sweep_side(step, dstep)?;
            self.start_side = !self.start_side;
        }
        Ok(!self.result_params.is_empty())
    }

    /// The number of sampling steps on the first curve by its type.
    fn nb_steps(&self) -> Result<f64, Failure> {
        let gac = Geom2dAdaptorCurve::with_curve(&self.curve1)?;
        Ok(match gac.get_type()? {
            CurveType::Line => 2.0,
            CurveType::Circle => 4.0,
            CurveType::Ellipse => 5.0,
            CurveType::BSplineCurve => 2.0 + f64::from(gac.degree()? * gac.nb_poles()?),
            // unknown: maximum
            _ => 100.0,
        })
    }

    /// The Newton search on each sampling interval of one side.
    fn sweep_side(&mut self, step: f64, dstep: f64) -> Result<(), Failure> {
        let mut left: Option<FilletPoint> = None;
        let mut param = self.start1 + step;
        while param < self.end1 || (self.end1 - param).abs() < precision::confusion() {
            let mut l = match left.take() {
                Some(l) => l,
                None => {
                    let mut l = FilletPoint::new(param - step);
                    self.fill_point(&mut l, param)?;
                    self.fill_diff(&mut l, dstep, true)?;
                    l
                }
            };
            let mut right = FilletPoint::new(param);
            self.fill_point(&mut right, param - step)?;
            self.fill_diff(&mut right, dstep, false)?;
            l.filter_points(&right)?;
            self.perform_newton(&l, &right)?;
            left = Some(right);
            param += step;
        }
        Ok(())
    }
}

/// The type of the 3d curve of an edge (BRepAdaptor_Curve).
fn curve_type(edge: &Edge) -> Result<CurveType, Failure> {
    BRepAdaptorCurve::with_edge(edge)?.get_type()
}

/// The end of a periodic curve moved by periods beyond its start.
fn unwrapped_end(curve: &Handle<dyn Geom2dCurve>, start: f64, end: f64) -> Result<f64, Failure> {
    let c = curve.get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            "ChFi2d_FilletAlgo: null projected curve",
        )
    })?;
    let mut end = end;
    while c.is_periodic() && start >= end {
        end += c.period()?;
    }
    Ok(end)
}
