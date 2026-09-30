// Rust translation of ModelingData/TKGeomBase/Geom2dConvert/Geom2dConvert_ApproxArcsSegments.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dConvert_ApproxArcsSegments`: the approximation of a 2D curve by a chain of
//! line segments and circular arcs (biarcs) within a distance and an angle.
//!
//! The native `.cxx` is split by responsibility: the elementary builders
//! (`builders.rs`), the free-form scan for straight and inflection parts
//! (`freeform.rs`), the recursive biarc approximation (`biarcs.rs`) and the file
//! static functions (`curve_tests.rs`).
mod biarcs;
mod builders;
mod curve_tests;
mod freeform;

use super::ppoint::PPoint;
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine, Geom2dTrimmedCurve};
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Axis2d, Direction2, Point2, Vector2};

/// `MAXPOINTS`: the sample count of the fallback scans.
const MAXPOINTS: i32 = 100;
/// `MyCurvatureTolerance`: the curvature below which a point is straight.
const MY_CURVATURE_TOLERANCE: f64 = 0.0001;

/// `Geom2dConvert_ApproxArcsSegments::Status`, the state of the computation (the
/// native object keeps it private).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum ApproxArcsStatus {
    /// `StatusOK`.
    Ok = 0,
    /// `StatusNotDone`.
    NotDone,
    /// `StatusError`.
    Error,
}

/// The approximation of a curve by segments and arcs, computed at construction.
///
/// `let approx = ApproxArcsSegments::new(&adaptor, 1e-3, 0.1)?; let curves = approx.get_result();`
pub struct ApproxArcsSegments<'a> {
    my_curve: &'a dyn Adaptor2dCurve2d,
    my_ext: [PPoint; 2],
    my_tolerance: f64,
    my_angle_tolerance: f64,
    my_seq_params: Vec<PPoint>,
    my_seq_curves: Sequence<Handle<dyn Geom2dCurve>>,
    my_status: ApproxArcsStatus,
}

impl<'a> ApproxArcsSegments<'a> {
    /// Approximates `curve`: a line gives one segment, a circle two arcs, any other
    /// curve segments on its straight parts and biarcs elsewhere, each within
    /// `tolerance` of the curve and meeting the next within `angle_tol`. An
    /// exception of the adaptor or of a gp construction propagates
    /// (`Geom2dConvert_ApproxArcsSegments(theCurve, theTolerance, theAngleTol)`);
    /// `ApproxArcsSegments::new(&adaptor, 1e-3, 0.1)?`.
    pub fn new(
        curve: &'a dyn Adaptor2dCurve2d,
        tolerance: f64,
        angle_tol: f64,
    ) -> Result<Self, Failure> {
        let first = PPoint::from_adaptor(curve.first_parameter()?, curve)?;
        let last = PPoint::from_adaptor(curve.last_parameter()?, curve)?;
        let mut approx = Self {
            my_curve: curve,
            my_ext: [first, last],
            my_tolerance: tolerance,
            my_angle_tolerance: angle_tol,
            my_seq_params: Vec::new(),
            my_seq_curves: Sequence::default(),
            my_status: ApproxArcsStatus::NotDone,
        };
        match curve.get_type()? {
            CurveType::Line => approx.single_segment()?,
            CurveType::Circle => approx.two_arcs()?,
            _ => {
                approx.make_freeform()?;
            }
        }
        approx.check_status();
        Ok(approx)
    }

    /// The segments and arcs, in the order of the curve (`GetResult()`). A failed
    /// approximation keeps what it built, and can end with a null curve (see the
    /// preserved behaviour of `makeApproximation`); `approx.get_result()`.
    pub fn get_result(&self) -> &Sequence<Handle<dyn Geom2dCurve>> {
        &self.my_seq_curves
    }

    // A line: a single segment from the first to the last point, when they are apart.
    fn single_segment(&mut self) -> Result<(), Failure> {
        let dist: f64 = self.my_ext[0].dist(&self.my_ext[1]);
        if dist > precision::confusion() {
            let start = Point2::new(self.my_ext[0].point().x(), self.my_ext[0].point().y());
            let end = Point2::new(self.my_ext[1].point().x(), self.my_ext[1].point().y());
            let axis = Axis2d::new(
                start,
                Direction2::from_vector(Vector2::between(start, end))?,
            );
            let line: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(axis)).cast();
            let segment = Geom2dTrimmedCurve::new(&line, 0.0, dist, true, true)?;
            self.my_seq_curves.append(Handle::new(segment).cast());
            self.my_status = ApproxArcsStatus::Ok;
        }
        Ok(())
    }

    // A circle: a couple of arcs of equal size.
    fn two_arcs(&mut self) -> Result<(), Failure> {
        let middle: f64 = 0.5 * (self.my_ext[0].parameter() + self.my_ext[1].parameter());
        let pp = PPoint::from_adaptor(middle, self.my_curve)?;
        let arc: Handle<dyn Geom2dCurve> = self.make_circle(&self.my_ext[0], &pp)?;
        if arc.is_null() {
            return Ok(());
        }
        self.my_seq_curves.append(arc);
        let arc: Handle<dyn Geom2dCurve> = self.make_circle(&pp, &self.my_ext[1])?;
        if !arc.is_null() {
            self.my_seq_curves.append(arc);
        }
        Ok(())
    }

    // Check the status of the calculation.
    fn check_status(&mut self) {
        if self.my_status == ApproxArcsStatus::NotDone {
            self.my_status = if self.my_seq_curves.is_empty() {
                ApproxArcsStatus::Error
            } else {
                ApproxArcsStatus::Ok
            };
        }
    }
}

// The 1-based element of a parameter sequence (NCollection_Sequence::Value).
fn seq_at(sequence: &[PPoint], index: i32) -> PPoint {
    sequence[(index - 1) as usize]
}
