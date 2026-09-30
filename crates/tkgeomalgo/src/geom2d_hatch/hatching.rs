// Rust translation of Geom2dHatch_Hatching.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dHatch_Hatching`: a hatching curve with its intersection points and
//! domains.
use crate::hatch_gen::text::sequence_slot;
use crate::hatch_gen::{Domain, ErrorStatus, PointOnHatching};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkmath::gp::Point2;

/// `Geom2dHatch_Hatching`: the curve of a hatching (owned by value), its
/// intersection points sorted along it, its domains and the flags and status of
/// their computation. Changing the points clears the domains of a done hatching.
///
/// ```
/// use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
/// use rustycad_tkgeomalgo::geom2d_hatch::Hatching;
/// use rustycad_tkgeomalgo::hatch_gen::PointOnHatching;
/// let mut hatching = Hatching::new(Geom2dAdaptorCurve::default());
/// hatching.add_point(&PointOnHatching::new(), 1e-9);
/// assert_eq!(hatching.nb_points(), 1);
/// ```
#[derive(Default)]
pub struct Hatching {
    my_curve: Geom2dAdaptorCurve,
    my_trim_done: bool,
    my_trim_failed: bool,
    my_points: Vec<PointOnHatching>,
    my_is_done: bool,
    my_status: ErrorStatus,
    my_domains: Vec<Domain>,
}

impl Hatching {
    /// `Geom2dHatch_Hatching(Curve)`: the hatching keeps `curve`, without points;
    /// `Hatching::new(adaptor)`.
    pub fn new(curve: Geom2dAdaptorCurve) -> Self {
        Self {
            my_curve: curve,
            ..Self::default()
        }
    }

    /// Curve(); `hatching.curve()`.
    pub fn curve(&self) -> &Geom2dAdaptorCurve {
        &self.my_curve
    }

    /// ChangeCurve(); `hatching.change_curve()`.
    pub fn change_curve(&mut self) -> &mut Geom2dAdaptorCurve {
        &mut self.my_curve
    }

    /// TrimDone(Flag): the intersections were computed; `hatching.set_trim_done(true)`.
    pub fn set_trim_done(&mut self, flag: bool) {
        self.my_trim_done = flag;
    }

    /// TrimDone(); `hatching.trim_done()`.
    pub fn trim_done(&self) -> bool {
        self.my_trim_done
    }

    /// TrimFailed(Flag): a true flag also sets the status TrimFailure;
    /// `hatching.set_trim_failed(true)`.
    pub fn set_trim_failed(&mut self, flag: bool) {
        self.my_trim_failed = flag;
        if self.my_trim_failed {
            self.my_status = ErrorStatus::TrimFailure;
        }
    }

    /// TrimFailed(); `hatching.trim_failed()`.
    pub fn trim_failed(&self) -> bool {
        self.my_trim_failed
    }

    /// IsDone(Flag): the domains were computed; `hatching.set_is_done(true)`.
    pub fn set_is_done(&mut self, flag: bool) {
        self.my_is_done = flag;
    }

    /// IsDone(); `hatching.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_is_done
    }

    /// Status(theStatus); `hatching.set_status(ErrorStatus::NoProblem)`.
    pub fn set_status(&mut self, status: ErrorStatus) {
        self.my_status = status;
    }

    /// Status(); `hatching.status()`.
    pub fn status(&self) -> ErrorStatus {
        self.my_status
    }

    /// AddPoint(Point, Confusion): inserts `point` before the first point that is
    /// not lower than it, or merges its points on element into an equal one (within
    /// `confusion`); clears the domains of a done hatching;
    /// `hatching.add_point(&point, 1e-9)`.
    pub fn add_point(&mut self, point: &PointOnHatching, confusion: f64) {
        let slot: Option<usize> = self
            .my_points
            .iter()
            .position(|known| !known.is_lower(point, confusion));
        match slot {
            None => self.my_points.push(point.clone()),
            Some(slot) if self.my_points[slot].is_greater(point, confusion) => {
                self.my_points.insert(slot, point.clone());
            }
            Some(slot) => merge_element_points(&mut self.my_points[slot], point, confusion),
        }
        if self.my_is_done {
            self.clr_domains();
        }
    }

    /// NbPoints(); `hatching.nb_points()`.
    pub fn nb_points(&self) -> i32 {
        self.my_points.len() as i32
    }

    /// Point(Index): Standard_OutOfRange outside [1, NbPoints]; `hatching.point(1)?`.
    pub fn point(&self, index: i32) -> Result<&PointOnHatching, Failure> {
        let slot = sequence_slot(self.my_points.len(), index, "Geom2dHatch_Hatching::Point")?;
        Ok(&self.my_points[slot])
    }

    /// ChangePoint(Index): Standard_OutOfRange outside [1, NbPoints];
    /// `hatching.change_point(1)?`.
    pub fn change_point(&mut self, index: i32) -> Result<&mut PointOnHatching, Failure> {
        let slot = sequence_slot(
            self.my_points.len(),
            index,
            "Geom2dHatch_Hatching::ChangePoint",
        )?;
        Ok(&mut self.my_points[slot])
    }

    /// RemPoint(Index): clears the domains of a done hatching, then removes the
    /// point; Standard_OutOfRange outside [1, NbPoints]; `hatching.rem_point(1)?`.
    pub fn rem_point(&mut self, index: i32) -> Result<(), Failure> {
        if self.my_is_done {
            self.clr_domains();
        }
        let slot = sequence_slot(
            self.my_points.len(),
            index,
            "Geom2dHatch_Hatching::RemPoint",
        )?;
        self.my_points.remove(slot);
        Ok(())
    }

    // RemPoint at a 0-based slot known to be in range.
    pub(super) fn remove_point_slot(&mut self, slot: usize) {
        if self.my_is_done {
            self.clr_domains();
        }
        self.my_points.remove(slot);
    }

    // The points in their order along the hatching.
    pub(super) fn points(&self) -> &[PointOnHatching] {
        &self.my_points
    }

    /// ClrPoints(): removes every point, clears the domains of a done hatching and
    /// both trimming flags (the status stays); `hatching.clr_points()`.
    pub fn clr_points(&mut self) {
        if self.my_is_done {
            self.clr_domains();
        }
        self.my_points
            .iter_mut()
            .for_each(PointOnHatching::clr_points);
        self.my_points.clear();
        self.my_trim_done = false;
        self.my_trim_failed = false;
    }

    /// AddDomain(Domain); `hatching.add_domain(domain)`.
    pub fn add_domain(&mut self, domain: Domain) {
        self.my_domains.push(domain);
    }

    /// NbDomains(); `hatching.nb_domains()`.
    pub fn nb_domains(&self) -> i32 {
        self.my_domains.len() as i32
    }

    /// Domain(Index): Standard_OutOfRange outside [1, NbDomains]; `hatching.domain(1)?`.
    pub fn domain(&self, index: i32) -> Result<&Domain, Failure> {
        let slot = sequence_slot(self.my_domains.len(), index, "Geom2dHatch_Hatching::Domain")?;
        Ok(&self.my_domains[slot])
    }

    /// RemDomain(Index): Standard_OutOfRange outside [1, NbDomains];
    /// `hatching.rem_domain(1)?`.
    pub fn rem_domain(&mut self, index: i32) -> Result<(), Failure> {
        let slot = sequence_slot(
            self.my_domains.len(),
            index,
            "Geom2dHatch_Hatching::RemDomain",
        )?;
        self.my_domains.remove(slot);
        Ok(())
    }

    /// ClrDomains(): removes every domain and marks the domains not computed;
    /// `hatching.clr_domains()`.
    pub fn clr_domains(&mut self) {
        self.my_domains.clear();
        self.my_is_done = false;
    }

    /// ClassificationPoint(): the point of the curve at its last parameter, or at its
    /// first one when the last is infinite, or at 0 when both are;
    /// `hatching.classification_point()?`.
    pub fn classification_point(&self) -> Result<Point2, Failure> {
        let a: f64 = self.my_curve.first_parameter()?;
        let b: f64 = self.my_curve.last_parameter()?;
        let t: f64 = if b >= precision::infinite() {
            if a <= -precision::infinite() { 0.0 } else { a }
        } else {
            b
        };
        self.my_curve.value(t)
    }
}

// The points on element of `point` added to the equal point `known`.
fn merge_element_points(known: &mut PointOnHatching, point: &PointOnHatching, confusion: f64) {
    for element_point in point.element_points() {
        known.add_point(element_point, confusion);
    }
}
