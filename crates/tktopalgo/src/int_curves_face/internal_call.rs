// Rust translation of IntCurvesFace_Intersector.cxx (InternalCall) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `InternalCall` of `IntCurvesFace_Intersector`: the classification of each
//! curve-surface point against the face and the sorted insertion of the kept ones.
use super::intersector::IntCurvesFaceIntersector;
use crate::brep_top_adaptor::BRepTopAdaptorTopolTool;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use rustycad_tkgeomalgo::geom_api::ProjectPointOnCurve;
use rustycad_tkgeomalgo::int_curve_surface::{HInter, IntersectionPoint, TransitionOnCurve};
use rustycad_tkmath::gp::Point2;

/// The 3d and 2d tolerances of the face boundary used to classify the points.
struct BoundTolerances {
    mintol2d: f64,
    maxtol2d: f64,
    maxtol3d: f64,
}

impl IntCurvesFaceIntersector {
    /// `InternalCall(HICS, parinf, parsup)`: keeps the points of `hics` that are IN or
    /// ON the face with a curve parameter in `[parinf, parsup]` (the last point
    /// first), or records the parallel flag of a done intersection without points.
    pub(super) fn internal_call(
        &mut self,
        hics: &HInter,
        parinf: f64,
        parsup: f64,
    ) -> Result<(), Failure> {
        if !(hics.is_done() && hics.nb_points()? > 0) {
            if hics.is_done() {
                self.my_is_parallel = hics.is_parallel();
            }
            return Ok(());
        }
        // Calculate tolerance for 2d classifier
        let tolerances: BoundTolerances = self.bound_tolerances()?;
        let mut additional_tool: Handle<BRepTopAdaptorTopolTool> = Handle::default();
        for index in (1..=hics.nb_points()?).rev() {
            let point: IntersectionPoint = hics.point(index)?;
            let state: State = self.point_state(&point, &tolerances, &mut additional_tool)?;
            let w: f64 = point.w();
            if (state == State::In || state == State::On) && w >= parinf && w <= parsup {
                self.insert_point(&point, state);
            }
        }
        Ok(())
    }

    // The 3d tolerances of the face and its edges, and their UV resolutions (at least
    // the intersector tolerance).
    fn bound_tolerances(&self) -> Result<BoundTolerances, Failure> {
        let mut mintol3d: f64 = BRepTool::face_tolerance(&self.face)?;
        let mut maxtol3d: f64 = mintol3d;
        let mut edges = Explorer::new(&self.face, ShapeEnum::Edge, ShapeEnum::Shape)?;
        while edges.more() {
            let curtol: f64 = BRepTool::edge_tolerance(&topods::edge(edges.current())?)?;
            mintol3d = std_min(mintol3d, curtol);
            maxtol3d = std_max(maxtol3d, curtol);
            edges.next()?;
        }
        let s = &self.hsurface;
        let minres: f64 = std_max(s.u_resolution(mintol3d)?, s.v_resolution(mintol3d)?);
        let maxres: f64 = std_max(s.u_resolution(maxtol3d)?, s.v_resolution(maxtol3d)?);
        Ok(BoundTolerances {
            mintol2d: std_max(minres, self.tol),
            maxtol2d: std_max(maxres, self.tol),
            maxtol3d,
        })
    }

    // The state of one point: the tool's classification with the smallest boundary
    // tolerance (0 without boundary tolerances); an OUT point that a second tool finds
    // ON with the largest tolerance is ON only when an edge 3d curve is within the
    // largest 3d tolerance of it.
    fn point_state(
        &self,
        point: &IntersectionPoint,
        tolerances: &BoundTolerances,
        additional_tool: &mut Handle<BRepTopAdaptorTopolTool>,
    ) -> Result<State, Failure> {
        let puv = Point2::new(point.u(), point.v());
        let tol: f64 = if !self.my_use_bound_tol {
            0.0
        } else {
            tolerances.mintol2d
        };
        let state: State = self.my_topol_tool.classify(puv, tol, true)?;
        if !(self.my_use_bound_tol
            && state == State::Out
            && tolerances.maxtol2d > tolerances.mintol2d)
        {
            return Ok(state);
        }
        if additional_tool.is_null() {
            *additional_tool = Handle::new(BRepTopAdaptorTopolTool::with_surface(&self.hsurface)?);
        }
        let state: State = additional_tool.classify(puv, tolerances.maxtol2d, true)?;
        if state != State::On {
            return Ok(state);
        }
        // Find out nearest edge and it's tolerance
        if self.has_edge_within(point, tolerances.maxtol3d)? {
            // Nearest edge is found, state is really ON
            return Ok(State::On);
        }
        Ok(State::Out)
    }

    // Whether the 3d curve of an edge of the face projects on the point within `tol`.
    fn has_edge_within(&self, point: &IntersectionPoint, tol: f64) -> Result<bool, Failure> {
        let mut edges = Explorer::new(&self.face, ShapeEnum::Edge, ShapeEnum::Shape)?;
        while edges.more() {
            let edge: Edge = topods::edge(edges.current())?;
            let (curve, first, last) = BRepTool::curve(&edge)?;
            let projection = ProjectPointOnCurve::with_range(point.pnt(), &curve, first, last)?;
            if projection.nb_points() > 0 && projection.lower_distance()? <= tol {
                return Ok(true);
            }
            edges.next()?;
        }
        Ok(false)
    }

    // `Insertion du point`: before the first point whose parameter is not below it,
    // with its transition swapped (IN and OUT) on a REVERSED face.
    fn insert_point(&mut self, point: &IntersectionPoint, state: State) {
        //  Modified by skv - Wed Sep  3 16:14:10 2003 OCC578: the state is kept too.
        let int_state: i32 = if state == State::In { 0 } else { 1 };
        let mut transition: TransitionOnCurve = point.transition();
        if transition != TransitionOnCurve::Tangent
            && self.face.orientation() == Orientation::Reversed
        {
            transition = if transition == TransitionOnCurve::In {
                TransitionOnCurve::Out
            } else {
                TransitionOnCurve::In
            };
        }
        let (u, v, w) = (point.u(), point.v(), point.w());
        let ppp = IntersectionPoint::new(point.pnt(), u, v, w, transition);
        let slot: usize = self
            .seq_pnt
            .iter()
            .position(|existing| existing.w() >= w)
            .unwrap_or(self.seq_pnt.len());
        self.seq_pnt.insert(slot, ppp);
        self.my_seq_state.insert(slot, int_state);
        self.nbpnt += 1;
    }
}
