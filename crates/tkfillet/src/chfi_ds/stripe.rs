// Rust translation of ChFiDS_Stripe.hxx/.lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! A band of fillet: the fillet surfaces computed along one guideline.
use super::errors::null_object;
use super::shared::{Shared, lock};
use super::spine::SpineHandle;
use super::surf_data::SurfDataHandle;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::top_abs::Orientation;

/// A shared, mutable `occ::handle<ChFiDS_Stripe>`; a null native handle is `None`.
pub type StripeHandle = Shared<Stripe>;
/// A shared, mutable `occ::handle<NCollection_HSequence<occ::handle<ChFiDS_SurfData>>>`.
pub type SurfDataSequenceHandle = Shared<Vec<SurfDataHandle>>;

/// The data of one fillet band (`ChFiDS_Stripe`): its guideline, its fillet surfaces, the
/// orientations on the two faces, and per end the parameters, curve and pcurve indices,
/// pcurve orientation, point indices and how many surfaces are in the data structure.
///
/// Index pairs are named per end (`first`/`last`) and per face (`S1`/`S2`), as natively.
///
/// ```
/// use rustycad_tkfillet::chfi_ds::Stripe;
/// let mut stripe = Stripe::new();
/// stripe.set_index_point(5, false, 2);
/// assert_eq!(stripe.index_last_point_on_s2(), 5);
/// stripe.in_ds(true, 2);
/// assert_eq!(stripe.is_in_ds(true), 2);
/// ```
#[derive(Clone)]
pub struct Stripe {
    parent: StandardTransient,
    first_params: (f64, f64),
    last_params: (f64, f64),
    spine: Option<SpineHandle>,
    surf_data: Option<SurfDataSequenceHandle>,
    first_pcurve: Handle<dyn Geom2dCurve>,
    last_pcurve: Handle<dyn Geom2dCurve>,
    choix: i32,
    solid_index: i32,
    first_curve: i32,
    last_curve: i32,
    first_point_on_s1: i32,
    last_point_on_s1: i32,
    first_point_on_s2: i32,
    last_point_on_s2: i32,
    first_in_ds: i32,
    last_in_ds: i32,
    orientation_on_face1: Orientation,
    orientation_on_face2: Orientation,
    first_pcurve_orientation: Orientation,
    last_pcurve_orientation: Orientation,
}
impl_transient!(Stripe, "ChFiDS_Stripe", StandardTransient, parent);

impl Default for Stripe {
    fn default() -> Self {
        Self {
            parent: StandardTransient::default(),
            first_params: (0.0, 0.0),
            last_params: (0.0, 0.0),
            spine: None,
            surf_data: None,
            first_pcurve: Handle::default(),
            last_pcurve: Handle::default(),
            choix: 0,
            solid_index: 0,
            first_curve: 0,
            last_curve: 0,
            first_point_on_s1: 0,
            last_point_on_s1: 0,
            first_point_on_s2: 0,
            last_point_on_s2: 0,
            first_in_ds: 0,
            last_in_ds: 0,
            orientation_on_face1: Orientation::Forward,
            orientation_on_face2: Orientation::Forward,
            first_pcurve_orientation: Orientation::Forward,
            last_pcurve_orientation: Orientation::Forward,
        }
    }
}

impl Stripe {
    /// No spine, no surfaces, zero indices, FORWARD orientations; `Stripe::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `Reset()`: forgets the surfaces, the pcurve orientations and the first pcurve,
    /// then resets the spine (not its data). Natively the first pcurve is nullified twice
    /// and the last one is kept; that quirk is kept. Standard_NullObject without a spine;
    /// `stripe.reset()?`.
    pub fn reset(&mut self) -> Result<(), Failure> {
        self.surf_data = None;
        self.first_pcurve_orientation = Orientation::Forward;
        self.last_pcurve_orientation = Orientation::Forward;
        self.first_pcurve = Handle::default();
        let spine = self
            .spine
            .as_ref()
            .ok_or_else(|| null_object("ChFiDS_Stripe::Reset: null spine; expected a spine"))?;
        lock(spine).reset(false)
    }

    /// `SetOfSurfData()`: the fillet surfaces, `None` for a null handle;
    /// `stripe.set_of_surf_data()`.
    pub fn set_of_surf_data(&self) -> Option<&SurfDataSequenceHandle> {
        self.surf_data.as_ref()
    }

    /// `ChangeSetOfSurfData()`: the editable handle of the surfaces;
    /// `*stripe.change_set_of_surf_data() = Some(shared(Vec::new()))`.
    pub fn change_set_of_surf_data(&mut self) -> &mut Option<SurfDataSequenceHandle> {
        &mut self.surf_data
    }

    /// `Spine()`: the guideline, `None` for a null handle; `stripe.spine()`.
    pub fn spine(&self) -> Option<&SpineHandle> {
        self.spine.as_ref()
    }

    /// `ChangeSpine()`: the editable handle of the guideline;
    /// `*stripe.change_spine() = Some(spine)`.
    pub fn change_spine(&mut self) -> &mut Option<SpineHandle> {
        &mut self.spine
    }

    /// `OrientationOnFace1()`; `stripe.orientation_on_face1()`.
    pub fn orientation_on_face1(&self) -> Orientation {
        self.orientation_on_face1
    }

    /// `OrientationOnFace2()`; `stripe.orientation_on_face2()`.
    pub fn orientation_on_face2(&self) -> Orientation {
        self.orientation_on_face2
    }

    /// `OrientationOnFace1(Or1)`; `stripe.set_orientation_on_face1(Orientation::Reversed)`.
    pub fn set_orientation_on_face1(&mut self, orientation: Orientation) {
        self.orientation_on_face1 = orientation;
    }

    /// `OrientationOnFace2(Or2)`; `stripe.set_orientation_on_face2(Orientation::Reversed)`.
    pub fn set_orientation_on_face2(&mut self, orientation: Orientation) {
        self.orientation_on_face2 = orientation;
    }

    /// `Choix()`: the choice of the solution of the section; `stripe.choix()`.
    pub fn choix(&self) -> i32 {
        self.choix
    }

    /// `Choix(C)`; `stripe.set_choix(1)`.
    pub fn set_choix(&mut self, choix: i32) {
        self.choix = choix;
    }

    /// `FirstParameters(Pdeb, Pfin)`; `let (start, end) = stripe.first_parameters();`.
    pub fn first_parameters(&self) -> (f64, f64) {
        self.first_params
    }

    /// `LastParameters(Pdeb, Pfin)`; `let (start, end) = stripe.last_parameters();`.
    pub fn last_parameters(&self) -> (f64, f64) {
        self.last_params
    }

    /// `ChangeFirstParameters(Pdeb, Pfin)`; `stripe.change_first_parameters(0.0, 1.0)`.
    pub fn change_first_parameters(&mut self, start: f64, end: f64) {
        self.first_params = (start, end);
    }

    /// `ChangeLastParameters(Pdeb, Pfin)`; `stripe.change_last_parameters(0.0, 1.0)`.
    pub fn change_last_parameters(&mut self, start: f64, end: f64) {
        self.last_params = (start, end);
    }

    /// `FirstCurve()`; `stripe.first_curve()`.
    pub fn first_curve(&self) -> i32 {
        self.first_curve
    }

    /// `LastCurve()`; `stripe.last_curve()`.
    pub fn last_curve(&self) -> i32 {
        self.last_curve
    }

    /// `ChangeFirstCurve(Index)`; `stripe.change_first_curve(1)`.
    pub fn change_first_curve(&mut self, index: i32) {
        self.first_curve = index;
    }

    /// `ChangeLastCurve(Index)`; `stripe.change_last_curve(1)`.
    pub fn change_last_curve(&mut self, index: i32) {
        self.last_curve = index;
    }

    /// `FirstPCurve()`; `stripe.first_pcurve()`.
    pub fn first_pcurve(&self) -> &Handle<dyn Geom2dCurve> {
        &self.first_pcurve
    }

    /// `LastPCurve()`; `stripe.last_pcurve()`.
    pub fn last_pcurve(&self) -> &Handle<dyn Geom2dCurve> {
        &self.last_pcurve
    }

    /// `ChangeFirstPCurve()`; `*stripe.change_first_pcurve() = curve`.
    pub fn change_first_pcurve(&mut self) -> &mut Handle<dyn Geom2dCurve> {
        &mut self.first_pcurve
    }

    /// `ChangeLastPCurve()`; `*stripe.change_last_pcurve() = curve`.
    pub fn change_last_pcurve(&mut self) -> &mut Handle<dyn Geom2dCurve> {
        &mut self.last_pcurve
    }

    /// `FirstPCurveOrientation()`; `stripe.first_pcurve_orientation()`.
    pub fn first_pcurve_orientation(&self) -> Orientation {
        self.first_pcurve_orientation
    }

    /// `LastPCurveOrientation()`; `stripe.last_pcurve_orientation()`.
    pub fn last_pcurve_orientation(&self) -> Orientation {
        self.last_pcurve_orientation
    }

    /// `FirstPCurveOrientation(O)`; `stripe.set_first_pcurve_orientation(Orientation::Reversed)`.
    pub fn set_first_pcurve_orientation(&mut self, orientation: Orientation) {
        self.first_pcurve_orientation = orientation;
    }

    /// `LastPCurveOrientation(O)`; `stripe.set_last_pcurve_orientation(Orientation::Reversed)`.
    pub fn set_last_pcurve_orientation(&mut self, orientation: Orientation) {
        self.last_pcurve_orientation = orientation;
    }

    /// `IndexFirstPointOnS1()`; `stripe.index_first_point_on_s1()`.
    pub fn index_first_point_on_s1(&self) -> i32 {
        self.first_point_on_s1
    }

    /// `IndexFirstPointOnS2()`; `stripe.index_first_point_on_s2()`.
    pub fn index_first_point_on_s2(&self) -> i32 {
        self.first_point_on_s2
    }

    /// `IndexLastPointOnS1()`; `stripe.index_last_point_on_s1()`.
    pub fn index_last_point_on_s1(&self) -> i32 {
        self.last_point_on_s1
    }

    /// `IndexLastPointOnS2()`; `stripe.index_last_point_on_s2()`.
    pub fn index_last_point_on_s2(&self) -> i32 {
        self.last_point_on_s2
    }

    /// `ChangeIndexFirstPointOnS1(Index)`; `stripe.change_index_first_point_on_s1(1)`.
    pub fn change_index_first_point_on_s1(&mut self, index: i32) {
        self.first_point_on_s1 = index;
    }

    /// `ChangeIndexFirstPointOnS2(Index)`; `stripe.change_index_first_point_on_s2(1)`.
    pub fn change_index_first_point_on_s2(&mut self, index: i32) {
        self.first_point_on_s2 = index;
    }

    /// `ChangeIndexLastPointOnS1(Index)`; `stripe.change_index_last_point_on_s1(1)`.
    pub fn change_index_last_point_on_s1(&mut self, index: i32) {
        self.last_point_on_s1 = index;
    }

    /// `ChangeIndexLastPointOnS2(Index)`; `stripe.change_index_last_point_on_s2(1)`.
    pub fn change_index_last_point_on_s2(&mut self, index: i32) {
        self.last_point_on_s2 = index;
    }

    /// `Parameters(First, Pdeb, Pfin)`; `let (start, end) = stripe.parameters(true);`.
    pub fn parameters(&self, first: bool) -> (f64, f64) {
        if first {
            self.first_params
        } else {
            self.last_params
        }
    }

    /// `SetParameters(First, Pdeb, Pfin)`; `stripe.set_parameters(true, 0.0, 1.0)`.
    pub fn set_parameters(&mut self, first: bool, start: f64, end: f64) {
        if first {
            self.first_params = (start, end);
        } else {
            self.last_params = (start, end);
        }
    }

    /// `Curve(First)`; `stripe.curve(true)`.
    pub fn curve(&self, first: bool) -> i32 {
        if first {
            self.first_curve
        } else {
            self.last_curve
        }
    }

    /// `SetCurve(Index, First)`; `stripe.set_curve(3, false)`.
    pub fn set_curve(&mut self, index: i32, first: bool) {
        if first {
            self.first_curve = index;
        } else {
            self.last_curve = index;
        }
    }

    /// `PCurve(First)`; `stripe.pcurve(true)`.
    pub fn pcurve(&self, first: bool) -> &Handle<dyn Geom2dCurve> {
        if first {
            &self.first_pcurve
        } else {
            &self.last_pcurve
        }
    }

    /// `ChangePCurve(First)`; `*stripe.change_pcurve(true) = curve`.
    pub fn change_pcurve(&mut self, first: bool) -> &mut Handle<dyn Geom2dCurve> {
        if first {
            &mut self.first_pcurve
        } else {
            &mut self.last_pcurve
        }
    }

    /// `Orientation(OnS)`: the orientation on the first face for `on_s == 1`, else on the
    /// second; `stripe.orientation_on_face(1)`.
    pub fn orientation_on_face(&self, on_s: i32) -> Orientation {
        if on_s == 1 {
            self.orientation_on_face1
        } else {
            self.orientation_on_face2
        }
    }

    /// `SetOrientation(Or, OnS)`; `stripe.set_orientation_on_face(Orientation::Reversed, 2)`.
    pub fn set_orientation_on_face(&mut self, orientation: Orientation, on_s: i32) {
        if on_s == 1 {
            self.orientation_on_face1 = orientation;
        } else {
            self.orientation_on_face2 = orientation;
        }
    }

    /// `Orientation(First)`: the pcurve orientation at the first or last end;
    /// `stripe.orientation_at_end(true)`.
    pub fn orientation_at_end(&self, first: bool) -> Orientation {
        if first {
            self.first_pcurve_orientation
        } else {
            self.last_pcurve_orientation
        }
    }

    /// `SetOrientation(Or, First)`; `stripe.set_orientation_at_end(Orientation::Reversed, true)`.
    pub fn set_orientation_at_end(&mut self, orientation: Orientation, first: bool) {
        if first {
            self.first_pcurve_orientation = orientation;
        } else {
            self.last_pcurve_orientation = orientation;
        }
    }

    /// `IndexPoint(First, OnS)`: one of the four point indices (`on_s != 1` is the second
    /// face); `stripe.index_point(true, 1)`.
    pub fn index_point(&self, first: bool, on_s: i32) -> i32 {
        match (first, on_s == 1) {
            (true, true) => self.first_point_on_s1,
            (true, false) => self.first_point_on_s2,
            (false, true) => self.last_point_on_s1,
            (false, false) => self.last_point_on_s2,
        }
    }

    /// `SetIndexPoint(Index, First, OnS)`; `stripe.set_index_point(4, true, 2)`.
    pub fn set_index_point(&mut self, index: i32, first: bool, on_s: i32) {
        let slot = match (first, on_s == 1) {
            (true, true) => &mut self.first_point_on_s1,
            (true, false) => &mut self.first_point_on_s2,
            (false, true) => &mut self.last_point_on_s1,
            (false, false) => &mut self.last_point_on_s2,
        };
        *slot = index;
    }

    /// `SolidIndex()`; `stripe.solid_index()`.
    pub fn solid_index(&self) -> i32 {
        self.solid_index
    }

    /// `SetSolidIndex(Index)`; `stripe.set_solid_index(1)`.
    pub fn set_solid_index(&mut self, index: i32) {
        self.solid_index = index;
    }

    /// `InDS(First, Nb)`: the number of surfaces of that end put in the data structure
    /// (the native default of `Nb` is 1); `stripe.in_ds(true, 1)`.
    pub fn in_ds(&mut self, first: bool, count: i32) {
        if first {
            self.first_in_ds = count;
        } else {
            self.last_in_ds = count;
        }
    }

    /// `IsInDS(First)`: the number of surfaces of that end in the data structure;
    /// `stripe.is_in_ds(false)`.
    pub fn is_in_ds(&self, first: bool) -> i32 {
        if first {
            self.first_in_ds
        } else {
            self.last_in_ds
        }
    }
}

#[cfg(test)]
#[path = "stripe_tests.rs"]
mod tests;
