// Rust translation of Geom2dHatch_Hatcher.hxx/.lxx/.cxx (general use, elements,
// hatchings and results) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dHatch_Hatcher`: hatchings trimmed by oriented boundary curves.
use super::{Element, Elements, Hatching, Intersector};
use crate::hatch_gen::{Domain, ErrorStatus, PointOnHatching};
use crate::int_res2d::not_done;
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::top_abs::Orientation;

/// `Geom2dHatch_Hatcher`: computes the domains of hatching curves inside the
/// region bounded by oriented element curves (the region is on the left of a
/// FORWARD element). Elements and hatchings are numbered from 1; a removed number is
/// reused by the next addition. [`Hatcher::trim`] intersects the hatchings with the
/// elements and [`Hatcher::compute_domains`] cuts them into domains, keeping the
/// isolated points and the segments on elements when asked.
///
/// ```
/// use rustycad_tkgeomalgo::geom2d_hatch::{Hatcher, Intersector};
/// let hatcher = Hatcher::new(&Intersector::new(1e-7, 1e-7), 1e-9, 1e-7, false, false);
/// assert_eq!(hatcher.confusion2d(), 1e-9);
/// ```
pub struct Hatcher {
    pub(super) my_intersector: Intersector,
    pub(super) my_confusion2d: f64,
    pub(super) my_confusion3d: f64,
    pub(super) my_keep_points: bool,
    pub(super) my_keep_segments: bool,
    pub(super) my_nb_elements: i32,
    pub(super) my_elements: Elements,
    pub(super) my_nb_hatchings: i32,
    pub(super) my_hatchings: DataMap<i32, Hatching>,
}

// The Standard_NoSuchObject of the native map's Find and ChangeFind.
fn unbound(what: &str, index: i32) -> Failure {
    Failure::with_type(
        "Standard_NoSuchObject",
        &format!("Geom2dHatch_Hatcher: {what} {index} is not bound; expected a bound index"),
        "",
    )
}

impl Hatcher {
    /// `Geom2dHatch_Hatcher(Intersector, Confusion2d, Confusion3d, KeepPnt = false,
    /// KeepSeg = false)`: an empty hatcher with a copy of `intersector`;
    /// `Hatcher::new(&intersector, 1e-9, 1e-7, false, false)`.
    pub fn new(
        intersector: &Intersector,
        confusion2d: f64,
        confusion3d: f64,
        keep_pnt: bool,
        keep_seg: bool,
    ) -> Self {
        Self {
            my_intersector: intersector.clone(),
            my_confusion2d: confusion2d,
            my_confusion3d: confusion3d,
            my_keep_points: keep_pnt,
            my_keep_segments: keep_seg,
            my_nb_elements: 0,
            my_elements: Elements::new(),
            my_nb_hatchings: 0,
            my_hatchings: DataMap::default(),
        }
    }

    // Runs `change` on every bound hatching, in index order.
    fn for_each_hatching(&mut self, mut change: impl FnMut(&mut Hatching)) {
        for ind_h in 1..=self.my_nb_hatchings {
            if let Some(hatching) = self.my_hatchings.change_seek(&ind_h) {
                change(hatching);
            }
        }
    }

    /// Intersector(Intersector): assigns the intersector and clears the points of
    /// every hatching; `hatcher.set_intersector(&intersector)`.
    pub fn set_intersector(&mut self, intersector: &Intersector) {
        self.my_intersector.assign(intersector);
        self.for_each_hatching(Hatching::clr_points);
    }

    /// Intersector(): the associated intersector; `hatcher.intersector()`.
    pub fn intersector(&self) -> &Intersector {
        &self.my_intersector
    }

    /// ChangeIntersector(); `hatcher.change_intersector()`.
    pub fn change_intersector(&mut self) -> &mut Intersector {
        &mut self.my_intersector
    }

    /// Confusion2d(Confusion): sets the tolerance and clears the points of every
    /// hatching; `hatcher.set_confusion2d(1e-9)`.
    pub fn set_confusion2d(&mut self, confusion: f64) {
        self.my_confusion2d = confusion;
        self.for_each_hatching(Hatching::clr_points);
    }

    /// Confusion2d(): the tolerance under which two points are the same in the
    /// parametric space of a hatching; `hatcher.confusion2d()`.
    pub fn confusion2d(&self) -> f64 {
        self.my_confusion2d
    }

    /// Confusion3d(Confusion): sets the tolerance and clears the points of every
    /// hatching; `hatcher.set_confusion3d(1e-7)`.
    pub fn set_confusion3d(&mut self, confusion: f64) {
        self.my_confusion3d = confusion;
        self.for_each_hatching(Hatching::clr_points);
    }

    /// Confusion3d(): the tolerance under which the two ends of an intersection
    /// segment are one point; `hatcher.confusion3d()`.
    pub fn confusion3d(&self) -> f64 {
        self.my_confusion3d
    }

    /// KeepPoints(Keep): sets the flag and clears the domains of every hatching;
    /// `hatcher.set_keep_points(true)`.
    pub fn set_keep_points(&mut self, keep: bool) {
        self.my_keep_points = keep;
        self.for_each_hatching(Hatching::clr_domains);
    }

    /// KeepPoints(): isolated points make domains; `hatcher.keep_points()`.
    pub fn keep_points(&self) -> bool {
        self.my_keep_points
    }

    /// KeepSegments(Keep): sets the flag and clears the domains of every hatching;
    /// `hatcher.set_keep_segments(true)`.
    pub fn set_keep_segments(&mut self, keep: bool) {
        self.my_keep_segments = keep;
        self.for_each_hatching(Hatching::clr_domains);
    }

    /// KeepSegments(): segments on elements make domains; `hatcher.keep_segments()`.
    pub fn keep_segments(&self) -> bool {
        self.my_keep_segments
    }

    /// Clear(): removes every hatching and every element; `hatcher.clear()`.
    pub fn clear(&mut self) {
        if self.my_nb_hatchings != 0 {
            self.clr_hatchings();
        }
        if self.my_nb_elements != 0 {
            self.clr_elements();
        }
    }
}

impl Hatcher {
    /// ElementCurve(IndE): Standard_NoSuchObject for an unbound index;
    /// `hatcher.element_curve(1)?`.
    pub fn element_curve(&self, ind_e: i32) -> Result<&Geom2dAdaptorCurve, Failure> {
        Ok(self.my_elements.find(ind_e)?.curve())
    }

    /// The protected `Element(IndE)` (public here, as the port has no derived
    /// classes): Standard_NoSuchObject for an unbound index; `hatcher.element_mut(1)?`.
    pub fn element_mut(&mut self, ind_e: i32) -> Result<&mut Element, Failure> {
        self.my_elements.change_find(ind_e)
    }

    /// AddElement(Curve, Orientation = TopAbs_FORWARD): adds an element at the first
    /// free index, clears the points of every hatching and returns the index;
    /// `hatcher.add_element(adaptor, Orientation::Forward)`.
    pub fn add_element(&mut self, curve: Geom2dAdaptorCurve, orientation: Orientation) -> i32 {
        let mut ind_e: i32 = 1;
        while ind_e <= self.my_nb_elements && self.my_elements.is_bound(ind_e) {
            ind_e += 1;
        }
        if ind_e > self.my_nb_elements {
            self.my_nb_elements += 1;
            ind_e = self.my_nb_elements;
        }
        self.my_elements
            .bind(ind_e, Element::new(curve, orientation));
        self.for_each_hatching(Hatching::clr_points);
        ind_e
    }

    /// AddElement(Curve, Orientation) of a Geom2d curve: the element of its adaptor
    /// (Standard_NullObject for a null curve);
    /// `hatcher.add_element_curve(&curve, Orientation::Forward)?`.
    pub fn add_element_curve(
        &mut self,
        curve: &Handle<dyn Geom2dCurve>,
        orientation: Orientation,
    ) -> Result<i32, Failure> {
        let adaptor = Geom2dAdaptorCurve::with_curve(curve)?;
        Ok(self.add_element(adaptor, orientation))
    }

    /// RemElement(IndE): removes the element and its points from every hatching.
    /// Natively each hatching point is scanned through a copy: a point is removed
    /// when all its points on element belong to `ind_e`, but a point kept keeps its
    /// points on `ind_e` as well; the domains of a hatching are cleared when a point
    /// on `ind_e` was seen; `hatcher.rem_element(1)`.
    pub fn rem_element(&mut self, ind_e: i32) {
        self.for_each_hatching(|hatching| rem_element_points(hatching, ind_e));
        self.my_elements.unbind(ind_e);
        if ind_e == self.my_nb_elements {
            self.my_nb_elements -= 1;
        }
    }

    /// ClrElements(): removes every element and clears the points of every
    /// hatching; `hatcher.clr_elements()`.
    pub fn clr_elements(&mut self) {
        if self.my_nb_elements == 0 {
            return;
        }
        if self.my_nb_hatchings != 0 {
            self.for_each_hatching(Hatching::clr_points);
        }
        self.my_elements.clear();
        self.my_nb_elements = 0;
    }
}

// The removal of the points on element `ind_e` from one hatching. Natively each
// point is scanned through a copy from which the points on `ind_e` are removed; the
// hatching point is removed when that copy is empty (so also when it had no point
// on element at all), and otherwise keeps every point on element.
fn rem_element_points(hatching: &mut Hatching, ind_e: i32) {
    let mut domains_to_clear: bool = false;
    for slot in (0..hatching.points().len()).rev() {
        let pnt_h: &PointOnHatching = &hatching.points()[slot];
        let on_ind_e: usize = pnt_h
            .element_points()
            .filter(|pnt_e| pnt_e.index() == ind_e)
            .count();
        let emptied: bool = on_ind_e == pnt_h.element_points().count();
        domains_to_clear |= on_ind_e > 0;
        if emptied {
            hatching.remove_point_slot(slot);
        }
    }
    if domains_to_clear {
        hatching.clr_domains();
    }
}

impl Hatcher {
    /// HatchingCurve(IndH): Standard_NoSuchObject for an unbound index;
    /// `hatcher.hatching_curve(1)?`.
    pub fn hatching_curve(&self, ind_h: i32) -> Result<&Geom2dAdaptorCurve, Failure> {
        Ok(self.hatching(ind_h)?.curve())
    }

    // The hatching `ind_h` (`myHatchings.Find`).
    pub(super) fn hatching(&self, ind_h: i32) -> Result<&Hatching, Failure> {
        self.my_hatchings
            .seek(&ind_h)
            .ok_or_else(|| unbound("hatching", ind_h))
    }

    /// The protected `Hatching(IndH)` (public here, as the port has no derived
    /// classes): Standard_NoSuchObject for an unbound index; `hatcher.hatching_mut(1)?`.
    pub fn hatching_mut(&mut self, ind_h: i32) -> Result<&mut Hatching, Failure> {
        self.my_hatchings
            .change_seek(&ind_h)
            .ok_or_else(|| unbound("hatching", ind_h))
    }

    /// AddHatching(Curve): adds a hatching at the first free index and returns the
    /// index; `hatcher.add_hatching(adaptor)`.
    pub fn add_hatching(&mut self, curve: Geom2dAdaptorCurve) -> i32 {
        let mut ind_h: i32 = 1;
        while ind_h <= self.my_nb_hatchings && self.my_hatchings.is_bound(&ind_h) {
            ind_h += 1;
        }
        if ind_h > self.my_nb_hatchings {
            self.my_nb_hatchings += 1;
            ind_h = self.my_nb_hatchings;
        }
        self.my_hatchings.bind(ind_h, Hatching::new(curve));
        ind_h
    }

    /// RemHatching(IndH): Standard_NoSuchObject for an unbound index;
    /// `hatcher.rem_hatching(1)?`.
    pub fn rem_hatching(&mut self, ind_h: i32) -> Result<(), Failure> {
        self.hatching_mut(ind_h)?.clr_points();
        self.my_hatchings.unbind(&ind_h);
        if ind_h == self.my_nb_hatchings {
            self.my_nb_hatchings -= 1;
        }
        Ok(())
    }

    /// ClrHatchings(): removes every hatching; `hatcher.clr_hatchings()`.
    pub fn clr_hatchings(&mut self) {
        if self.my_nb_hatchings == 0 {
            return;
        }
        self.for_each_hatching(Hatching::clr_points);
        self.my_hatchings.clear(false);
        self.my_nb_hatchings = 0;
    }

    /// NbPoints(IndH): the intersection points of the hatching;
    /// `hatcher.nb_points(1)?`.
    pub fn nb_points(&self, ind_h: i32) -> Result<i32, Failure> {
        Ok(self.hatching(ind_h)?.nb_points())
    }

    /// Point(IndH, IndP): Standard_OutOfRange outside [1, NbPoints];
    /// `hatcher.point(1, 1)?`.
    pub fn point(&self, ind_h: i32, ind_p: i32) -> Result<&PointOnHatching, Failure> {
        self.hatching(ind_h)?.point(ind_p)
    }

    /// TrimDone(IndH): the intersections of the hatching were computed;
    /// `hatcher.trim_done(1)?`.
    pub fn trim_done(&self, ind_h: i32) -> Result<bool, Failure> {
        Ok(self.hatching(ind_h)?.trim_done())
    }

    /// TrimFailed(IndH): an intersection of the hatching failed;
    /// `hatcher.trim_failed(1)?`.
    pub fn trim_failed(&self, ind_h: i32) -> Result<bool, Failure> {
        Ok(self.hatching(ind_h)?.trim_failed())
    }

    /// IsDone(IndH): the domains of the hatching were computed;
    /// `hatcher.is_done(1)?`.
    pub fn is_done(&self, ind_h: i32) -> Result<bool, Failure> {
        Ok(self.hatching(ind_h)?.is_done())
    }

    /// Status(IndH); `hatcher.status(1)?`.
    pub fn status(&self, ind_h: i32) -> Result<ErrorStatus, Failure> {
        Ok(self.hatching(ind_h)?.status())
    }

    /// NbDomains(IndH): StdFail_NotDone before the domains are computed; a single
    /// infinite domain means the whole hatching is inside; `hatcher.nb_domains(1)?`.
    pub fn nb_domains(&self, ind_h: i32) -> Result<i32, Failure> {
        let hatching = self.done_hatching(ind_h, "Geom2dHatch_Hatcher::NbDomains")?;
        Ok(hatching.nb_domains())
    }

    /// Domain(IndH, IDom): StdFail_NotDone before the domains are computed,
    /// Standard_OutOfRange outside [1, NbDomains]; `hatcher.domain(1, 1)?`.
    pub fn domain(&self, ind_h: i32, i_dom: i32) -> Result<&Domain, Failure> {
        self.done_hatching(ind_h, "Geom2dHatch_Hatcher::Domain")?
            .domain(i_dom)
    }

    fn done_hatching(&self, ind_h: i32, member: &str) -> Result<&Hatching, Failure> {
        let hatching = self.hatching(ind_h)?;
        if !hatching.is_done() {
            return Err(not_done(member));
        }
        Ok(hatching)
    }
}
