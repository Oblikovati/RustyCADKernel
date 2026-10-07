// Rust translation of ChFiDS_ChamfSpine.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The guideline of a chamfer with its distances or its distance and angle.
use super::errors::standard_failure;
use super::spine::{Spine, SpineObject};
use super::{ChamfMethod, ChamfMode, ElSpine};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle};
use std::ops::{Deref, DerefMut};

/// A chamfer guideline (`ChFiDS_ChamfSpine`): a [`Spine`] with the chamfer method and its
/// distances or distance and angle.
///
/// It dereferences to its [`Spine`] part, as the native class inherits it.
///
/// ```
/// use rustycad_tkfillet::chfi_ds::{ChamfMethod, ChamfSpine};
/// let mut spine = ChamfSpine::new();
/// spine.set_dists(1.0, 2.0);
/// assert_eq!(spine.is_chamfer(), ChamfMethod::TwoDist);
/// assert_eq!(spine.dists()?, (1.0, 2.0));
/// assert!(spine.get_dist().is_err());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct ChamfSpine {
    spine: Spine,
    d1: f64,
    d2: f64,
    angle: f64,
    method: ChamfMethod,
}
impl_transient!(ChamfSpine, "ChFiDS_ChamfSpine", Spine, spine);

impl Default for ChamfSpine {
    fn default() -> Self {
        Self::from_spine(Spine::new())
    }
}

impl ChamfSpine {
    /// `ChFiDS_ChamfSpine()`: a symmetric chamfer guideline of distance 0, classic mode;
    /// `ChamfSpine::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `ChFiDS_ChamfSpine(Tol)`; `ChamfSpine::with_tolerance(1e-4)`.
    pub fn with_tolerance(tolerance: f64) -> Self {
        Self::from_spine(Spine::with_tolerance(tolerance))
    }

    fn from_spine(mut spine: Spine) -> Self {
        spine.set_mode(ChamfMode::ClassicChamfer);
        Self {
            spine,
            d1: 0.0,
            d2: 0.0,
            angle: 0.0,
            method: ChamfMethod::Sym,
        }
    }

    /// `GetDist(Dis)`: Standard_Failure "Chamfer is not symmetric" unless symmetric;
    /// `spine.get_dist()?`.
    pub fn get_dist(&self) -> Result<f64, Failure> {
        if self.method != ChamfMethod::Sym {
            return Err(standard_failure("Chamfer is not symmetric"));
        }
        Ok(self.d1)
    }

    /// `SetDist(Dis)`: a symmetric chamfer of distance `distance`; `spine.set_dist(1.0)`.
    pub fn set_dist(&mut self, distance: f64) {
        self.method = ChamfMethod::Sym;
        self.d1 = distance;
    }

    /// `Dists(Dis1, Dis2)`: Standard_Failure unless a two-distance chamfer;
    /// `let (d1, d2) = spine.dists()?;`.
    pub fn dists(&self) -> Result<(f64, f64), Failure> {
        if self.method != ChamfMethod::TwoDist {
            return Err(standard_failure("Chamfer is not a Two Dists Chamfer"));
        }
        Ok((self.d1, self.d2))
    }

    /// `SetDists(Dis1, Dis2)`: a two-distance chamfer; `spine.set_dists(1.0, 2.0)`.
    pub fn set_dists(&mut self, distance1: f64, distance2: f64) {
        self.method = ChamfMethod::TwoDist;
        self.d1 = distance1;
        self.d2 = distance2;
    }

    /// `GetDistAngle(Dis, Angle)`: Standard_Failure (with the native "Two Dists" message)
    /// unless a distance-angle chamfer; `let (d, a) = spine.get_dist_angle()?;`.
    pub fn get_dist_angle(&self) -> Result<(f64, f64), Failure> {
        if self.method != ChamfMethod::DistAngle {
            return Err(standard_failure("Chamfer is not a Two Dists Chamfer"));
        }
        Ok((self.d1, self.angle))
    }

    /// `SetDistAngle(Dis, Angle)`: a distance-angle chamfer; `spine.set_dist_angle(1.0, 0.5)`.
    pub fn set_dist_angle(&mut self, distance: f64, angle: f64) {
        self.method = ChamfMethod::DistAngle;
        self.d1 = distance;
        self.angle = angle;
    }

    /// `SetMode(theMode)`: the chamfer mode of the guideline;
    /// `spine.set_chamfer_mode(ChamfMode::ConstThroatChamfer)`.
    pub fn set_chamfer_mode(&mut self, mode: ChamfMode) {
        self.spine.set_mode(mode);
    }

    /// `IsChamfer()`: the chamfer method; `spine.is_chamfer()`.
    pub fn is_chamfer(&self) -> ChamfMethod {
        self.method
    }
}

impl SpineObject for ChamfSpine {
    fn spine(&self) -> &Spine {
        &self.spine
    }
    fn spine_mut(&mut self) -> &mut Spine {
        &mut self.spine
    }
    fn reset(&mut self, all_data: bool) -> Result<(), Failure> {
        SpineObject::reset(&mut self.spine, all_data)
    }
    fn append_el_spine(&mut self, el_spine: &Handle<ElSpine>) -> Result<(), Failure> {
        SpineObject::append_el_spine(&mut self.spine, el_spine)
    }
    fn as_chamf_spine(&self) -> Option<&ChamfSpine> {
        Some(self)
    }
    fn as_chamf_spine_mut(&mut self) -> Option<&mut ChamfSpine> {
        Some(self)
    }
}

impl Deref for ChamfSpine {
    type Target = Spine;
    fn deref(&self) -> &Spine {
        &self.spine
    }
}

impl DerefMut for ChamfSpine {
    fn deref_mut(&mut self) -> &mut Spine {
        &mut self.spine
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkernel::standard::RuntimeTyped;

    #[test]
    fn each_query_requires_its_method() {
        let mut spine = ChamfSpine::new();
        assert_eq!(spine.get_dist().unwrap(), 0.0);
        spine.set_dist_angle(2.0, 0.25);
        assert_eq!(spine.get_dist_angle().unwrap(), (2.0, 0.25));
        assert_eq!(
            spine.get_dist().unwrap_err().message(),
            "Chamfer is not symmetric"
        );
        assert!(spine.dists().is_err());
    }

    #[test]
    fn mode_is_classic_until_set() {
        let mut spine = ChamfSpine::with_tolerance(1e-3);
        assert_eq!(spine.mode(), ChamfMode::ClassicChamfer);
        spine.set_chamfer_mode(ChamfMode::ConstThroatChamfer);
        assert_eq!(spine.mode(), ChamfMode::ConstThroatChamfer);
        assert_eq!(spine.get_tolesp(), 1e-3);
    }

    #[test]
    fn registered_under_the_spine_descriptor() {
        let descriptor = ChamfSpine::static_type();
        assert_eq!(descriptor.name(), "ChFiDS_ChamfSpine");
        assert!(descriptor.is_subtype(Spine::static_type()));
    }
}
